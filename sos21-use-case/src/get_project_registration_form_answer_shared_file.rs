use crate::error::{UseCaseError, UseCaseResult};
use crate::model::file::File;
use crate::model::file_sharing::FileSharingId;
use crate::model::project::ProjectId;
use crate::model::registration_form::RegistrationFormId;

use anyhow::Context;
use sos21_domain::context::{
    FileSharingRepository, Login, ProjectRepository, RegistrationFormAnswerRepository,
    RegistrationFormRepository,
};
use sos21_domain::model::file_sharing;

#[derive(Debug, Clone)]
pub struct Input {
    pub project_id: ProjectId,
    pub registration_form_id: RegistrationFormId,
    pub sharing_id: FileSharingId,
}

#[derive(Debug, Clone)]
pub enum Error {
    ProjectNotFound,
    RegistrationFormNotFound,
    RegistrationFormAnswerNotFound,
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
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<File, Error>
where
    C: ProjectRepository
        + RegistrationFormAnswerRepository
        + RegistrationFormRepository
        + FileSharingRepository
        + Send
        + Sync,
{
    let login_user = ctx.login_user();

    let result = ctx
        .get_project(input.project_id.into_entity())
        .await
        .context("Failed to get a project")?;
    let project = match result {
        Some(result) if result.project.is_visible_to(login_user) => result.project,
        _ => return Err(UseCaseError::UseCase(Error::ProjectNotFound)),
    };

    let registration_form_id = input.registration_form_id.into_entity();
    let result = ctx
        .get_registration_form_answer_by_registration_form_and_project(
            registration_form_id,
            project.id(),
        )
        .await
        .context("Failed to get a registration form answer")?;
    let answer = match result {
        Some(answer) if answer.is_visible_to_with_project(login_user, &project) => answer,
        _ => {
            if let Some(registration_form) = ctx.get_registration_form(registration_form_id).await?
            {
                if registration_form.is_visible_to_with_project(login_user, &project) {
                    return Err(UseCaseError::UseCase(Error::RegistrationFormAnswerNotFound));
                }
            }
            return Err(UseCaseError::UseCase(Error::RegistrationFormNotFound));
        }
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
        .to_witness_with_registration_form_answer(&answer)
        .map_err(|err| UseCaseError::UseCase(Error::from_witness_error(err)))?;

    use_case_ensure!(file.is_visible_to_with_sharing(&witness));
    Ok(File::from_entity(file))
}

#[cfg(test)]
mod tests {
    use crate::model::file::FileId;
    use crate::model::file_sharing::FileSharingId;
    use crate::model::project::ProjectId;
    use crate::model::registration_form::RegistrationFormId;
    use crate::{get_project_registration_form_answer_shared_file, UseCaseError};

    use sos21_domain::model::{file_sharing, registration_form_answer};
    use sos21_domain::test;

    // Checks that the general user cannot read others' file which is shared to other's registration form answer.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let operator = test::model::new_operator_user();

        let (other_file, other_object) = test::model::new_file(other.id().clone());
        let registration_form = test::model::new_registration_form(operator.id().clone());
        let other_project = test::model::new_general_project(other.id().clone());
        let other_answer = test::model::new_registration_form_answer_with_project(
            other.id().clone(),
            other_project.id(),
            &registration_form,
        );

        let sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::RegistrationFormAnswer(
                registration_form_answer::RegistrationFormAnswerRespondent::Project(
                    other_project.id(),
                ),
                other_answer.registration_form_id(),
            ),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone(), operator.clone()])
            .projects(vec![other_project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![other_answer])
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_registration_form_answer_shared_file::Input {
            project_id: ProjectId::from_entity(other_project.id()),
            registration_form_id: RegistrationFormId::from_entity(registration_form.id),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_registration_form_answer_shared_file::run(&app, input).await,
            Err(UseCaseError::UseCase(_))
        ));
    }

    // Checks that the general user can read others' file which is shared to a registration form answer from
    // an owning project.
    #[tokio::test]
    async fn test_general_owner_get() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let operator = test::model::new_operator_user();

        let (other_file, other_object) = test::model::new_file(other.id().clone());
        let registration_form = test::model::new_registration_form(operator.id().clone());
        let project = test::model::new_general_project(user.id().clone());
        let answer = test::model::new_registration_form_answer_with_project(
            user.id().clone(),
            project.id(),
            &registration_form,
        );

        let sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::RegistrationFormAnswer(
                registration_form_answer::RegistrationFormAnswerRespondent::Project(project.id()),
                answer.registration_form_id(),
            ),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer])
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_registration_form_answer_shared_file::Input {
            project_id: ProjectId::from_entity(project.id()),
            registration_form_id: RegistrationFormId::from_entity(registration_form.id),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_registration_form_answer_shared_file::run(&app, input).await,
            Ok(file)
            if file.id == FileId::from_entity(other_file.id)
        ));
    }

    // Checks that the general user can read others' file which is shared to a registration form answer from
    // an owning project.
    #[tokio::test]
    async fn test_general_subowner_get() {
        let owner = test::model::new_general_user();
        let user = test::model::new_general_user();
        let operator = test::model::new_operator_user();

        let (owner_file, owner_object) = test::model::new_file(owner.id().clone());
        let registration_form = test::model::new_registration_form(operator.id().clone());
        let project = test::model::new_general_project_with_subowner(
            owner.id().clone(),
            user.id().clone(),
        );
        let answer = test::model::new_registration_form_answer_with_project(
            owner.id().clone(),
            project.id(),
            &registration_form,
        );

        let sharing = file_sharing::FileSharing::new(
            owner_file.id,
            file_sharing::FileSharingScope::RegistrationFormAnswer(
                registration_form_answer::RegistrationFormAnswerRespondent::Project(project.id()),
                answer.registration_form_id(),
            ),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), owner.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .files(vec![owner_file.clone()])
            .objects(vec![owner_object])
            .await
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer])
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_registration_form_answer_shared_file::Input {
            project_id: ProjectId::from_entity(project.id()),
            registration_form_id: RegistrationFormId::from_entity(registration_form.id),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_registration_form_answer_shared_file::run(&app, input).await,
            Ok(file)
            if file.id == FileId::from_entity(owner_file.id)
        ));
    }

    // Checks that the general user cannot read others' file which is not shared to a form answer from
    // an owning project using an owning project.
    #[tokio::test]
    async fn test_general_owner_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let operator = test::model::new_operator_user();

        let (other_file, other_object) = test::model::new_file(other.id().clone());
        let registration_form = test::model::new_registration_form(operator.id().clone());
        let project = test::model::new_general_project(user.id().clone());
        let answer = test::model::new_registration_form_answer_with_project(
            user.id().clone(),
            project.id(),
            &registration_form,
        );
        let other_project = test::model::new_general_project(other.id().clone());
        let other_answer = test::model::new_registration_form_answer_with_project(
            other.id().clone(),
            other_project.id(),
            &registration_form,
        );

        let sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::RegistrationFormAnswer(
                registration_form_answer::RegistrationFormAnswerRespondent::Project(
                    other_project.id(),
                ),
                other_answer.registration_form_id(),
            ),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone(), operator.clone()])
            .projects(vec![project.clone(), other_project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer, other_answer])
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_registration_form_answer_shared_file::Input {
            project_id: ProjectId::from_entity(project.id()),
            registration_form_id: RegistrationFormId::from_entity(registration_form.id),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_registration_form_answer_shared_file::run(&app, input).await,
            Err(UseCaseError::UseCase(
                get_project_registration_form_answer_shared_file::Error::FileSharingNotFound
            ))
        ));
    }

    // Checks that the general user cannot read others' file which is shared to a registration form answer from
    // an owning project but revoked.
    #[tokio::test]
    async fn test_general_owner_revoked() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let operator = test::model::new_operator_user();

        let (other_file, other_object) = test::model::new_file(other.id().clone());
        let registration_form = test::model::new_registration_form(operator.id().clone());
        let project = test::model::new_general_project(user.id().clone());
        let answer = test::model::new_registration_form_answer_with_project(
            user.id().clone(),
            project.id(),
            &registration_form,
        );

        let mut sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::RegistrationFormAnswer(
                registration_form_answer::RegistrationFormAnswerRespondent::Project(project.id()),
                answer.registration_form_id(),
            ),
        );

        sharing.revoke().unwrap();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer])
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_registration_form_answer_shared_file::Input {
            project_id: ProjectId::from_entity(project.id()),
            registration_form_id: RegistrationFormId::from_entity(registration_form.id),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_registration_form_answer_shared_file::run(&app, input).await,
            Err(UseCaseError::UseCase(
                get_project_registration_form_answer_shared_file::Error::InvalidSharing
            ))
        ));
    }

    // Checks that the general user cannot read others' file which is shared to a registration form answer from
    // an owning project but expired.
    #[tokio::test]
    async fn test_general_owner_expired() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let operator = test::model::new_operator_user();

        let (other_file, other_object) = test::model::new_file(other.id().clone());
        let registration_form = test::model::new_registration_form(operator.id().clone());
        let project = test::model::new_general_project(user.id().clone());
        let answer = test::model::new_registration_form_answer_with_project(
            user.id().clone(),
            project.id(),
            &registration_form,
        );

        let sharing = test::model::new_expired_file_sharing(
            other_file.id,
            file_sharing::FileSharingScope::RegistrationFormAnswer(
                registration_form_answer::RegistrationFormAnswerRespondent::Project(project.id()),
                answer.registration_form_id(),
            ),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer])
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_registration_form_answer_shared_file::Input {
            project_id: ProjectId::from_entity(project.id()),
            registration_form_id: RegistrationFormId::from_entity(registration_form.id),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_registration_form_answer_shared_file::run(&app, input).await,
            Err(UseCaseError::UseCase(
                get_project_registration_form_answer_shared_file::Error::InvalidSharing
            ))
        ));
    }
}
