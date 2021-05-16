use crate::error::{UseCaseError, UseCaseResult};
use crate::model::file::FileObject;
use crate::model::file_sharing::FileSharingId;
use crate::model::form_answer::FormAnswerId;

use anyhow::Context;
use sos21_domain::context::{FileSharingRepository, FormAnswerRepository, Login, ObjectRepository};
use sos21_domain::model::file_sharing;

#[derive(Debug, Clone)]
pub struct Input {
    pub answer_id: FormAnswerId,
    pub sharing_id: FileSharingId,
}

#[derive(Debug, Clone)]
pub enum Error {
    FormAnswerNotFound,
    FileSharingNotFound,
    InvalidSharing,
}

impl Error {
    fn from_witness_error(err: file_sharing::ToWitnessError) -> Self {
        match err.kind() {
            file_sharing::ToWitnessErrorKind::OutOfScope => Error::FileSharingNotFound,
            file_sharing::ToWitnessErrorKind::ExpiredSharing => Error::InvalidSharing,
            file_sharing::ToWitnessErrorKind::RevokedSharing => Error::InvalidSharing,
        }
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<FileObject, Error>
where
    C: FormAnswerRepository + FileSharingRepository + ObjectRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let result = ctx
        .get_form_answer(input.answer_id.into_entity())
        .await
        .context("Failed to get a form answer")?;
    let answer = match result {
        Some(answer) if answer.is_visible_to(login_user) => answer,
        _ => return Err(UseCaseError::UseCase(Error::FormAnswerNotFound)),
    };

    let result = ctx
        .get_file_sharing(input.sharing_id.into_entity())
        .await
        .context("Failed to get a file sharing")?;
    let (sharing, file) = match result {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::FileSharingNotFound)),
    };

    let witness = sharing
        .to_witness_with_form_answer(&answer)
        .map_err(|err| UseCaseError::UseCase(Error::from_witness_error(err)))?;

    use_case_ensure!(file.is_visible_to_with_sharing(&witness));

    let object = ctx
        .get_object(file.object_id)
        .await
        .context("Failed to get an object")?
        .context("Could not find an object referenced by object_id")?;

    use_case_ensure!(object.is_visible_to_with_sharing(&file, &witness));
    Ok(FileObject::from_entity(file, object))
}

#[cfg(test)]
mod tests {
    use crate::model::file::FileId;
    use crate::model::file_sharing::FileSharingId;
    use crate::model::form_answer::FormAnswerId;
    use crate::{get_form_answer_shared_file_object, UseCaseError};

    use sos21_domain::model::file_sharing;
    use sos21_domain::test;

    // Checks that the committee user can read others' file which is shared to a form answer.
    #[tokio::test]
    async fn test_committee_get() {
        let user = test::model::new_committee_user();
        let operator = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let (other_file, other_object) = test::model::new_file(other.id().clone());

        let other_project = test::model::new_general_project(other.id().clone());
        let form = test::model::new_form(operator.id().clone());
        let other_answer =
            test::model::new_form_answer(other.id().clone(), other_project.id(), &form);

        let sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::form_answer_scope(&other_answer),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), operator, other])
            .projects(vec![other_project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .forms(vec![form.clone()])
            .answers(vec![other_answer.clone()])
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user)
            .await;

        let input = get_form_answer_shared_file_object::Input {
            answer_id: FormAnswerId::from_entity(other_answer.id()),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_form_answer_shared_file_object::run(&app, input).await,
            Ok(object)
            if object.file.id == FileId::from_entity(other_file.id)
        ));
    }

    // Checks that the general user cannot read others' file which is shared to other's form answer.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let operator = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let (other_file, other_object) = test::model::new_file(other.id().clone());

        let other_project = test::model::new_general_project(other.id().clone());
        let form = test::model::new_form(operator.id().clone());
        let other_answer =
            test::model::new_form_answer(other.id().clone(), other_project.id(), &form);

        let sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::form_answer_scope(&other_answer),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), operator, other])
            .projects(vec![other_project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .forms(vec![form.clone()])
            .answers(vec![other_answer.clone()])
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user)
            .await;

        let input = get_form_answer_shared_file_object::Input {
            answer_id: FormAnswerId::from_entity(other_answer.id()),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_form_answer_shared_file_object::run(&app, input).await,
            Err(UseCaseError::UseCase(
                get_form_answer_shared_file_object::Error::FormAnswerNotFound
            ))
        ));
    }

    // Checks that the general user cannot read others' file which is not shared to the owning form answer using it.
    #[tokio::test]
    async fn test_general_owner_other() {
        let user = test::model::new_general_user();
        let operator = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let (other_file, other_object) = test::model::new_file(other.id().clone());

        let project = test::model::new_general_project(user.id().clone());
        let other_project = test::model::new_general_project(other.id().clone());
        let form = test::model::new_form(operator.id().clone());
        let answer = test::model::new_form_answer(user.id().clone(), project.id(), &form);
        let other_answer =
            test::model::new_form_answer(other.id().clone(), other_project.id(), &form);

        let sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::form_answer_scope(&other_answer),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), operator, other])
            .projects(vec![project, other_project])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .forms(vec![form.clone()])
            .answers(vec![answer.clone(), other_answer])
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user)
            .await;

        let input = get_form_answer_shared_file_object::Input {
            answer_id: FormAnswerId::from_entity(answer.id()),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_form_answer_shared_file_object::run(&app, input).await,
            Err(UseCaseError::UseCase(_))
        ));
    }

    // Checks that the committee user cannot read others' file which is shared to a form answer but
    // revoked.
    #[tokio::test]
    async fn test_committee_revoked() {
        let user = test::model::new_committee_user();
        let operator = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let (other_file, other_object) = test::model::new_file(other.id().clone());

        let other_project = test::model::new_general_project(other.id().clone());
        let form = test::model::new_form(operator.id().clone());
        let other_answer =
            test::model::new_form_answer(other.id().clone(), other_project.id(), &form);

        let mut sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::form_answer_scope(&other_answer),
        );
        sharing.revoke().unwrap();

        let app = test::build_mock_app()
            .users(vec![user.clone(), operator, other])
            .projects(vec![other_project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .forms(vec![form.clone()])
            .answers(vec![other_answer.clone()])
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user)
            .await;

        let input = get_form_answer_shared_file_object::Input {
            answer_id: FormAnswerId::from_entity(other_answer.id()),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_form_answer_shared_file_object::run(&app, input).await,
            Err(UseCaseError::UseCase(
                get_form_answer_shared_file_object::Error::InvalidSharing
            ))
        ));
    }

    // Checks that the committee user cannot read others' file which is shared to a form answer but
    // expired.
    #[tokio::test]
    async fn test_committee_expired() {
        let user = test::model::new_committee_user();
        let operator = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let (other_file, other_object) = test::model::new_file(other.id().clone());

        let other_project = test::model::new_general_project(other.id().clone());
        let form = test::model::new_form(operator.id().clone());
        let other_answer =
            test::model::new_form_answer(other.id().clone(), other_project.id(), &form);

        let sharing = test::model::new_expired_file_sharing(
            other_file.id,
            file_sharing::FileSharingScope::form_answer_scope(&other_answer),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), operator, other])
            .projects(vec![other_project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .forms(vec![form.clone()])
            .answers(vec![other_answer.clone()])
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user)
            .await;

        let input = get_form_answer_shared_file_object::Input {
            answer_id: FormAnswerId::from_entity(other_answer.id()),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_form_answer_shared_file_object::run(&app, input).await,
            Err(UseCaseError::UseCase(
                get_form_answer_shared_file_object::Error::InvalidSharing
            ))
        ));
    }
}
