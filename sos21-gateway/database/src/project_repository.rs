use std::collections::HashSet;
use std::convert::TryInto;

use crate::user_repository::to_user;

use anyhow::{ensure, Result};
use futures::lock::Mutex;
use futures::{future, stream::TryStreamExt};
use ref_cast::RefCast;
use sos21_database::{command, model as data, query};
use sos21_domain::context::ProjectRepository;
use sos21_domain::model::{
    date_time::DateTime,
    project::{
        Project, ProjectAttribute, ProjectAttributes, ProjectCategory, ProjectDescription,
        ProjectGroupName, ProjectId, ProjectIndex, ProjectKanaGroupName, ProjectKanaName,
        ProjectName,
    },
    user::{User, UserId},
};
use sqlx::{Postgres, Transaction};

#[derive(Debug, RefCast)]
#[repr(transparent)]
pub struct ProjectDatabase(Mutex<Transaction<'static, Postgres>>);

#[async_trait::async_trait]
impl ProjectRepository for ProjectDatabase {
    async fn store_project(&self, project: Project) -> Result<()> {
        let mut lock = self.0.lock().await;

        let project = from_project(project);
        if query::find_project(&mut *lock, project.id).await?.is_some() {
            let input = command::update_project::Input {
                id: project.id,
                owner_id: project.owner_id,
                name: project.name,
                kana_name: project.kana_name,
                group_name: project.group_name,
                kana_group_name: project.kana_group_name,
                description: project.description,
                category: project.category,
                attributes: project.attributes,
            };
            command::update_project(&mut *lock, input).await
        } else {
            command::insert_project(&mut *lock, project).await
        }
    }

    async fn get_project_by_index(&self, index: ProjectIndex) -> Result<Option<(Project, User)>> {
        let mut lock = self.0.lock().await;

        let opt = query::find_project_by_index(&mut *lock, index.to_i16()).await?;
        let result = match opt {
            Some(x) => x,
            None => return Ok(None),
        };
        to_project_with_owner(result.project, result.owner).map(Some)
    }

    async fn get_project(&self, id: ProjectId) -> Result<Option<(Project, User)>> {
        let mut lock = self.0.lock().await;
        let result = match query::find_project(&mut *lock, id.to_uuid()).await? {
            Some(x) => x,
            None => return Ok(None),
        };
        to_project_with_owner(result.project, result.owner).map(Some)
    }

    async fn count_projects(&self) -> Result<u64> {
        let mut lock = self.0.lock().await;
        query::count_projects(&mut *lock).await
    }

    async fn list_projects(&self) -> Result<Vec<(Project, User)>> {
        let mut lock = self.0.lock().await;
        query::list_projects(&mut *lock)
            .and_then(|result| future::ready(to_project_with_owner(result.project, result.owner)))
            .try_collect()
            .await
    }

    async fn list_projects_by_owner(&self, id: UserId) -> Result<Vec<Project>> {
        let mut lock = self.0.lock().await;
        query::list_projects_by_owner(&mut *lock, id.0)
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
        index,
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
        index: index.to_i16(),
        created_at: created_at.utc(),
        owner_id: owner_id.0,
        name: name.into_string(),
        kana_name: kana_name.into_string(),
        group_name: group_name.into_string(),
        kana_group_name: kana_group_name.into_string(),
        description: description.into_string(),
        category: match category {
            ProjectCategory::General => data::project::ProjectCategory::General,
            ProjectCategory::Stage => data::project::ProjectCategory::Stage,
            ProjectCategory::Cooking => data::project::ProjectCategory::Cooking,
            ProjectCategory::Food => data::project::ProjectCategory::Food,
        },
        attributes: attributes
            .attributes()
            .map(|attr| match attr {
                ProjectAttribute::Academic => data::project::ProjectAttributes::ACADEMIC,
                ProjectAttribute::Artistic => data::project::ProjectAttributes::ARTISTIC,
                ProjectAttribute::Committee => data::project::ProjectAttributes::COMMITTEE,
                ProjectAttribute::Outdoor => data::project::ProjectAttributes::OUTDOOR,
            })
            .collect(),
    }
}

fn to_project(project: data::project::Project) -> Result<Project> {
    let data::project::Project {
        id,
        index,
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
    if attributes.contains(data::project::ProjectAttributes::OUTDOOR) {
        attrs.insert(ProjectAttribute::Outdoor);
        attributes.remove(data::project::ProjectAttributes::OUTDOOR);
    }
    ensure!(attributes.is_empty());

    Ok(Project {
        id: ProjectId::from_uuid(id),
        index: ProjectIndex::from_u16(index.try_into()?)?,
        created_at: DateTime::from_utc(created_at),
        owner_id: UserId(owner_id),
        name: ProjectName::from_string(name)?,
        kana_name: ProjectKanaName::from_string(kana_name)?,
        group_name: ProjectGroupName::from_string(group_name)?,
        kana_group_name: ProjectKanaGroupName::from_string(kana_group_name)?,
        description: ProjectDescription::from_string(description)?,
        category: match category {
            data::project::ProjectCategory::General => ProjectCategory::General,
            data::project::ProjectCategory::Stage => ProjectCategory::Stage,
            data::project::ProjectCategory::Cooking => ProjectCategory::Cooking,
            data::project::ProjectCategory::Food => ProjectCategory::Food,
        },
        attributes: ProjectAttributes::from_attributes(attrs)?,
    })
}
