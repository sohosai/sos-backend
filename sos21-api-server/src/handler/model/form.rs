use crate::handler::model::date_time::DateTime;
use crate::handler::model::project::ProjectId;
use crate::handler::model::project_query::ProjectQuery;
use crate::handler::model::user::UserId;

use serde::{Deserialize, Serialize};
use sos21_use_case::model::form as use_case;
use uuid::Uuid;

pub mod item;
pub use item::{FormItem, FormItemId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormId(pub Uuid);

impl FormId {
    pub fn from_use_case(id: use_case::FormId) -> Self {
        FormId(id.0)
    }

    pub fn into_use_case(self) -> use_case::FormId {
        use_case::FormId(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormCondition {
    pub query: ProjectQuery,
    pub includes: Vec<ProjectId>,
    pub excludes: Vec<ProjectId>,
}

impl FormCondition {
    pub fn from_use_case(condition: use_case::FormCondition) -> Self {
        let includes = condition
            .includes
            .into_iter()
            .map(ProjectId::from_use_case)
            .collect();
        let excludes = condition
            .excludes
            .into_iter()
            .map(ProjectId::from_use_case)
            .collect();
        FormCondition {
            query: ProjectQuery::from_use_case(condition.query),
            includes,
            excludes,
        }
    }

    pub fn into_use_case(self) -> use_case::FormCondition {
        let includes = self
            .includes
            .into_iter()
            .map(ProjectId::into_use_case)
            .collect();
        let excludes = self
            .excludes
            .into_iter()
            .map(ProjectId::into_use_case)
            .collect();
        use_case::FormCondition {
            query: self.query.into_use_case(),
            includes,
            excludes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form {
    pub id: FormId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub name: String,
    pub description: String,
    pub starts_at: DateTime,
    pub ends_at: DateTime,
    pub items: Vec<FormItem>,
    pub condition: FormCondition,
    pub answer_notification_webhook: Option<String>,
}

impl Form {
    pub fn from_use_case(form: use_case::Form) -> Self {
        let items = form
            .items
            .into_iter()
            .map(FormItem::from_use_case)
            .collect();
        Form {
            id: FormId::from_use_case(form.id),
            created_at: DateTime::from_use_case(form.created_at),
            author_id: UserId::from_use_case(form.author_id),
            name: form.name,
            description: form.description,
            starts_at: DateTime::from_use_case(form.starts_at),
            ends_at: DateTime::from_use_case(form.ends_at),
            items,
            condition: FormCondition::from_use_case(form.condition),
            answer_notification_webhook: form.answer_notification_webhook,
        }
    }
}
