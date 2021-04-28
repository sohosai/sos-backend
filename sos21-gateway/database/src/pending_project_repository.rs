use crate::project_repository::{
    from_project_attributes, from_project_category, to_project_attributes, to_project_category,
};
use crate::user_repository::to_user;

use anyhow::Result;
use futures::lock::Mutex;
use ref_cast::RefCast;
use sos21_database::{command, model as data, query};
use sos21_domain::context::pending_project_repository::{
    PendingProjectRepository, PendingProjectWithOwner,
};
use sos21_domain::model::{
    date_time::DateTime,
    pending_project::{PendingProject, PendingProjectContent, PendingProjectId},
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
    ) -> Result<Option<PendingProjectWithOwner>> {
        let mut lock = self.0.lock().await;
        query::find_pending_project(&mut *lock, id.to_uuid())
            .await
            .and_then(|opt| opt.map(to_pending_project_with_owner).transpose())
    }
}

fn from_pending_project(pending_project: PendingProject) -> data::pending_project::PendingProject {
    let PendingProjectContent {
        id,
        created_at,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category,
        attributes,
    } = pending_project.into_content();

    data::pending_project::PendingProject {
        id: id.to_uuid(),
        created_at: created_at.utc(),
        name: name.into_string(),
        kana_name: kana_name.into_string(),
        group_name: group_name.into_string(),
        kana_group_name: kana_group_name.into_string(),
        description: description.into_string(),
        category: from_project_category(category),
        attributes: from_project_attributes(&attributes),
    }
}

fn to_pending_project_with_owner(
    pending_project_with_owner: data::pending_project::PendingProjectWithOwner,
) -> Result<PendingProjectWithOwner> {
    let data::pending_project::PendingProjectWithOwner {
        pending_project,
        owner,
    } = pending_project_with_owner;

    let data::pending_project::PendingProject {
        id,
        created_at,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category,
        attributes,
    } = pending_project;

    let pending_project = PendingProject::from_content(
        PendingProjectContent {
            id: PendingProjectId::from_uuid(id),
            created_at: DateTime::from_utc(created_at),
            name: ProjectName::from_string(name)?,
            kana_name: ProjectKanaName::from_string(kana_name)?,
            group_name: ProjectGroupName::from_string(group_name)?,
            kana_group_name: ProjectKanaGroupName::from_string(kana_group_name)?,
            description: ProjectDescription::from_string(description)?,
            category: to_project_category(category),
            attributes: to_project_attributes(attributes)?,
        },
        UserId(owner.id.clone()),
    );

    Ok(PendingProjectWithOwner {
        pending_project,
        owner: to_user(owner)?,
    })
}
