use crate::model::form_answer::FormAnswerItem;
use crate::model::pending_project::PendingProjectId;
use crate::model::project::ProjectId;
use crate::model::registration_form::RegistrationFormId;
use crate::model::user::UserId;

use chrono::{DateTime, Utc};
use sos21_domain::model::registration_form_answer as entity;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegistrationFormAnswerId(pub Uuid);

impl RegistrationFormAnswerId {
    pub fn from_entity(id: entity::RegistrationFormAnswerId) -> RegistrationFormAnswerId {
        RegistrationFormAnswerId(id.to_uuid())
    }

    pub fn into_entity(self) -> entity::RegistrationFormAnswerId {
        entity::RegistrationFormAnswerId::from_uuid(self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationFormAnswerRespondent {
    Project(ProjectId),
    PendingProject(PendingProjectId),
}

impl RegistrationFormAnswerRespondent {
    pub fn from_entity(respondent: entity::RegistrationFormAnswerRespondent) -> Self {
        match respondent {
            entity::RegistrationFormAnswerRespondent::Project(project_id) => {
                RegistrationFormAnswerRespondent::Project(ProjectId::from_entity(project_id))
            }
            entity::RegistrationFormAnswerRespondent::PendingProject(pending_project_id) => {
                RegistrationFormAnswerRespondent::PendingProject(PendingProjectId::from_entity(
                    pending_project_id,
                ))
            }
        }
    }

    pub fn into_entity(self) -> entity::RegistrationFormAnswerRespondent {
        match self {
            RegistrationFormAnswerRespondent::Project(project_id) => {
                entity::RegistrationFormAnswerRespondent::Project(project_id.into_entity())
            }
            RegistrationFormAnswerRespondent::PendingProject(pending_project_id) => {
                entity::RegistrationFormAnswerRespondent::PendingProject(
                    pending_project_id.into_entity(),
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegistrationFormAnswer {
    pub id: RegistrationFormAnswerId,
    pub registration_form_id: RegistrationFormId,
    pub respondent: RegistrationFormAnswerRespondent,
    pub created_at: DateTime<Utc>,
    pub author_id: UserId,
    pub items: Vec<FormAnswerItem>,
}

impl RegistrationFormAnswer {
    pub fn from_entity(answer: entity::RegistrationFormAnswer) -> Self {
        RegistrationFormAnswer {
            id: RegistrationFormAnswerId::from_entity(answer.id()),
            registration_form_id: RegistrationFormId::from_entity(answer.registration_form_id()),
            respondent: RegistrationFormAnswerRespondent::from_entity(answer.respondent()),
            created_at: answer.created_at().utc(),
            author_id: UserId::from_entity(answer.author_id().clone()),
            items: answer
                .into_items()
                .into_items()
                .map(FormAnswerItem::from_entity)
                .collect(),
        }
    }
}
