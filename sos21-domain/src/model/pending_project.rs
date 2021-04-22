use std::convert::TryInto;

use crate::context::ProjectRepository;
use crate::model::date_time::DateTime;
use crate::model::project::{
    self, Project, ProjectAttributes, ProjectCategory, ProjectDescription, ProjectGroupName,
    ProjectId, ProjectIndex, ProjectKanaGroupName, ProjectKanaName, ProjectName,
};
use crate::model::user::{User, UserId};

use anyhow::Context;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PendingProjectId(Uuid);

impl PendingProjectId {
    pub fn from_uuid(uuid: Uuid) -> PendingProjectId {
        PendingProjectId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct PendingProject {
    pub id: PendingProjectId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub name: ProjectName,
    pub kana_name: ProjectKanaName,
    pub group_name: ProjectGroupName,
    pub kana_group_name: ProjectKanaGroupName,
    pub description: ProjectDescription,
    pub category: ProjectCategory,
    pub attributes: ProjectAttributes,
}

#[derive(Debug, Error, Clone)]
#[error("too many projects")]
pub struct TooManyProjectsError {
    _priv: (),
}

impl TooManyProjectsError {
    fn from_count_integer_error(_err: std::num::TryFromIntError) -> Self {
        TooManyProjectsError { _priv: () }
    }

    fn from_index_error(_err: project::index::FromU16Error) -> Self {
        TooManyProjectsError { _priv: () }
    }
}

impl PendingProject {
    pub fn is_visible_to(&self, _user: &User) -> bool {
        true
    }

    pub async fn accept_subowner<C>(
        self,
        ctx: C,
        subowner: &User,
    ) -> anyhow::Result<Result<Project, TooManyProjectsError>>
    where
        C: ProjectRepository,
    {
        let count = ctx
            .count_projects()
            .await
            .context("Failed to count projects")?;
        let count = match count.try_into() {
            Ok(count) => count,
            Err(err) => return Ok(Err(TooManyProjectsError::from_count_integer_error(err))),
        };
        let index = match ProjectIndex::from_u16(count) {
            Ok(index) => index,
            Err(err) => return Ok(Err(TooManyProjectsError::from_index_error(err))),
        };

        Ok(Ok(Project {
            id: ProjectId::from_uuid(Uuid::new_v4()),
            index,
            created_at: DateTime::now(),
            owner_id: self.author_id,
            subowner_id: subowner.id.clone(),
            name: self.name,
            kana_name: self.kana_name,
            group_name: self.group_name,
            kana_group_name: self.kana_group_name,
            description: self.description,
            category: self.category,
            attributes: self.attributes,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::test;

    #[tokio::test]
    async fn test_accept_ok() {
        let owner = test::model::new_general_user();
        let subowner = test::model::new_general_user();

        let app = test::build_mock_app().build();
        let pending_project = test::model::new_general_pending_project(owner.id.clone());
        let project = pending_project
            .accept_subowner(&app, &subowner)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(project.owner_id, owner.id);
        assert_eq!(project.subowner_id, subowner.id);
    }
}
