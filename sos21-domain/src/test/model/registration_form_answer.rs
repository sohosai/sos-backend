use crate::model::{
    date_time::DateTime,
    form_answer::FormAnswerItems,
    pending_project::PendingProjectId,
    project::ProjectId,
    registration_form::RegistrationForm,
    registration_form_answer::{
        RegistrationFormAnswer, RegistrationFormAnswerContent, RegistrationFormAnswerId,
        RegistrationFormAnswerRespondent,
    },
    user::UserId,
};
use crate::test::model as test_model;

use uuid::Uuid;

pub fn new_registration_form_answer_id() -> RegistrationFormAnswerId {
    RegistrationFormAnswerId::from_uuid(Uuid::new_v4())
}

pub fn new_registration_form_answer_with_items(
    author_id: UserId,
    respondent: RegistrationFormAnswerRespondent,
    registration_form: &RegistrationForm,
    items: FormAnswerItems,
) -> RegistrationFormAnswer {
    RegistrationFormAnswer::from_content(RegistrationFormAnswerContent {
        id: new_registration_form_answer_id(),
        respondent,
        registration_form_id: registration_form.id,
        created_at: DateTime::now(),
        updated_at: DateTime::now(),
        author_id,
        items,
    })
}

pub fn new_registration_form_answer(
    author_id: UserId,
    respondent: RegistrationFormAnswerRespondent,
    registration_form: &RegistrationForm,
) -> RegistrationFormAnswer {
    new_registration_form_answer_with_items(
        author_id,
        respondent,
        registration_form,
        test_model::mock_form_answer_items(&registration_form.items),
    )
}

pub fn new_registration_form_answer_with_project(
    author_id: UserId,
    project_id: ProjectId,
    registration_form: &RegistrationForm,
) -> RegistrationFormAnswer {
    new_registration_form_answer(
        author_id,
        RegistrationFormAnswerRespondent::Project(project_id),
        registration_form,
    )
}

pub fn new_registration_form_answer_with_pending_project(
    author_id: UserId,
    pending_project_id: PendingProjectId,
    registration_form: &RegistrationForm,
) -> RegistrationFormAnswer {
    new_registration_form_answer(
        author_id,
        RegistrationFormAnswerRespondent::PendingProject(pending_project_id),
        registration_form,
    )
}
