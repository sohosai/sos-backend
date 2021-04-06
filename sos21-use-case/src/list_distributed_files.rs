use crate::error::{UseCaseError, UseCaseResult};
use crate::model::file_distribution::FileDistributionDistributedFile;
use crate::model::project::ProjectId;

use anyhow::Context;
use sos21_domain::context::{FileDistributionRepository, Login, ProjectRepository};

#[derive(Debug, Clone)]
pub enum Error {
    ProjectNotFound,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(
    ctx: &Login<C>,
    project_id: ProjectId,
) -> UseCaseResult<Vec<FileDistributionDistributedFile>, Error>
where
    C: ProjectRepository + FileDistributionRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let result = ctx
        .get_project(project_id.into_entity())
        .await
        .context("Failed to get project")?;
    let project = match result {
        Some(result) if result.project.is_visible_to(login_user) => result.project,
        _ => return Err(UseCaseError::UseCase(Error::ProjectNotFound)),
    };

    let distributions = ctx
        .list_file_distributions_by_project(project.id)
        .await
        .context("Failed to list file distributions")?;

    let mut distributed_files = Vec::new();
    for distribution in distributions {
        let distributed_file = distribution
            .get_distributed_file_for(&project)
            .context("Failed to get distributed file from by_project list")?;
        use_case_ensure!(distributed_file.is_visible_to_with_project(login_user, &project));
        distributed_files.push(FileDistributionDistributedFile::from_entity(
            distributed_file,
        ));
    }

    Ok(distributed_files)
}

#[cfg(test)]
mod tests {
    use crate::model::file_distribution::FileDistributionId;
    use crate::model::file_sharing::FileSharingId;
    use crate::model::project::ProjectId;
    use crate::{list_distributed_files, UseCaseError};

    use sos21_domain::model::{file, file_distribution, file_sharing, object, project, user};
    use sos21_domain::test;

    #[tokio::test]
    async fn test_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();

        let other_project = test::model::new_general_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![other_project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            list_distributed_files::run(&app, ProjectId::from_entity(other_project.id)).await,
            Err(UseCaseError::UseCase(
                list_distributed_files::Error::ProjectNotFound
            ))
        ));
    }

    fn mock_distribution(
        author_id: user::UserId,
        project_id: project::ProjectId,
    ) -> (
        file::File,
        object::Object,
        file_sharing::FileSharing,
        file_distribution::FileDistribution,
    ) {
        let (file, object) = test::model::new_file(author_id.clone());
        let sharing = file_sharing::FileSharing::new(
            file.id,
            file_sharing::FileSharingScope::Project(project_id),
        );
        let files = test::model::mock_file_distribution_files_with_project_sharing(&sharing);
        let distribution = test::model::new_file_distribution_with_files(author_id, files);

        (file, object, sharing, distribution)
    }

    #[tokio::test]
    async fn test_owner() {
        use std::collections::HashMap;

        let user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id.clone());

        let other = test::model::new_general_user();
        let other_project = test::model::new_general_project(other.id.clone());

        let operator = test::model::new_operator_user();
        let (file1, object1, sharing1, distribution1) =
            mock_distribution(operator.id.clone(), project.id);
        let (file2, object2, sharing2, distribution2) =
            mock_distribution(operator.id.clone(), project.id);
        let (file3, object3, sharing3, distribution3) =
            mock_distribution(operator.id.clone(), other_project.id);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone(), operator.clone()])
            .projects(vec![project.clone(), other_project.clone()])
            .files(vec![file1.clone(), file2.clone(), file3.clone()])
            .objects(vec![object1, object2, object3])
            .await
            .sharings(vec![sharing1.clone(), sharing2.clone(), sharing3.clone()])
            .distributions(vec![
                distribution1.clone(),
                distribution2.clone(),
                distribution3.clone(),
            ])
            .build()
            .login_as(user.clone())
            .await;

        let got = list_distributed_files::run(&app, ProjectId::from_entity(project.id))
            .await
            .unwrap();
        let got: HashMap<_, _> = got
            .into_iter()
            .map(|distributed_file| {
                (
                    distributed_file.distribution_id,
                    distributed_file.sharing_id,
                )
            })
            .collect();
        let expected: HashMap<_, _> = vec![
            (distribution1.id, sharing1.id()),
            (distribution2.id, sharing2.id()),
        ]
        .into_iter()
        .map(|(distribution_id, sharing_id)| {
            (
                FileDistributionId::from_entity(distribution_id),
                FileSharingId::from_entity(sharing_id),
            )
        })
        .collect();

        assert_eq!(got, expected);
    }
}
