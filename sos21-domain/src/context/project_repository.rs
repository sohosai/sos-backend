use crate::model::{
    project::{Project, ProjectId, ProjectIndex},
    user::{User, UserId},
};

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ProjectWithOwners {
    pub project: Project,
    pub owner: User,
    pub subowner: User,
}

#[async_trait::async_trait]
pub trait ProjectRepository {
    async fn store_project(&self, project: Project) -> Result<()>;
    async fn get_project(&self, id: ProjectId) -> Result<Option<ProjectWithOwners>>;
    async fn get_project_by_index(&self, index: ProjectIndex) -> Result<Option<ProjectWithOwners>>;
    async fn count_projects(&self) -> Result<u64>;
    async fn list_projects(&self) -> Result<Vec<ProjectWithOwners>>;
    async fn list_projects_by_user(&self, user_id: UserId) -> Result<Vec<ProjectWithOwners>>;
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
                Option<$crate::context::project_repository::ProjectWithOwners>,
            > {
                $target.get_project(id).await
            }
            async fn get_project_by_index(
                &$sel,
                index: $crate::model::project::ProjectIndex,
            ) -> ::anyhow::Result<
                Option<$crate::context::project_repository::ProjectWithOwners>,
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
                Vec<$crate::context::project_repository::ProjectWithOwners>,
            > {
                $target.list_projects().await
            }
            async fn list_projects_by_user(
                &$sel,
                user_id: $crate::model::user::UserId,
            ) -> ::anyhow::Result<Vec<$crate::context::project_repository::ProjectWithOwners>> {
                $target.list_projects_by_user(user_id).await
            }
        }
    };
}

#[async_trait::async_trait]
impl<C: ProjectRepository + Sync> ProjectRepository for &C {
    async fn store_project(&self, project: Project) -> Result<()> {
        <C as ProjectRepository>::store_project(self, project).await
    }

    async fn get_project(&self, id: ProjectId) -> Result<Option<ProjectWithOwners>> {
        <C as ProjectRepository>::get_project(self, id).await
    }

    async fn get_project_by_index(&self, index: ProjectIndex) -> Result<Option<ProjectWithOwners>> {
        <C as ProjectRepository>::get_project_by_index(self, index).await
    }

    async fn count_projects(&self) -> Result<u64> {
        <C as ProjectRepository>::count_projects(self).await
    }

    async fn list_projects(&self) -> Result<Vec<ProjectWithOwners>> {
        <C as ProjectRepository>::list_projects(self).await
    }

    async fn list_projects_by_user(&self, user_id: UserId) -> Result<Vec<ProjectWithOwners>> {
        <C as ProjectRepository>::list_projects_by_user(self, user_id).await
    }
}
