use crate::error::{UseCaseError, UseCaseResult};
use crate::model::file::FileId;
use crate::model::file_distribution::FileDistribution;
use crate::model::file_sharing::FileSharingId;
use crate::model::project::ProjectId;

use anyhow::Context;
use sos21_domain::context::{
    FileDistributionRepository, FileRepository, FileSharingRepository, Login, ProjectRepository,
};
use sos21_domain::model::{
    date_time::DateTime, file, file_distribution, file_sharing, permissions::Permissions, project,
    user,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub name: String,
    pub description: String,
    pub files: Vec<InputFileMapping>,
}

#[derive(Debug, Clone)]
pub struct InputFileMapping {
    pub project_id: ProjectId,
    pub file: InputFile,
}

#[derive(Debug, Clone)]
pub enum InputFile {
    File(FileId),
    Sharing(FileSharingId),
}

#[derive(Debug, Clone)]
pub enum Error {
    InsufficientPermissions,
    InvalidName,
    InvalidDescription,
    NonSharableFile,
    NoFiles,
    TooManyFiles,
    DuplicatedProjectId(ProjectId),
    ProjectNotFound,
    FileNotFound,
    FileSharingNotFound,
    OutOfScopeFileSharing,
}

impl Error {
    fn from_name_error(_err: file_distribution::name::NameError) -> Self {
        Error::InvalidName
    }

    fn from_description_error(_err: file_distribution::description::DescriptionError) -> Self {
        Error::InvalidDescription
    }

    fn from_share_error(_err: file::NonSharableFileError) -> Self {
        Error::NonSharableFile
    }

    fn from_files_error(err: file_distribution::files::FromSharingsError) -> Self {
        match err.kind() {
            file_distribution::files::FromSharingsErrorKind::Empty => Error::NoFiles,
            file_distribution::files::FromSharingsErrorKind::TooLong => Error::TooManyFiles,
            file_distribution::files::FromSharingsErrorKind::Duplicated(project_id) => {
                Error::DuplicatedProjectId(ProjectId::from_entity(project_id))
            }
        }
    }

    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        Error::InsufficientPermissions
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<FileDistribution, Error>
where
    C: FileDistributionRepository
        + FileRepository
        + FileSharingRepository
        + ProjectRepository
        + Send
        + Sync,
{
    let login_user = ctx.login_user();

    login_user
        .require_permissions(Permissions::DISTRIBUTE_FILES)
        .map_err(|err| UseCaseError::UseCase(Error::from_permissions_error(err)))?;

    let name = file_distribution::FileDistributionName::from_string(input.name)
        .map_err(|err| UseCaseError::UseCase(Error::from_name_error(err)))?;

    let description =
        file_distribution::FileDistributionDescription::from_string(input.description)
            .map_err(|err| UseCaseError::UseCase(Error::from_description_error(err)))?;

    let mut sharings = Vec::new();
    for input in input.files {
        let result = ctx
            .get_project(input.project_id.into_entity())
            .await
            .context("Failed to get a project")?;
        let project = match result {
            Some(result) if result.project.is_visible_to(login_user) => result.project,
            _ => return Err(UseCaseError::UseCase(Error::ProjectNotFound)),
        };

        let sharing = to_file_sharing(ctx, &project, input.file).await?;
        sharings.push((project.id, sharing.id()));
    }
    let files = file_distribution::FileDistributionFiles::from_sharings(sharings)
        .map_err(|err| UseCaseError::UseCase(Error::from_files_error(err)))?;

    let distribution = file_distribution::FileDistribution {
        id: file_distribution::FileDistributionId::from_uuid(Uuid::new_v4()),
        created_at: DateTime::now(),
        author_id: login_user.id.clone(),
        name,
        description,
        files,
    };

    ctx.store_file_distribution(distribution.clone())
        .await
        .context("Failed to store a file distribution")?;
    use_case_ensure!(distribution.is_visible_to(login_user));
    Ok(FileDistribution::from_entity(distribution))
}

async fn to_file_sharing<C>(
    ctx: &Login<C>,
    project: &project::Project,
    file: InputFile,
) -> UseCaseResult<file_sharing::FileSharing, Error>
where
    C: FileRepository + FileSharingRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    match file {
        InputFile::File(file_id) => {
            let result = ctx
                .get_file(file_id.into_entity())
                .await
                .context("Failed to get a file")?;
            let file = match result {
                Some(file) if file.is_visible_to(login_user) => file,
                _ => return Err(UseCaseError::UseCase(Error::FileNotFound)),
            };

            let scope = file_sharing::FileSharingScope::Project(project.id);
            let sharing = file
                .share_by(login_user, scope)
                .map_err(|err| UseCaseError::UseCase(Error::from_share_error(err)))?;

            ctx.store_file_sharing(sharing.clone())
                .await
                .context("Failed to store a file sharing")?;

            use_case_ensure!(sharing.scope().contains_project(project));
            Ok(sharing)
        }
        InputFile::Sharing(sharing_id) => {
            let result = ctx
                .get_file_sharing(sharing_id.into_entity())
                .await
                .context("Failed to get a file sharing")?;
            let sharing = match result {
                Some((sharing, file)) if sharing.is_visible_to_with_file(login_user, &file) => {
                    sharing
                }
                _ => return Err(UseCaseError::UseCase(Error::FileSharingNotFound)),
            };

            if !sharing.scope().contains_project(project) {
                return Err(UseCaseError::UseCase(Error::OutOfScopeFileSharing));
            }

            Ok(sharing)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::file::FileId;
    use crate::model::file_sharing::FileSharingId;
    use crate::model::project::ProjectId;
    use crate::{distribute_files, get_distributed_file, get_project_shared_file, UseCaseError};

    use sos21_domain::model::file_sharing;
    use sos21_domain::test;

    fn mock_input(files: Vec<(ProjectId, distribute_files::InputFile)>) -> distribute_files::Input {
        let name = test::model::mock_file_distribution_name().into_string();
        let description = test::model::mock_file_distribution_description().into_string();
        distribute_files::Input {
            name,
            description,
            files: files
                .into_iter()
                .map(|(project_id, file)| distribute_files::InputFileMapping { project_id, file })
                .collect(),
        }
    }

    // Checks that the normal user cannot distribute files.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id.clone());
        let (file, object) = test::model::new_file(user.id.clone());
        let scope = file_sharing::FileSharingScope::Project(project.id);
        let sharing = file_sharing::FileSharing::new(file.id, scope);

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = mock_input(vec![(
            ProjectId::from_entity(project.id),
            distribute_files::InputFile::Sharing(FileSharingId::from_entity(sharing.id())),
        )]);
        assert!(matches!(
            distribute_files::run(&app, input).await,
            Err(UseCaseError::UseCase(
                distribute_files::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the committee user cannot distribute files.
    #[tokio::test]
    async fn test_committee() {
        let user = test::model::new_committee_user();
        let project = test::model::new_general_project(user.id.clone());
        let (file, object) = test::model::new_file(user.id.clone());
        let scope = file_sharing::FileSharingScope::Project(project.id);
        let sharing = file_sharing::FileSharing::new(file.id, scope);

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = mock_input(vec![(
            ProjectId::from_entity(project.id),
            distribute_files::InputFile::Sharing(FileSharingId::from_entity(sharing.id())),
        )]);
        assert!(matches!(
            distribute_files::run(&app, input).await,
            Err(UseCaseError::UseCase(
                distribute_files::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the operator user can distribute files.
    #[tokio::test]
    async fn test_operator() {
        let user = test::model::new_operator_user();
        let (file, object) = test::model::new_file(user.id.clone());

        let other = test::model::new_general_user();
        let other_project = test::model::new_general_project(other.id.clone());

        let scope = file_sharing::FileSharingScope::Project(other_project.id);
        let sharing = file_sharing::FileSharing::new(file.id, scope);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![other_project.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .sharings(vec![sharing.clone()])
            .build();

        let user_app = app.clone().login_as(user.clone()).await;

        let other_project_id = ProjectId::from_entity(other_project.id);
        let sharing_id = FileSharingId::from_entity(sharing.id());
        let input = mock_input(vec![(
            other_project_id,
            distribute_files::InputFile::Sharing(sharing_id),
        )]);
        let distribution = distribute_files::run(&user_app, input).await.unwrap();
        assert!(distribution
            .files
            .iter()
            .find(|mapping| mapping.project_id == ProjectId::from_entity(other_project.id))
            .is_some());

        let other_app = app.clone().login_as(other.clone()).await;

        let input = get_distributed_file::Input {
            project_id: other_project_id,
            distribution_id: distribution.id,
        };
        assert!(matches!(
            get_distributed_file::run(&other_app, input).await,
            Ok(got)
            if got.sharing_id == sharing_id
        ));
    }

    // Checks that the operator user cannot distribute files using sharing with unexpected scope.
    #[tokio::test]
    async fn test_invalid_scope_operator() {
        let user = test::model::new_operator_user();
        let (file, object) = test::model::new_file(user.id.clone());

        let other = test::model::new_general_user();
        let other_project = test::model::new_general_project(other.id.clone());

        let sharing =
            file_sharing::FileSharing::new(file.id, file_sharing::FileSharingScope::Committee);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![other_project.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = mock_input(vec![(
            ProjectId::from_entity(other_project.id),
            distribute_files::InputFile::Sharing(FileSharingId::from_entity(sharing.id())),
        )]);
        assert!(matches!(
            distribute_files::run(&app, input).await,
            Err(UseCaseError::UseCase(
                distribute_files::Error::OutOfScopeFileSharing
            ))
        ));
    }

    // Checks that the operator user can distribute multiple files.
    #[tokio::test]
    async fn test_multi_operator() {
        use std::collections::HashSet;

        let user = test::model::new_operator_user();
        let (file1, object1) = test::model::new_file(user.id.clone());
        let (file2, object2) = test::model::new_file(user.id.clone());

        let other1 = test::model::new_general_user();
        let other2 = test::model::new_general_user();
        let other1_project = test::model::new_general_project(other1.id.clone());
        let other2_project = test::model::new_general_project(other2.id.clone());

        let scope1 = file_sharing::FileSharingScope::Project(other1_project.id);
        let sharing1 = file_sharing::FileSharing::new(file1.id, scope1);
        let scope2 = file_sharing::FileSharingScope::Project(other2_project.id);
        let sharing2 = file_sharing::FileSharing::new(file2.id, scope2);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other1.clone(), other2.clone()])
            .projects(vec![other1_project.clone(), other2_project.clone()])
            .files(vec![file1.clone(), file2.clone()])
            .objects(vec![object1, object2])
            .await
            .sharings(vec![sharing1.clone(), sharing2.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = mock_input(vec![
            (
                ProjectId::from_entity(other1_project.id),
                distribute_files::InputFile::Sharing(FileSharingId::from_entity(sharing1.id())),
            ),
            (
                ProjectId::from_entity(other2_project.id),
                distribute_files::InputFile::Sharing(FileSharingId::from_entity(sharing2.id())),
            ),
        ]);
        let distribution = distribute_files::run(&app, input).await.unwrap();
        let got: HashSet<ProjectId> = distribution
            .files
            .iter()
            .map(|mapping| mapping.project_id)
            .collect();
        let expected: HashSet<ProjectId> = vec![other1_project.id, other2_project.id]
            .into_iter()
            .map(ProjectId::from_entity)
            .collect();
        assert_eq!(got, expected);
    }

    // Checks that the operator user can share and distribute files.
    #[tokio::test]
    async fn test_share_operator() {
        let user = test::model::new_operator_user();
        let (file, object) = test::model::new_file(user.id.clone());

        let other = test::model::new_general_user();
        let other_project = test::model::new_general_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![other_project.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .build();

        let user_app = app.clone().login_as(user.clone()).await;

        let other_project_id = ProjectId::from_entity(other_project.id);
        let file_id = FileId::from_entity(file.id);
        let input = mock_input(vec![(
            other_project_id,
            distribute_files::InputFile::File(file_id),
        )]);
        let distribution = distribute_files::run(&user_app, input).await.unwrap();
        assert!(distribution
            .files
            .iter()
            .find(|mapping| mapping.project_id == other_project_id)
            .is_some());

        let other_app = app.clone().login_as(other.clone()).await;

        let input = get_distributed_file::Input {
            project_id: other_project_id,
            distribution_id: distribution.id,
        };
        let distributed_file = get_distributed_file::run(&other_app, input).await.unwrap();

        let input = get_project_shared_file::Input {
            project_id: other_project_id,
            sharing_id: distributed_file.sharing_id,
        };
        assert!(matches!(
            get_project_shared_file::run(&other_app, input).await,
            Ok(got)
            if got.id == file_id
        ));
    }
}
