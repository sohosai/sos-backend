use crate::project_repository::{
    from_project_attributes, from_project_category, to_project_attributes, to_project_category,
};
use crate::user_repository::to_user;

use anyhow::Result;
use futures::{future, lock::Mutex, stream::TryStreamExt};
use ref_cast::RefCast;
use sos21_database::{command, model as data, query};
use sos21_domain::context::pending_project_repository::{
    PendingProjectRepository, PendingProjectWithAuthor,
};
use sos21_domain::model::{
    date_time::DateTime,
    pending_project::{PendingProject, PendingProjectId},
    project::{
        ProjectDescription, ProjectGroupName, ProjectKanaGroupName, ProjectKanaName, ProjectName,
    },
    user::UserId,
};
use sqlx::{Postgres, Transaction};

#[derive(Debug, RefCast)]
#[repr(transparent)]
pub struct PendingProjectDatabase(Mutex<Transaction<'static, Postgres>>);

#[async_trait::async_trait]
impl PendingProjectRepository for PendingProjectDatabase {
    async fn store_pending_project(&self, pending_project: PendingProject) -> Result<()> {
        let mut lock = self.0.lock().await;

        let pending_project = from_pending_project(pending_project);
        if query::find_pending_project(&mut *lock, pending_project.id)
            .await?
            .is_some()
        {
            let input = command::update_pending_project::Input {
                id: pending_project.id,
                created_at: pending_project.created_at,
                author_id: pending_project.author_id,
                name: pending_project.name,
                kana_name: pending_project.kana_name,
                group_name: pending_project.group_name,
                kana_group_name: pending_project.kana_group_name,
                description: pending_project.description,
                category: pending_project.category,
                attributes: pending_project.attributes,
            };
            command::update_pending_project(&mut *lock, input).await
        } else {
            command::insert_pending_project(&mut *lock, pending_project).await
        }
    }

    async fn delete_pending_project(&self, id: PendingProjectId) -> Result<()> {
        let mut lock = self.0.lock().await;
        command::delete_pending_project(&mut *lock, id.to_uuid()).await
    }

    async fn get_pending_project(
        &self,
        id: PendingProjectId,
    ) -> Result<Option<PendingProjectWithAuthor>> {
        let mut lock = self.0.lock().await;
        query::find_pending_project(&mut *lock, id.to_uuid())
            .await
            .and_then(|opt| opt.map(to_pending_project_with_author).transpose())
    }

    async fn list_pending_projects_by_user(&self, user_id: UserId) -> Result<Vec<PendingProject>> {
        let mut lock = self.0.lock().await;
        query::list_pending_projects_by_user(&mut *lock, user_id.0)
            .and_then(|pending_project| future::ready(to_pending_project(pending_project)))
            .try_collect()
            .await
    }
}

fn from_pending_project(pending_project: PendingProject) -> data::pending_project::PendingProject {
    let PendingProject {
        id,
        created_at,
        author_id,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category,
        attributes,
    } = pending_project;

    data::pending_project::PendingProject {
        id: id.to_uuid(),
        created_at: created_at.utc(),
        author_id: author_id.0,
        name: name.into_string(),
        kana_name: kana_name.into_string(),
        group_name: group_name.into_string(),
        kana_group_name: kana_group_name.into_string(),
        description: description.into_string(),
        category: from_project_category(category),
        attributes: from_project_attributes(&attributes),
    }
}

fn to_pending_project_with_author(
    pending_project_with_author: data::pending_project::PendingProjectWithAuthor,
) -> Result<PendingProjectWithAuthor> {
    let data::pending_project::PendingProjectWithAuthor {
        pending_project,
        author,
    } = pending_project_with_author;

    Ok(PendingProjectWithAuthor {
        pending_project: to_pending_project(pending_project)?,
        author: to_user(author)?,
    })
}

fn to_pending_project(
    pending_project: data::pending_project::PendingProject,
) -> Result<PendingProject> {
    let data::pending_project::PendingProject {
        id,
        created_at,
        author_id,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category,
        attributes,
    } = pending_project;

    Ok(PendingProject {
        id: PendingProjectId::from_uuid(id),
        created_at: DateTime::from_utc(created_at),
        author_id: UserId(author_id),
        name: ProjectName::from_string(name)?,
        kana_name: ProjectKanaName::from_string(kana_name)?,
        group_name: ProjectGroupName::from_string(group_name)?,
        kana_group_name: ProjectKanaGroupName::from_string(kana_group_name)?,
        description: ProjectDescription::from_string(description)?,
        category: to_project_category(category),
        attributes: to_project_attributes(attributes)?,
    })
}
