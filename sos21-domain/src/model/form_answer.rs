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

#[cfg(test)]
mod tests {
    use crate::test::model as test_model;

    #[test]
    fn test_visibility_general() {
        let user = test_model::new_general_user();
        let user_project = test_model::new_general_project(user.id.clone());
        let operator = test_model::new_operator_user();
        let form = test_model::new_form(operator.id);
        let form_answer = test_model::new_form_answer(user.id.clone(), user_project.id, &form);
        assert!(!form_answer.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_committee() {
        let user = test_model::new_committee_user();
        let user_project = test_model::new_general_project(user.id.clone());
        let operator = test_model::new_operator_user();
        let form = test_model::new_form(operator.id);
        let form_answer = test_model::new_form_answer(user.id.clone(), user_project.id, &form);
        assert!(form_answer.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_operator() {
        let user = test_model::new_operator_user();
        let user_project = test_model::new_general_project(user.id.clone());
        let operator = test_model::new_operator_user();
        let form = test_model::new_form(operator.id);
        let form_answer = test_model::new_form_answer(user.id.clone(), user_project.id, &form);
        assert!(form_answer.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_general_via_owning_project() {
        let user = test_model::new_general_user();
        let user_project = test_model::new_general_project(user.id.clone());
        let operator = test_model::new_operator_user();
        let form = test_model::new_form(operator.id);
        let form_answer = test_model::new_form_answer(user.id.clone(), user_project.id, &form);
        assert!(form_answer.is_visible_to_with_project(&user, &user_project));
    }

    #[test]
    fn test_visibility_general_via_non_owning_project() {
        let user = test_model::new_general_user();
        let operator = test_model::new_operator_user();
        let operator_project = test_model::new_general_project(operator.id.clone());
        let form = test_model::new_form(operator.id);
        let form_answer = test_model::new_form_answer(user.id.clone(), operator_project.id, &form);
        assert!(!form_answer.is_visible_to_with_project(&user, &operator_project));
    }
}
