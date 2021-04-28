use crate::error::{UseCaseError, UseCaseResult};
use crate::model::file::FileObject;
use crate::model::file_sharing::FileSharingId;
use crate::model::project::ProjectId;

use anyhow::Context;
use sos21_domain::context::{FileSharingRepository, Login, ObjectRepository, ProjectRepository};
use sos21_domain::model::file_sharing;

#[derive(Debug, Clone)]
pub struct Input {
    pub project_id: ProjectId,
    pub sharing_id: FileSharingId,
}

#[derive(Debug, Clone)]
pub enum Error {
    ProjectNotFound,
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
    C: ProjectRepository + FileSharingRepository + ObjectRepository + Send + Sync,
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

    let result = ctx
        .get_file_sharing(input.sharing_id.into_entity())
        .await
        .context("Failed to get a file sharing")?;
    let (sharing, file) = match result {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::FileSharingNotFound)),
    };

    let witness = sharing
        .to_witness_with_project(&project)
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
    use crate::model::project::ProjectId;
    use crate::{get_project_shared_file_object, UseCaseError};

    use sos21_domain::model::file_sharing;
    use sos21_domain::test;

    // Checks that the normal user cannot read others' file which is shared to others' project.
    #[tokio::test]
    async fn test_general_others_project() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let other_project = test::model::new_general_project(other.id.clone());
        let (other_file, other_object) = test::model::new_file(other.id.clone());

        let sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::Project(other_project.id()),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![other_project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_shared_file_object::Input {
            project_id: ProjectId::from_entity(other_project.id()),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_shared_file_object::run(&app, input).await,
            Err(UseCaseError::UseCase(
                get_project_shared_file_object::Error::ProjectNotFound
            ))
        ));
    }

    // Checks that the normal user can read others' file which is shared to owning project.
    #[tokio::test]
    async fn test_general_owner_project() {
        let user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id.clone());
        let other = test::model::new_general_user();
        let (other_file, other_object) = test::model::new_file(other.id.clone());

        let sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::Project(project.id()),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_shared_file_object::Input {
            project_id: ProjectId::from_entity(project.id()),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_shared_file_object::run(&app, input).await,
            Ok(object)
            if object.file.id == FileId::from_entity(other_file.id)
        ));
    }

    // Checks that the normal user can read others' file which is shared to owning project.
    #[tokio::test]
    async fn test_general_subowner_project() {
        let owner = test::model::new_general_user();
        let user = test::model::new_general_user();
        let project =
            test::model::new_general_project_with_subowner(owner.id.clone(), user.id.clone());
        let other = test::model::new_general_user();
        let (other_file, other_object) = test::model::new_file(other.id.clone());

        let sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::Project(project.id()),
        );

        let app = test::build_mock_app()
            .users(vec![owner.clone(), user.clone(), other.clone()])
            .projects(vec![project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_shared_file_object::Input {
            project_id: ProjectId::from_entity(project.id()),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_shared_file_object::run(&app, input).await,
            Ok(object)
            if object.file.id == FileId::from_entity(other_file.id)
        ));
    }

    // Checks that the normal user cannot read others' file which is not shared to the owning project using it.
    #[tokio::test]
    async fn test_general_others_owner() {
        let user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id.clone());
        let other = test::model::new_general_user();
        let other_project = test::model::new_general_project(other.id.clone());
        let (other_file, other_object) = test::model::new_file(other.id.clone());

        let sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::Project(other_project.id()),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project.clone(), other_project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_shared_file_object::Input {
            project_id: ProjectId::from_entity(project.id()),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_shared_file_object::run(&app, input).await,
            Err(UseCaseError::UseCase(
                get_project_shared_file_object::Error::FileSharingNotFound
            ))
        ));
    }

    // Checks that the normal user cannot read others' file which is shared to owning project but
    // revoked.
    #[tokio::test]
    async fn test_general_owner_revoked() {
        let user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id.clone());
        let other = test::model::new_general_user();
        let (other_file, other_object) = test::model::new_file(other.id.clone());

        let mut sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::Project(project.id()),
        );
        sharing.revoke().unwrap();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_shared_file_object::Input {
            project_id: ProjectId::from_entity(project.id()),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_shared_file_object::run(&app, input).await,
            Err(UseCaseError::UseCase(
                get_project_shared_file_object::Error::InvalidSharing
            ))
        ));
    }

    // Checks that the normal user cannot read others' file which is shared to owning project but
    // expired.
    #[tokio::test]
    async fn test_general_owner_expired() {
        let user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id.clone());
        let other = test::model::new_general_user();
        let (other_file, other_object) = test::model::new_file(other.id.clone());

        let sharing = test::model::new_expired_file_sharing(
            other_file.id,
            file_sharing::FileSharingScope::Project(project.id()),
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_project_shared_file_object::Input {
            project_id: ProjectId::from_entity(project.id()),
            sharing_id: FileSharingId::from_entity(sharing.id()),
        };
        assert!(matches!(
            get_project_shared_file_object::run(&app, input).await,
            Err(UseCaseError::UseCase(
                get_project_shared_file_object::Error::InvalidSharing
            ))
        ));
    }
}
