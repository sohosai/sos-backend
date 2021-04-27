use std::convert::TryInto;

use crate::context::{
    ProjectRepository, RegistrationFormAnswerRepository, RegistrationFormRepository,
};
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

#[derive(Debug, Clone, Copy)]
pub enum AcceptSubownerErrorKind {
    TooManyProjects,
    NotAnsweredRegistrationForm,
    SameOwnerSubowner,
}

#[derive(Debug, Error, Clone)]
#[error("failed to accept subowner")]
pub struct AcceptSubownerError {
    kind: AcceptSubownerErrorKind,
}

impl AcceptSubownerError {
    pub fn kind(&self) -> AcceptSubownerErrorKind {
        self.kind
    }

    fn from_count_integer_error(_err: std::num::TryFromIntError) -> Self {
        AcceptSubownerError {
            kind: AcceptSubownerErrorKind::TooManyProjects,
        }
    }

    fn from_index_error(_err: project::index::FromU16Error) -> Self {
        AcceptSubownerError {
            kind: AcceptSubownerErrorKind::TooManyProjects,
        }
    }

    fn from_project_error(_err: project::SameOwnerSubownerError) -> Self {
        AcceptSubownerError {
            kind: AcceptSubownerErrorKind::SameOwnerSubowner,
        }
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
    ) -> anyhow::Result<Result<Project, AcceptSubownerError>>
    where
        C: ProjectRepository + RegistrationFormRepository + RegistrationFormAnswerRepository,
    {
        let forms_count = ctx
            .count_registration_forms_by_pending_project(self.id)
            .await
            .context("Failed to count registration forms")?;
        let answers_count = ctx
            .count_registration_form_answers_by_pending_project(self.id)
            .await
            .context("Failed to count registration form answers")?;
        if forms_count != answers_count {
            return Ok(Err(AcceptSubownerError {
                kind: AcceptSubownerErrorKind::NotAnsweredRegistrationForm,
            }));
        }

        let projects_count = ctx
            .count_projects()
            .await
            .context("Failed to count projects")?;
        let projects_count = match projects_count.try_into() {
            Ok(count) => count,
            Err(err) => return Ok(Err(AcceptSubownerError::from_count_integer_error(err))),
        };
        let index = match ProjectIndex::from_u16(projects_count) {
            Ok(index) => index,
            Err(err) => return Ok(Err(AcceptSubownerError::from_index_error(err))),
        };

        Ok(Project::from_content(project::ProjectContent {
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
        })
        .map_err(AcceptSubownerError::from_project_error))
    }
}

#[cfg(test)]
mod tests {
    use crate::test;

    #[tokio::test]
    async fn test_accept_ok() {
        let owner = test::model::new_general_user();
        let subowner = test::model::new_general_user();

        let pending_project = test::model::new_general_pending_project(owner.id.clone());
        let app = test::build_mock_app()
            .users(vec![owner.clone(), subowner.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build();

        let project = pending_project
            .accept_subowner(&app, &subowner)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(project.owner_id(), &owner.id);
        assert_eq!(project.subowner_id(), &subowner.id);
    }
}
