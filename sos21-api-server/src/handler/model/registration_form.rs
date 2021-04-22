use crate::handler::model::date_time::DateTime;
use crate::handler::model::form::FormItem;
use crate::handler::model::project_query::ProjectQuery;
use crate::handler::model::user::UserId;

use serde::{Deserialize, Serialize};
use sos21_use_case::model::registration_form as use_case;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RegistrationFormId(pub Uuid);

impl RegistrationFormId {
    pub fn from_use_case(id: use_case::RegistrationFormId) -> Self {
        RegistrationFormId(id.0)
    }

    pub fn into_use_case(self) -> use_case::RegistrationFormId {
        use_case::RegistrationFormId(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationForm {
    pub id: RegistrationFormId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub name: String,
    pub description: String,
    pub items: Vec<FormItem>,
    pub query: ProjectQuery,
}

impl RegistrationForm {
    pub fn from_use_case(registration_form: use_case::RegistrationForm) -> Self {
        let items = registration_form
            .items
            .into_iter()
            .map(FormItem::from_use_case)
            .collect();
        RegistrationForm {
            id: RegistrationFormId::from_use_case(registration_form.id),
            created_at: DateTime::from_use_case(registration_form.created_at),
            author_id: UserId::from_use_case(registration_form.author_id),
            name: registration_form.name,
            description: registration_form.description,
            items,
            query: ProjectQuery::from_use_case(registration_form.query),
        }
    }
}
