use crate::error::{UseCaseError, UseCaseResult};
use crate::model::file::File;
use crate::model::file_sharing::FileSharingId;
use crate::model::form::FormId;
use crate::model::project::ProjectId;

use anyhow::Context;
use sos21_domain::context::{
    FileSharingRepository, FormAnswerRepository, FormRepository, Login, ProjectRepository,
};
use sos21_domain::model::file_sharing;

#[derive(Debug, Clone)]
pub struct Input {
    pub project_id: ProjectId,
    pub form_id: FormId,
    pub sharing_id: FileSharingId,
}

#[derive(Debug, Clone)]
pub enum Error {
    ProjectNotFound,
    FormNotFound,
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
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<File, Error>
where
    C: ProjectRepository
        + FormAnswerRepository
        + FormRepository
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

    let form_id = input.form_id.into_entity();
    let result = ctx
        .get_form_answer_by_form_and_project(form_id, project.id())
        .await
        .context("Failed to get a form answer")?;
    let answer = match result {
        Some(answer) if answer.is_visible_to_with_project(login_user, &project) => answer,
        _ => {
            if let Some(form) = ctx.get_form(form_id).await? {
                if form.is_visible_to(login_user) {
                    return Err(UseCaseError::UseCase(Error::FormAnswerNotFound));
                }
            }
            return Err(UseCaseError::UseCase(Error::FormNotFound));
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
        .to_witness_with_form_answer(&answer)
        .map_err(|err| UseCaseError::UseCase(Error::from_witness_error(err)))?;

    use_case_ensure!(file.is_visible_to_with_sharing(&witness));
    Ok(File::from_entity(file))
}

#[cfg(test)]
mod tests {
    use crate::model::file::FileId;
    use crate::model::file_sharing::FileSharingId;
    use crate::model::form::FormId;
    use crate::model::project::ProjectId;
    use crate::{get_project_form_answer_shared_file, UseCaseError};

    use sos21_domain::model::file_sharing;
    use sos21_domain::test;

    // Checks that the general user cannot read others' file which is shared to other's form answer.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let operator = test::model::new_operator_user();

        let (other_file, other_object) = test::model::new_file(other.id().clone());
        let form = test::model::new_form(operator.id().clone());
        let other_project = test::model::new_general_project(other.id().clone());
        let other_answer = test::model::new_form_answer(other.id().clone(), &other_project, &form);

        let sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::form_answer_scope(&other_answer),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone(), operator.clone()])
            .projects(vec![other_project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .forms(vec![form.clone()])
            .answers(vec![other_answer])
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_form_answer_shared_file::Input {
            project_id: ProjectId::from_entity(other_project.id()),
            form_id: FormId::from_entity(form.id()),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_form_answer_shared_file::run(&app, input).await,
            Err(UseCaseError::UseCase(_))
        ));
    }

    // Checks that the general user can read others' file which is shared to a form answer from
    // an owning project.
    #[tokio::test]
    async fn test_general_owner_get() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let operator = test::model::new_operator_user();

        let (other_file, other_object) = test::model::new_file(other.id().clone());
        let form = test::model::new_form(operator.id().clone());
        let project = test::model::new_general_project(user.id().clone());
        let answer = test::model::new_form_answer(user.id().clone(), &project, &form);

        let sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::form_answer_scope(&answer),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .forms(vec![form.clone()])
            .answers(vec![answer])
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_form_answer_shared_file::Input {
            project_id: ProjectId::from_entity(project.id()),
            form_id: FormId::from_entity(form.id()),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_form_answer_shared_file::run(&app, input).await,
            Ok(file)
            if file.id == FileId::from_entity(other_file.id)
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
        let form = test::model::new_form(operator.id().clone());
        let project = test::model::new_general_project(user.id().clone());
        let answer = test::model::new_form_answer(user.id().clone(), &project, &form);
        let other_project = test::model::new_general_project(other.id().clone());
        let other_answer = test::model::new_form_answer(other.id().clone(), &other_project, &form);

        let sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::form_answer_scope(&other_answer),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone(), operator.clone()])
            .projects(vec![project.clone(), other_project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .forms(vec![form.clone()])
            .answers(vec![answer, other_answer])
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_form_answer_shared_file::Input {
            project_id: ProjectId::from_entity(project.id()),
            form_id: FormId::from_entity(form.id()),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_form_answer_shared_file::run(&app, input).await,
            Err(UseCaseError::UseCase(
                get_project_form_answer_shared_file::Error::FileSharingNotFound
            ))
        ));
    }

    // Checks that the general user cannot read others' file which is shared to a form answer from
    // an owning project but revoked.
    #[tokio::test]
    async fn test_general_owner_revoked() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let operator = test::model::new_operator_user();

        let (other_file, other_object) = test::model::new_file(other.id().clone());
        let form = test::model::new_form(operator.id().clone());
        let project = test::model::new_general_project(user.id().clone());
        let answer = test::model::new_form_answer(user.id().clone(), &project, &form);

        let mut sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::form_answer_scope(&answer),
        );

        sharing.revoke().unwrap();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .forms(vec![form.clone()])
            .answers(vec![answer])
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_form_answer_shared_file::Input {
            project_id: ProjectId::from_entity(project.id()),
            form_id: FormId::from_entity(form.id()),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_form_answer_shared_file::run(&app, input).await,
            Err(UseCaseError::UseCase(
                get_project_form_answer_shared_file::Error::InvalidSharing
            ))
        ));
    }

    // Checks that the general user cannot read others' file which is shared to a form answer from
    // an owning project but expired.
    #[tokio::test]
    async fn test_general_owner_expired() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let operator = test::model::new_operator_user();

        let (other_file, other_object) = test::model::new_file(other.id().clone());
        let form = test::model::new_form(operator.id().clone());
        let project = test::model::new_general_project(user.id().clone());
        let answer = test::model::new_form_answer(user.id().clone(), &project, &form);

        let sharing = test::model::new_expired_file_sharing(
            other_file.id,
            file_sharing::FileSharingScope::form_answer_scope(&answer),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .forms(vec![form.clone()])
            .answers(vec![answer])
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_form_answer_shared_file::Input {
            project_id: ProjectId::from_entity(project.id()),
            form_id: FormId::from_entity(form.id()),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_form_answer_shared_file::run(&app, input).await,
            Err(UseCaseError::UseCase(
                get_project_form_answer_shared_file::Error::InvalidSharing
            ))
        ));
    }
}
