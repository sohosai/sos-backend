use std::collections::HashSet;

use crate::user_repository::to_user;

use anyhow::{ensure, Result};
use futures::{future, stream::TryStreamExt};
use sos21_database::{command, model as data, query};
use sos21_domain_context::ProjectRepository;
use sos21_domain_model::{
    project::{
        Project, ProjectAttribute, ProjectAttributes, ProjectCategory, ProjectDescription,
        ProjectGroupName, ProjectId, ProjectKanaGroupName, ProjectKanaName, ProjectName,
    },
    user::{User, UserId},
};
use sqlx::postgres::PgPool;

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Database { pool }
    }
}

#[async_trait::async_trait]
impl ProjectRepository for Database {
    async fn create_project(&self, project: Project) -> Result<()> {
        command::insert_project(&self.pool, from_project(project)).await
    }

    async fn get_project(&self, id: ProjectId) -> Result<Option<(Project, User)>> {
        let result = match query::find_project(&self.pool, id.to_uuid()).await? {
            Some(x) => x,
            None => return Ok(None),
        };
        to_project_with_owner(result.project, result.owner).map(Some)
    }

    async fn list_projects(&self) -> Result<Vec<(Project, User)>> {
        query::list_projects(&self.pool)
            .and_then(|result| future::ready(to_project_with_owner(result.project, result.owner)))
            .try_collect()
            .await
    }

    async fn list_projects_by_owner(&self, id: UserId) -> Result<Vec<Project>> {
        query::list_projects_by_owner(&self.pool, id.0)
            .and_then(|project| future::ready(to_project(project)))
            .try_collect()
            .await
    }
}

fn to_project_with_owner(
    project: data::project::Project,
    owner: data::user::User,
) -> Result<(Project, User)> {
    let project = to_project(project)?;
    let owner = to_user(owner)?;
    Ok((project, owner))
}

fn from_project(project: Project) -> data::project::Project {
    let Project {
        id,
        created_at,
        owner_id,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category,
        attributes,
    } = project;
    data::project::Project {
        id: id.to_uuid(),
        created_at,
        owner_id: owner_id.0,
        name: name.into_string(),
        kana_name: kana_name.into_string(),
        group_name: group_name.into_string(),
        kana_group_name: kana_group_name.into_string(),
        description: description.into_string(),
        category: match category {
            ProjectCategory::General => data::project::ProjectCategory::General,
            ProjectCategory::Stage => data::project::ProjectCategory::Stage,
        },
        attributes: attributes
            .attributes()
            .map(|attr| match attr {
                ProjectAttribute::Academic => data::project::ProjectAttributes::ACADEMIC,
                ProjectAttribute::Artistic => data::project::ProjectAttributes::ARTISTIC,
                ProjectAttribute::Committee => data::project::ProjectAttributes::COMMITTEE,
            })
            .collect(),
    }
}

fn to_project(project: data::project::Project) -> Result<Project> {
    let data::project::Project {
        id,
        created_at,
        owner_id,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category,
        mut attributes,
    } = project;

    // TODO: better impl
    let mut attrs = HashSet::new();
    if attributes.contains(data::project::ProjectAttributes::ACADEMIC) {
        attrs.insert(ProjectAttribute::Academic);
        attributes.remove(data::project::ProjectAttributes::ACADEMIC);
    }
    if attributes.contains(data::project::ProjectAttributes::ARTISTIC) {
        attrs.insert(ProjectAttribute::Artistic);
        attributes.remove(data::project::ProjectAttributes::ARTISTIC);
    }
    if attributes.contains(data::project::ProjectAttributes::COMMITTEE) {
        attrs.insert(ProjectAttribute::Committee);
        attributes.remove(data::project::ProjectAttributes::COMMITTEE);
    }
    ensure!(attributes.is_empty());

    Ok(Project {
        id: ProjectId::from_uuid(id),
        created_at,
        owner_id: UserId(owner_id),
        name: ProjectName::from_string(name)?,
        kana_name: ProjectKanaName::from_string(kana_name)?,
        group_name: ProjectGroupName::from_string(group_name)?,
        kana_group_name: ProjectKanaGroupName::from_string(kana_group_name)?,
        description: ProjectDescription::from_string(description)?,
        category: match category {
            data::project::ProjectCategory::General => ProjectCategory::General,
            data::project::ProjectCategory::Stage => ProjectCategory::Stage,
        },
        attributes: ProjectAttributes::from_attributes(attrs)?,
    })
}
