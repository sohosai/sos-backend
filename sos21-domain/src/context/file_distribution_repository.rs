use crate::model::file_distribution::{FileDistribution, FileDistributionId};
use crate::model::project::ProjectId;

use anyhow::Result;

#[async_trait::async_trait]
pub trait FileDistributionRepository {
    async fn store_file_distribution(&self, distribution: FileDistribution) -> Result<()>;
    async fn get_file_distribution(
        &self,
        id: FileDistributionId,
    ) -> Result<Option<FileDistribution>>;
    async fn list_file_distributions(&self) -> Result<Vec<FileDistribution>>;
    async fn list_file_distributions_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<FileDistribution>>;
}

#[macro_export]
macro_rules! delegate_file_distribution_repository {
    (impl $(<$($vars:ident $(: $c0:ident $(+ $cs:ident)* )? ),*>)? FileDistributionRepository for $ty:ty {
        $sel:ident $target:block
    }) => {
        #[::async_trait::async_trait]
        impl $(<$($vars$(: $c0 $(+ $cs)* )?,)*>)? $crate::context::FileDistributionRepository for $ty {
            async fn store_file_distribution(
                &$sel,
                distribution: $crate::model::file_distribution::FileDistribution
            ) -> ::anyhow::Result<()> {
                $target.store_file_distribution(distribution).await
            }
            async fn get_file_distribution(
                &$sel,
                id: $crate::model::file_distribution::FileDistributionId
            ) -> ::anyhow::Result<Option<
                $crate::model::file_distribution::FileDistribution,
            >> {
                $target.get_file_distribution(id).await
            }
            async fn list_file_distributions(
                &$sel,
            ) -> ::anyhow::Result<Vec<
                $crate::model::file_distribution::FileDistribution,
            >> {
                $target.list_file_distributions().await
            }
            async fn list_file_distributions_by_project(
                &$sel,
                project_id: $crate::model::project::ProjectId
            ) -> ::anyhow::Result<Vec<
                $crate::model::file_distribution::FileDistribution,
            >> {
                $target.list_file_distributions_by_project(project_id).await
            }
        }
    }
}

#[async_trait::async_trait]
impl<C: FileDistributionRepository + Sync> FileDistributionRepository for &C {
    async fn store_file_distribution(&self, distribution: FileDistribution) -> Result<()> {
        <C as FileDistributionRepository>::store_file_distribution(self, distribution).await
    }

    async fn get_file_distribution(
        &self,
        id: FileDistributionId,
    ) -> Result<Option<FileDistribution>> {
        <C as FileDistributionRepository>::get_file_distribution(self, id).await
    }

    async fn list_file_distributions(&self) -> Result<Vec<FileDistribution>> {
        <C as FileDistributionRepository>::list_file_distributions(self).await
    }

    async fn list_file_distributions_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<FileDistribution>> {
        <C as FileDistributionRepository>::list_file_distributions_by_project(self, project_id)
            .await
    }
}
