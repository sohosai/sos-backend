use crate::model::{
    pending_project::{PendingProject, PendingProjectId},
    user::UserId,
};

use anyhow::Result;

#[async_trait::async_trait]
pub trait PendingProjectRepository {
    async fn store_pending_project(&self, pending_project: PendingProject) -> Result<()>;
    async fn get_pending_project(&self, id: PendingProjectId) -> Result<Option<PendingProject>>;
    async fn list_pending_projects_by_user(&self, user_id: UserId) -> Result<Vec<PendingProject>>;
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
            async fn get_pending_project(
                &$sel,
                id: $crate::model::pending_project::PendingProjectId,
            ) -> ::anyhow::Result<
                Option<
                    $crate::model::pending_project::PendingProject,
                >,
            > {
                $target.get_pending_project(id).await
            }
            async fn list_pending_projects_by_user(
                &$sel,
                user_id: $crate::model::user::UserId,
            ) -> ::anyhow::Result<
                Vec<
                    $crate::model::pending_project::PendingProject,
                >,
            > {
                $target.list_pending_projects_by_user(user_id).await
            }
        }
    }
}

#[async_trait::async_trait]
impl<C: PendingProjectRepository + Sync> PendingProjectRepository for &C {
    async fn store_pending_project(&self, pending_project: PendingProject) -> Result<()> {
        <C as PendingProjectRepository>::store_pending_project(self, pending_project).await
    }

    async fn get_pending_project(&self, id: PendingProjectId) -> Result<Option<PendingProject>> {
        <C as PendingProjectRepository>::get_pending_project(self, id).await
    }

    async fn list_pending_projects_by_user(&self, user_id: UserId) -> Result<Vec<PendingProject>> {
        <C as PendingProjectRepository>::list_pending_projects_by_user(self, user_id).await
    }
}
