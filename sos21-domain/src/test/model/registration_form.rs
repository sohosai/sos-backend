use crate::model::{
    date_time::DateTime,
    form::FormItems,
    project_query::ProjectQuery,
    registration_form::{
        RegistrationForm, RegistrationFormDescription, RegistrationFormId, RegistrationFormName,
    },
    user::UserId,
};
use crate::test::model as test_model;

use uuid::Uuid;

pub fn new_registration_form_id() -> RegistrationFormId {
    RegistrationFormId::from_uuid(Uuid::new_v4())
}

pub fn mock_registration_form_name() -> RegistrationFormName {
    RegistrationFormName::from_string("テスト申請").unwrap()
}

pub fn mock_registration_form_description() -> RegistrationFormDescription {
    RegistrationFormDescription::from_string("テスト").unwrap()
}

pub fn new_registration_form_with_items(author_id: UserId, items: FormItems) -> RegistrationForm {
    RegistrationForm {
        id: new_registration_form_id(),
        created_at: DateTime::now(),
        author_id,
        name: mock_registration_form_name(),
        description: mock_registration_form_description(),
        items,
        query: test_model::mock_project_query(),
    }
}

pub fn new_registration_form_with_query(
    author_id: UserId,
    query: ProjectQuery,
) -> RegistrationForm {
    RegistrationForm {
        id: new_registration_form_id(),
        created_at: DateTime::now(),
        author_id,
        name: mock_registration_form_name(),
        description: mock_registration_form_description(),
        items: test_model::new_form_items(),
        query,
    }
}

/// Mocks a new `RegistrationForm` with wildcard query
pub fn new_registration_form(author_id: UserId) -> RegistrationForm {
    new_registration_form_with_query(author_id, test_model::mock_project_query())
}
