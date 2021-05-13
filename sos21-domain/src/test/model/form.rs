use crate::model::{
    date_time::DateTime,
    form::{
        Form, FormCondition, FormConditionProjectSet, FormContent, FormDescription, FormId,
        FormItems, FormName, FormPeriod,
    },
    project_query::ProjectQuery,
    user::UserId,
};
use crate::test::model as test_model;

use uuid::Uuid;

mod item;
pub use item::*;

pub fn new_form_id() -> FormId {
    FormId::from_uuid(Uuid::new_v4())
}

pub fn mock_form_name() -> FormName {
    FormName::from_string("テスト申請").unwrap()
}

pub fn mock_form_description() -> FormDescription {
    FormDescription::from_string("テスト").unwrap()
}

pub fn mock_form_period_with_start(starts_at: DateTime) -> FormPeriod {
    let ends_at = DateTime::from_utc(starts_at.utc() + chrono::Duration::hours(1));
    FormPeriod::from_datetime(starts_at, ends_at).unwrap()
}

pub fn new_form_period_from_now() -> FormPeriod {
    mock_form_period_with_start(DateTime::now())
}

pub fn mock_form_condition() -> FormCondition {
    FormCondition {
        query: test_model::mock_project_query(),
        includes: FormConditionProjectSet::from_projects(Vec::new()).unwrap(),
        excludes: FormConditionProjectSet::from_projects(Vec::new()).unwrap(),
    }
}

pub fn new_form_with_items(author_id: UserId, items: FormItems) -> Form {
    Form::from_content(FormContent {
        id: new_form_id(),
        created_at: DateTime::now(),
        author_id,
        name: mock_form_name(),
        description: mock_form_description(),
        period: new_form_period_from_now(),
        items,
        condition: mock_form_condition(),
    })
}

pub fn new_form_with_condition(author_id: UserId, condition: FormCondition) -> Form {
    Form::from_content(FormContent {
        id: new_form_id(),
        created_at: DateTime::now(),
        author_id,
        name: mock_form_name(),
        description: mock_form_description(),
        period: new_form_period_from_now(),
        items: new_form_items(),
        condition,
    })
}

pub fn new_form_with_query(author_id: UserId, query: ProjectQuery) -> Form {
    new_form_with_condition(
        author_id,
        FormCondition {
            query,
            includes: FormConditionProjectSet::from_projects(Vec::new()).unwrap(),
            excludes: FormConditionProjectSet::from_projects(Vec::new()).unwrap(),
        },
    )
}

/// Mocks a new `Form` with wildcard query
pub fn new_form(author_id: UserId) -> Form {
    new_form_with_query(author_id, test_model::mock_project_query())
}
