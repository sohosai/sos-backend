use crate::model::form::FormItem;
use crate::model::project_query::ProjectQuery;
use crate::model::user::UserId;

use chrono::{DateTime, Utc};
use sos21_domain::model::registration_form as entity;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegistrationFormId(pub Uuid);

impl RegistrationFormId {
    pub fn from_entity(id: entity::RegistrationFormId) -> RegistrationFormId {
        RegistrationFormId(id.to_uuid())
    }

    pub fn into_entity(self) -> entity::RegistrationFormId {
        entity::RegistrationFormId::from_uuid(self.0)
    }
}

#[derive(Debug, Clone)]
pub struct RegistrationForm {
    pub id: RegistrationFormId,
    pub created_at: DateTime<Utc>,
    pub author_id: UserId,
    pub name: String,
    pub description: String,
    pub items: Vec<FormItem>,
    pub query: ProjectQuery,
}

impl RegistrationForm {
    pub fn from_entity(registratiion_form: entity::RegistrationForm) -> RegistrationForm {
        RegistrationForm {
            id: RegistrationFormId::from_entity(registratiion_form.id),
            created_at: registratiion_form.created_at.utc(),
            author_id: UserId::from_entity(registratiion_form.author_id),
            name: registratiion_form.name.into_string(),
            description: registratiion_form.description.into_string(),
            items: registratiion_form
                .items
                .into_items()
                .map(FormItem::from_entity)
                .collect(),
            query: ProjectQuery::from_entity(registratiion_form.query),
        }
    }
}
