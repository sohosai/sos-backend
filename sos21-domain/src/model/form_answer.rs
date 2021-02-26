use crate::model::date_time::DateTime;
use crate::model::form::FormId;
use crate::model::permissions::Permissions;
use crate::model::project::{Project, ProjectId};
use crate::model::user::{User, UserId};

use uuid::Uuid;

pub mod item;
pub use item::{FormAnswerItem, FormAnswerItems};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl FormAnswer {
    pub fn is_visible_to(&self, user: &User) -> bool {
        user.permissions()
            .contains(Permissions::READ_ALL_FORM_ANSWERS)
    }

    pub fn is_visible_to_with_project(&self, user: &User, project: &Project) -> bool {
        if self.is_visible_to(user) {
            return true;
        }

        self.project_id == project.id && project.is_visible_to(user)
    }
}
