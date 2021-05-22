use crate::model::form::FormId;
use crate::model::project::ProjectId;
use crate::model::user::UserId;

use chrono::{DateTime, Utc};
use sos21_domain::model::form_answer as entity;
use uuid::Uuid;

pub mod item;
pub use item::FormAnswerItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FormAnswerId(pub Uuid);

impl FormAnswerId {
    pub fn from_entity(id: entity::FormAnswerId) -> FormAnswerId {
        FormAnswerId(id.to_uuid())
    }

    pub fn into_entity(self) -> entity::FormAnswerId {
        entity::FormAnswerId::from_uuid(self.0)
    }
}

#[derive(Debug, Clone)]
pub struct FormAnswer {
    pub id: FormAnswerId,
    pub project_id: ProjectId,
    pub form_id: FormId,
    pub created_at: DateTime<Utc>,
    pub author_id: UserId,
    pub items: Vec<FormAnswerItem>,
}

impl FormAnswer {
    pub fn from_entity(answer: entity::FormAnswer) -> Self {
        FormAnswer {
            id: FormAnswerId::from_entity(answer.id()),
            project_id: ProjectId::from_entity(answer.project_id()),
            form_id: FormId::from_entity(answer.form_id()),
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
