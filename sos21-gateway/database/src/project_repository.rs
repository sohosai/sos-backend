use std::collections::HashSet;
use std::convert::TryInto;

use crate::user_repository::to_user;

use anyhow::{ensure, Result};
use futures::lock::Mutex;
use futures::{future, stream::TryStreamExt};
use ref_cast::RefCast;
use sos21_database::{command, model as data, query};
use sos21_domain::context::project_repository::{ProjectRepository, ProjectWithOwners};
use sos21_domain::model::{
    date_time::DateTime,
    project::{
        Project, ProjectAttribute, ProjectAttributes, ProjectCategory, ProjectContent,
        ProjectDescription, ProjectGroupName, ProjectId, ProjectIndex, ProjectKanaGroupName,
        ProjectKanaName, ProjectName,
    },
    user::UserId,
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
                updated_at: project.updated_at,
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

    async fn get_project_by_index(&self, index: ProjectIndex) -> Result<Option<ProjectWithOwners>> {
        let mut lock = self.0.lock().await;
        query::find_project_by_index(&mut *lock, index.to_i16())
            .await?
            .map(to_project_with_owner)
            .transpose()
    }

    async fn get_project(&self, id: ProjectId) -> Result<Option<ProjectWithOwners>> {
        let mut lock = self.0.lock().await;
        query::find_project(&mut *lock, id.to_uuid())
            .await?
            .map(to_project_with_owner)
            .transpose()
    }

    async fn count_projects(&self) -> Result<u64> {
        let mut lock = self.0.lock().await;
        query::count_projects(&mut *lock).await
    }

    async fn get_next_index(&self) -> Result<u64> {
        let mut lock = self.0.lock().await;
        query::get_next_index(&mut *lock).await
    }

    async fn list_projects(&self) -> Result<Vec<ProjectWithOwners>> {
        let mut lock = self.0.lock().await;
        query::list_projects(&mut *lock)
            .and_then(|result| future::ready(to_project_with_owner(result)))
            .try_collect()
            .await
    }
}

fn to_project_with_owner(
    project_with_owners: data::project::ProjectWithOwners,
) -> Result<ProjectWithOwners> {
    let data::project::ProjectWithOwners {
        project,
        owner,
        subowner,
    } = project_with_owners;

    let data::project::Project {
        id,
        index,
        created_at,
        updated_at,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category,
        attributes,
    } = project;

    let project = Project::from_content(
        ProjectContent {
            id: ProjectId::from_uuid(id),
            index: ProjectIndex::from_u16(index.try_into()?)?,
            created_at: DateTime::from_utc(created_at),
            updated_at: DateTime::from_utc(updated_at),
            name: ProjectName::from_string(name)?,
            kana_name: ProjectKanaName::from_string(kana_name)?,
            group_name: ProjectGroupName::from_string(group_name)?,
            kana_group_name: ProjectKanaGroupName::from_string(kana_group_name)?,
            description: ProjectDescription::from_string(description)?,
            category: to_project_category(category),
            attributes: to_project_attributes(attributes)?,
        },
        UserId(owner.id.clone()),
        UserId(subowner.id.clone()),
    )?;

    Ok(ProjectWithOwners {
        project,
        owner: to_user(owner)?,
        subowner: to_user(subowner)?,
    })
}

fn from_project(project: Project) -> data::project::Project {
    let ProjectContent {
        id,
        index,
        created_at,
        updated_at,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category,
        attributes,
    } = project.into_content();

    data::project::Project {
        id: id.to_uuid(),
        index: index.to_i16(),
        created_at: created_at.utc(),
        updated_at: updated_at.utc(),
        name: name.into_string(),
        kana_name: kana_name.into_string(),
        group_name: group_name.into_string(),
        kana_group_name: kana_group_name.into_string(),
        description: description.into_string(),
        category: from_project_category(category),
        attributes: from_project_attributes(&attributes),
    }
}

pub fn from_project_category(category: ProjectCategory) -> data::project::ProjectCategory {
    match category {
        ProjectCategory::General => data::project::ProjectCategory::General,
        ProjectCategory::CookingRequiringPreparationArea => data::project::ProjectCategory::CookingRequiringPreparationArea,
        ProjectCategory::Cooking => data::project::ProjectCategory::Cooking,
        ProjectCategory::Food => data::project::ProjectCategory::Food,
        ProjectCategory::Stage => data::project::ProjectCategory::Stage
    }
}

pub fn from_project_attributes(attributes: &ProjectAttributes) -> data::project::ProjectAttributes {
    attributes
        .attributes()
        .map(|attr| match attr {
            ProjectAttribute::Academic => data::project::ProjectAttributes::ACADEMIC,
            ProjectAttribute::Artistic => data::project::ProjectAttributes::ARTISTIC,
            ProjectAttribute::Committee => data::project::ProjectAttributes::COMMITTEE,
            ProjectAttribute::Outdoor => data::project::ProjectAttributes::OUTDOOR,
        })
        .collect()
}

pub fn to_project_category(category: data::project::ProjectCategory) -> ProjectCategory {
    match category {
        data::project::ProjectCategory::General => ProjectCategory::General,
        data::project::ProjectCategory::CookingRequiringPreparationArea => ProjectCategory::CookingRequiringPreparationArea,
        data::project::ProjectCategory::Cooking => ProjectCategory::Cooking,
        data::project::ProjectCategory::Food => ProjectCategory::Food,
        data::project::ProjectCategory::Stage => ProjectCategory::Stage
    }
}

pub fn to_project_attributes(
    mut attributes: data::project::ProjectAttributes,
) -> Result<ProjectAttributes> {
    // TODO: better impl
    let mut result = HashSet::new();
    if attributes.contains(data::project::ProjectAttributes::ACADEMIC) {
        result.insert(ProjectAttribute::Academic);
        attributes.remove(data::project::ProjectAttributes::ACADEMIC);
    }
    if attributes.contains(data::project::ProjectAttributes::ARTISTIC) {
        result.insert(ProjectAttribute::Artistic);
        attributes.remove(data::project::ProjectAttributes::ARTISTIC);
    }
    if attributes.contains(data::project::ProjectAttributes::COMMITTEE) {
        result.insert(ProjectAttribute::Committee);
        attributes.remove(data::project::ProjectAttributes::COMMITTEE);
    }
    if attributes.contains(data::project::ProjectAttributes::OUTDOOR) {
        result.insert(ProjectAttribute::Outdoor);
        attributes.remove(data::project::ProjectAttributes::OUTDOOR);
    }
    ensure!(attributes.is_empty());

    let result = ProjectAttributes::from_attributes(result)?;
    Ok(result)
}
