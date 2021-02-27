use crate::handler::model::form::FormId;
use crate::handler::model::project::ProjectId;
use crate::handler::model::user::UserId;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sos21_use_case::model::form_answer as use_case;
use uuid::Uuid;

pub mod item;
pub use item::FormAnswerItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormAnswerId(pub Uuid);

impl FormAnswerId {
    pub fn from_use_case(id: use_case::FormAnswerId) -> Self {
        FormAnswerId(id.0)
    }

    pub fn into_use_case(self) -> use_case::FormAnswerId {
        use_case::FormAnswerId(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormAnswer {
    pub id: FormAnswerId,
    pub project_id: ProjectId,
    pub form_id: FormId,
    pub created_at: DateTime<Utc>,
    pub author_id: UserId,
    pub items: Vec<FormAnswerItem>,
}

impl FormAnswer {
    pub fn from_use_case(answer: use_case::FormAnswer) -> Self {
        FormAnswer {
            id: FormAnswerId::from_use_case(answer.id),
            project_id: ProjectId::from_use_case(answer.project_id),
            form_id: FormId::from_use_case(answer.form_id),
            created_at: answer.created_at,
            author_id: UserId::from_use_case(answer.author_id),
            items: answer
                .items
                .into_iter()
                .map(FormAnswerItem::from_use_case)
                .collect(),
        }
    }
}
