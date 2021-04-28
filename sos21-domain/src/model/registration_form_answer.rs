use crate::model::date_time::DateTime;
use crate::model::form_answer::FormAnswerItems;
use crate::model::pending_project::PendingProject;
use crate::model::permissions::Permissions;
use crate::model::project::Project;
use crate::model::registration_form::RegistrationFormId;
use crate::model::user::{User, UserId};

use uuid::Uuid;

pub mod respondent;
pub use respondent::RegistrationFormAnswerRespondent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegistrationFormAnswerId(Uuid);

impl RegistrationFormAnswerId {
    pub fn from_uuid(uuid: Uuid) -> Self {
        RegistrationFormAnswerId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct RegistrationFormAnswer {
    pub id: RegistrationFormAnswerId,
    pub respondent: RegistrationFormAnswerRespondent,
    pub registration_form_id: RegistrationFormId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub items: FormAnswerItems,
}

impl RegistrationFormAnswer {
    pub fn is_visible_to(&self, user: &User) -> bool {
        user.permissions()
            .contains(Permissions::READ_ALL_REGISTRATION_FORM_ANSWERS)
    }

    pub fn is_visible_to_with_project(&self, user: &User, project: &Project) -> bool {
        if self.is_visible_to(user) {
            return true;
        }

        self.respondent.is_project(project) && project.is_visible_to(user)
    }

    pub fn is_visible_to_with_pending_project(
        &self,
        user: &User,
        pending_project: &PendingProject,
    ) -> bool {
        if self.is_visible_to(user) {
            return true;
        }

        self.respondent.is_pending_project(pending_project)
            && pending_project.owner_id() == &user.id
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
        let registration_form = test_model::new_registration_form(operator.id);
        let registration_form_answer = test_model::new_registration_form_answer_with_project(
            user.id.clone(),
            user_project.id(),
            &registration_form,
        );
        assert!(!registration_form_answer.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_committee() {
        let user = test_model::new_committee_user();
        let user_project = test_model::new_general_project(user.id.clone());
        let operator = test_model::new_operator_user();
        let registration_form = test_model::new_registration_form(operator.id);
        let registration_form_answer = test_model::new_registration_form_answer_with_project(
            user.id.clone(),
            user_project.id(),
            &registration_form,
        );
        assert!(registration_form_answer.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_operator() {
        let user = test_model::new_operator_user();
        let user_project = test_model::new_general_project(user.id.clone());
        let operator = test_model::new_operator_user();
        let registration_form = test_model::new_registration_form(operator.id);
        let registration_form_answer = test_model::new_registration_form_answer_with_project(
            user.id.clone(),
            user_project.id(),
            &registration_form,
        );
        assert!(registration_form_answer.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_general_via_owning_project() {
        let user = test_model::new_general_user();
        let user_project = test_model::new_general_project(user.id.clone());
        let operator = test_model::new_operator_user();
        let registration_form = test_model::new_registration_form(operator.id);
        let registration_form_answer = test_model::new_registration_form_answer_with_project(
            user.id.clone(),
            user_project.id(),
            &registration_form,
        );
        assert!(registration_form_answer.is_visible_to_with_project(&user, &user_project));
    }

    #[test]
    fn test_visibility_general_via_owning_pending_project() {
        let user = test_model::new_general_user();
        let user_pending_project = test_model::new_general_pending_project(user.id.clone());
        let operator = test_model::new_operator_user();
        let registration_form = test_model::new_registration_form(operator.id);
        let registration_form_answer =
            test_model::new_registration_form_answer_with_pending_project(
                user.id.clone(),
                user_pending_project.id(),
                &registration_form,
            );
        assert!(registration_form_answer
            .is_visible_to_with_pending_project(&user, &user_pending_project));
    }

    #[test]
    fn test_visibility_general_via_non_owning_project() {
        let user = test_model::new_general_user();
        let operator = test_model::new_operator_user();
        let operator_project = test_model::new_general_project(operator.id.clone());
        let registration_form = test_model::new_registration_form(operator.id);
        let registration_form_answer = test_model::new_registration_form_answer_with_project(
            user.id.clone(),
            operator_project.id(),
            &registration_form,
        );
        assert!(!registration_form_answer.is_visible_to_with_project(&user, &operator_project));
    }

    #[test]
    fn test_visibility_general_via_non_owning_pending_project() {
        let user = test_model::new_general_user();
        let operator = test_model::new_operator_user();
        let operator_pending_project = test_model::new_general_pending_project(operator.id.clone());
        let registration_form = test_model::new_registration_form(operator.id);
        let registration_form_answer =
            test_model::new_registration_form_answer_with_pending_project(
                user.id.clone(),
                operator_pending_project.id(),
                &registration_form,
            );
        assert!(!registration_form_answer
            .is_visible_to_with_pending_project(&user, &operator_pending_project));
    }
}
