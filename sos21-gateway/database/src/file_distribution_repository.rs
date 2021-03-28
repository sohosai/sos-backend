use anyhow::Result;
use futures::{
    future::{self, TryFutureExt},
    lock::Mutex,
    stream::TryStreamExt,
};
use ref_cast::RefCast;
use sos21_database::{command, model as data, query};
use sos21_domain::context::FileDistributionRepository;
use sos21_domain::model::{
    date_time::DateTime,
    file_distribution::{
        FileDistribution, FileDistributionDescription, FileDistributionFiles, FileDistributionId,
        FileDistributionName,
    },
    file_sharing::FileSharingId,
    project::ProjectId,
    user::UserId,
};
use sqlx::{Postgres, Transaction};

#[derive(Debug, RefCast)]
#[repr(transparent)]
pub struct FileDistributionDatabase(Mutex<Transaction<'static, Postgres>>);

#[async_trait::async_trait]
impl FileDistributionRepository for FileDistributionDatabase {
    async fn store_file_distribution(&self, distribution: FileDistribution) -> Result<()> {
        let mut lock = self.0.lock().await;

        let distribution_id = distribution.id.to_uuid();
        if query::find_file_distribution(&mut *lock, distribution_id)
            .await?
            .is_some()
        {
            let files = from_file_distribution_files(&distribution.files);
            command::delete_file_distribution_files(&mut *lock, distribution_id).await?;
            command::insert_file_distribution_files(&mut *lock, distribution_id, files).await?;

            let distribution = from_file_distribution(distribution);
            let input = command::update_file_distribution::Input {
                id: distribution.id,
                name: distribution.name,
                description: distribution.description,
            };
            command::update_file_distribution(&mut *lock, input).await?;
        } else {
            let files = from_file_distribution_files(&distribution.files);
            let distribution = from_file_distribution(distribution);

            command::insert_file_distribution(&mut *lock, distribution).await?;
            command::insert_file_distribution_files(&mut *lock, distribution_id, files).await?;
        }

        Ok(())
    }

    async fn get_file_distribution(
        &self,
        id: FileDistributionId,
    ) -> Result<Option<FileDistribution>> {
        let mut lock = self.0.lock().await;
        query::find_file_distribution(&mut *lock, id.to_uuid())
            .and_then(|data| future::ready(data.map(to_file_distribution).transpose()))
            .await
    }

    async fn list_file_distributions(&self) -> Result<Vec<FileDistribution>> {
        let mut lock = self.0.lock().await;
        query::list_file_distributions(&mut *lock)
            .and_then(|data| future::ready(to_file_distribution(data)))
            .try_collect()
            .await
    }

    async fn list_file_distributions_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<FileDistribution>> {
        let mut lock = self.0.lock().await;
        query::list_file_distributions_by_project(&mut *lock, project_id.to_uuid())
            .and_then(|data| future::ready(to_file_distribution(data)))
            .try_collect()
            .await
    }
}

fn from_file_distribution_files(
    files: &FileDistributionFiles,
) -> Vec<data::file_distribution::FileDistributionFile> {
    files
        .sharings()
        .map(
            |(project_id, sharing_id)| data::file_distribution::FileDistributionFile {
                project_id: project_id.to_uuid(),
                sharing_id: sharing_id.to_uuid(),
            },
        )
        .collect()
}

fn from_file_distribution(
    distribution: FileDistribution,
) -> data::file_distribution::FileDistribution {
    data::file_distribution::FileDistribution {
        id: distribution.id.to_uuid(),
        created_at: distribution.created_at.utc(),
        author_id: distribution.author_id.0,
        name: distribution.name.into_string(),
        description: distribution.description.into_string(),
    }
}

fn to_file_distribution(
    distribution: data::file_distribution::FileDistributionData,
) -> Result<FileDistribution> {
    let files = FileDistributionFiles::from_sharings(distribution.files.into_iter().map(|file| {
        (
            ProjectId::from_uuid(file.project_id),
            FileSharingId::from_uuid(file.sharing_id),
        )
    }))?;
    let distribution = distribution.distribution;

    Ok(FileDistribution {
        id: FileDistributionId::from_uuid(distribution.id),
        created_at: DateTime::from_utc(distribution.created_at),
        author_id: UserId(distribution.author_id),
        name: FileDistributionName::from_string(distribution.name)?,
        description: FileDistributionDescription::from_string(distribution.description)?,
        files,
    })
}
