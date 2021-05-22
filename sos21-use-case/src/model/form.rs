use crate::model::project::ProjectId;
use crate::model::project_query::ProjectQuery;
use crate::model::user::UserId;

use chrono::{DateTime, Utc};
use sos21_domain::model::form as entity;
use uuid::Uuid;

pub mod item;
pub use item::{FormItem, FormItemId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FormId(pub Uuid);

impl FormId {
    pub fn from_entity(id: entity::FormId) -> FormId {
        FormId(id.to_uuid())
    }

    pub fn into_entity(self) -> entity::FormId {
        entity::FormId::from_uuid(self.0)
    }
}

#[derive(Debug, Clone)]
pub struct FormCondition {
    pub query: ProjectQuery,
    pub includes: Vec<ProjectId>,
    pub excludes: Vec<ProjectId>,
}

impl FormCondition {
    pub fn from_entity(condition: entity::FormCondition) -> FormCondition {
        FormCondition {
            query: ProjectQuery::from_entity(condition.query),
            includes: condition
                .includes
                .projects()
                .map(ProjectId::from_entity)
                .collect(),
            excludes: condition
                .excludes
                .projects()
                .map(ProjectId::from_entity)
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Form {
    pub id: FormId,
    pub created_at: DateTime<Utc>,
    pub author_id: UserId,
    pub name: String,
    pub description: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub items: Vec<FormItem>,
    pub condition: FormCondition,
}

impl Form {
    pub fn from_entity(form: entity::Form) -> Form {
        let period = form.period();
        let condition = FormCondition::from_entity(form.condition().clone());
        Form {
            id: FormId::from_entity(form.id()),
            created_at: form.created_at().utc(),
            author_id: UserId::from_entity(form.author_id().clone()),
            name: form.name().clone().into_string(),
            description: form.description().clone().into_string(),
            starts_at: period.starts_at().utc(),
            ends_at: period.ends_at().utc(),
            items: form
                .into_items()
                .into_items()
                .into_iter()
                .map(FormItem::from_entity)
                .collect(),
            condition,
        }
    }
}
