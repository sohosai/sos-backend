use crate::model::date_time::DateTime;
use crate::model::form::FormId;
use crate::model::project::ProjectId;
use crate::model::user::UserId;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod item;
pub use item::{FormAnswerItem, FormAnswerItems};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FormAnswerId(Uuid);

impl FormAnswerId {
    pub fn from_uuid(uuid: Uuid) -> Self {
        FormAnswerId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct FormAnswer {
    pub id: FormAnswerId,
    pub project_id: ProjectId,
    pub form_id: FormId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub items: FormAnswerItems,
}
