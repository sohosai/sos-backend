use anyhow::Result;
use sos21_domain_model::{
    project::{Project, ProjectId},
    user::{User, UserId},
};

#[async_trait::async_trait]
pub trait ProjectRepository {
    async fn create_project(&self, project: Project) -> Result<()>;
    async fn get_project(&self, id: ProjectId) -> Result<Option<(Project, User)>>;
    async fn list_projects(&self) -> Result<Vec<(Project, User)>>;
    async fn list_projects_by_owner(&self, id: UserId) -> Result<Vec<Project>>;
}

#[macro_export]
macro_rules! delegate_project_repository {
    (impl <$($vars:ident $(: $c0:ident $(+ $cs:ident)* )? ),*> for $t:ty : $field:ident) => {
        #[::async_trait::async_trait]
        impl<$($vars$(: $c0 $(+ $cs)* )?,)*> $crate::ProjectRepository for $t {
            async fn create_project(
                &self,
                project: ::sos21_domain_model::project::Project,
            ) -> ::anyhow::Result<()> {
                self.$field.create_project(project).await
            }
            async fn get_project(
                &self,
                id: ::sos21_domain_model::project::ProjectId,
            ) -> ::anyhow::Result<
                Option<(
                    ::sos21_domain_model::project::Project,
                    ::sos21_domain_model::user::User,
                )>,
            > {
                self.$field.get_project(id).await
            }
            async fn list_projects(
                &self,
            ) -> ::anyhow::Result<
                Vec<(
                    ::sos21_domain_model::project::Project,
                    ::sos21_domain_model::user::User,
                )>,
            > {
                self.$field.list_projects().await
            }
            async fn list_projects_by_owner(
                &self,
                id: ::sos21_domain_model::user::UserId,
            ) -> ::anyhow::Result<Vec<::sos21_domain_model::project::Project>> {
                self.$field.list_projects_by_owner(id).await
            }
        }
    };
}

#[async_trait::async_trait]
impl<C: ProjectRepository + Sync> ProjectRepository for &C {
    async fn create_project(&self, project: Project) -> Result<()> {
        <C as ProjectRepository>::create_project(self, project).await
    }

    async fn get_project(&self, id: ProjectId) -> Result<Option<(Project, User)>> {
        <C as ProjectRepository>::get_project(self, id).await
    }

    async fn list_projects(&self) -> Result<Vec<(Project, User)>> {
        <C as ProjectRepository>::list_projects(self).await
    }

    async fn list_projects_by_owner(&self, id: UserId) -> Result<Vec<Project>> {
        <C as ProjectRepository>::list_projects_by_owner(self, id).await
    }
}
