use crate::model::date_time::DateTime;
use crate::model::permissions::Permissions;
use crate::model::user::{User, UserId};

use uuid::Uuid;

pub mod condition;
pub mod description;
pub mod item;
pub mod name;
pub mod period;

pub use condition::{FormCondition, FormConditionProjectSet};
pub use description::FormDescription;
pub use item::{FormItem, FormItems};
pub use name::FormName;
pub use period::FormPeriod;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormId(Uuid);

impl FormId {
    pub fn from_uuid(uuid: Uuid) -> Self {
        FormId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Form {
    pub id: FormId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub name: FormName,
    pub description: FormDescription,
    pub period: FormPeriod,
    pub items: FormItems,
    pub condition: FormCondition,
}

impl Form {
    pub fn is_visible_to(&self, user: &User) -> bool {
        user.permissions().contains(Permissions::READ_ALL_FORMS)
    }
}
