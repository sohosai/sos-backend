use crate::model::{
    pending_project::{PendingProject, PendingProjectId},
    user::User,
};

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PendingProjectWithOwner {
    pub pending_project: PendingProject,
    pub owner: User,
}

#[async_trait::async_trait]
pub trait PendingProjectRepository {
    async fn store_pending_project(&self, pending_project: PendingProject) -> Result<()>;
    async fn delete_pending_project(&self, id: PendingProjectId) -> Result<()>;
    async fn get_pending_project(
        &self,
        id: PendingProjectId,
    ) -> Result<Option<PendingProjectWithOwner>>;
}

#[macro_export]
macro_rules! delegate_pending_project_repository {
    (impl $(<$($vars:ident $(: $c0:ident $(+ $cs:ident)* )? ),*>)? PendingProjectRepository for $ty:ty {
        $sel:ident $target:block
    }) => {
        #[::async_trait::async_trait]
        impl $(<$($vars$(: $c0 $(+ $cs)* )?,)*>)? $crate::context::PendingProjectRepository for $ty {
            async fn store_pending_project(
                &$sel,
                pending_project: $crate::model::pending_project::PendingProject,
            ) -> ::anyhow::Result<()> {
                $target.store_pending_project(pending_project).await
            }
            async fn delete_pending_project(
                &$sel,
                id: $crate::model::pending_project::PendingProjectId,
            ) -> ::anyhow::Result<()> {
                $target.delete_pending_project(id).await
            }
            async fn get_pending_project(
                &$sel,
                id: $crate::model::pending_project::PendingProjectId,
            ) -> ::anyhow::Result<
                Option<$crate::context::pending_project_repository::PendingProjectWithOwner>,
            > {
                $target.get_pending_project(id).await
            }
        }
    }
}

#[async_trait::async_trait]
impl<C: PendingProjectRepository + Sync> PendingProjectRepository for &C {
    async fn store_pending_project(&self, pending_project: PendingProject) -> Result<()> {
        <C as PendingProjectRepository>::store_pending_project(self, pending_project).await
    }

    async fn delete_pending_project(&self, id: PendingProjectId) -> Result<()> {
        <C as PendingProjectRepository>::delete_pending_project(self, id).await
    }

    async fn get_pending_project(
        &self,
        id: PendingProjectId,
    ) -> Result<Option<PendingProjectWithOwner>> {
        <C as PendingProjectRepository>::get_pending_project(self, id).await
    }
}
