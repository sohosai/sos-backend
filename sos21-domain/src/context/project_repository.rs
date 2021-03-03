use crate::model::{
    project::{Project, ProjectId, ProjectIndex},
    user::{User, UserId},
};

use anyhow::Result;

#[async_trait::async_trait]
pub trait ProjectRepository {
    async fn store_project(&self, project: Project) -> Result<()>;
    async fn get_project(&self, id: ProjectId) -> Result<Option<(Project, User)>>;
    async fn get_project_by_index(&self, index: ProjectIndex) -> Result<Option<(Project, User)>>;
    async fn count_projects(&self) -> Result<u64>;
    async fn list_projects(&self) -> Result<Vec<(Project, User)>>;
    async fn list_projects_by_owner(&self, id: UserId) -> Result<Vec<Project>>;
}

#[macro_export]
macro_rules! delegate_project_repository {
    (impl $(<$($vars:ident $(: $c0:ident $(+ $cs:ident)* )? ),*>)? ProjectRepository for $ty:ty {
        $sel:ident $target:block
    }) => {
        #[::async_trait::async_trait]
        impl $(<$($vars$(: $c0 $(+ $cs)* )?,)*>)? $crate::context::ProjectRepository for $ty {
            async fn store_project(
                &$sel,
                project: $crate::model::project::Project,
            ) -> ::anyhow::Result<()> {
                $target.store_project(project).await
            }
            async fn get_project(
                &$sel,
                id: $crate::model::project::ProjectId,
            ) -> ::anyhow::Result<
                Option<(
                    $crate::model::project::Project,
                    $crate::model::user::User,
                )>,
            > {
                $target.get_project(id).await
            }
            async fn get_project_by_index(
                &$sel,
                index: $crate::model::project::ProjectIndex,
            ) -> ::anyhow::Result<
                Option<(
                    $crate::model::project::Project,
                    $crate::model::user::User,
                )>,
            > {
                $target.get_project_by_index(index).await
            }
            async fn count_projects(
                &$sel,
            ) -> ::anyhow::Result<u64> {
                $target.count_projects().await
            }
            async fn list_projects(
                &$sel,
            ) -> ::anyhow::Result<
                Vec<(
                    $crate::model::project::Project,
                    $crate::model::user::User,
                )>,
            > {
                $target.list_projects().await
            }
            async fn list_projects_by_owner(
                &$sel,
                id: $crate::model::user::UserId,
            ) -> ::anyhow::Result<Vec<$crate::model::project::Project>> {
                $target.list_projects_by_owner(id).await
            }
        }
    };
}

#[async_trait::async_trait]
impl<C: ProjectRepository + Sync> ProjectRepository for &C {
    async fn store_project(&self, project: Project) -> Result<()> {
        <C as ProjectRepository>::store_project(self, project).await
    }

    async fn get_project(&self, id: ProjectId) -> Result<Option<(Project, User)>> {
        <C as ProjectRepository>::get_project(self, id).await
    }

    async fn get_project_by_index(&self, index: ProjectIndex) -> Result<Option<(Project, User)>> {
        <C as ProjectRepository>::get_project_by_index(self, index).await
    }

    async fn count_projects(&self) -> Result<u64> {
        <C as ProjectRepository>::count_projects(self).await
    }

    async fn list_projects(&self) -> Result<Vec<(Project, User)>> {
        <C as ProjectRepository>::list_projects(self).await
    }

    async fn list_projects_by_owner(&self, id: UserId) -> Result<Vec<Project>> {
        <C as ProjectRepository>::list_projects_by_owner(self, id).await
    }
}
