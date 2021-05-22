use crate::context::{ConfigContext, RegistrationFormAnswerRepository};
use crate::model::date_time::DateTime;
use crate::model::form::item::{self, FormItemId, FormItems};
use crate::model::form_answer::item::FormAnswerItems;
use crate::model::pending_project::PendingProject;
use crate::model::permissions::Permissions;
use crate::model::project::Project;
use crate::model::project_query::ProjectQuery;
use crate::model::registration_form_answer::{self, RegistrationFormAnswer};
use crate::model::user::{User, UserId};
use crate::{DomainError, DomainResult};

use thiserror::Error;
use uuid::Uuid;

pub mod description;
pub mod name;

pub use description::RegistrationFormDescription;
pub use name::RegistrationFormName;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegistrationFormId(Uuid);

impl RegistrationFormId {
    pub fn from_uuid(uuid: Uuid) -> Self {
        RegistrationFormId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct RegistrationForm {
    pub id: RegistrationFormId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub name: RegistrationFormName,
    pub description: RegistrationFormDescription,
    pub items: FormItems,
    pub query: ProjectQuery,
}

#[derive(Debug, Clone, Copy)]
pub enum AnswerErrorKind {
    NotTargeted,
    AlreadyAnswered,
    OutOfProjectCreationPeriod,
    MismatchedItemsLength,
    MismatchedItemId {
        expected: FormItemId,
        got: FormItemId,
    },
    InvalidItem {
        id: FormItemId,
        kind: item::CheckAnswerItemErrorKind,
    },
}

#[derive(Debug, Error, Clone)]
#[error("Cannot answer the registration form")]
pub struct AnswerError {
    kind: AnswerErrorKind,
}

impl AnswerError {
    pub fn kind(&self) -> AnswerErrorKind {
        self.kind
    }

    fn from_new_registration_form_answer_error(
        err: registration_form_answer::NewRegistrationFormAnswerError,
    ) -> Self {
        let kind = match err.kind() {
            registration_form_answer::NewRegistrationFormAnswerErrorKind::MismatchedItemsLength => {
                AnswerErrorKind::MismatchedItemsLength
            }
            registration_form_answer::NewRegistrationFormAnswerErrorKind::MismatchedItemId {
                expected,
                got,
            } => AnswerErrorKind::MismatchedItemId { expected, got },
            registration_form_answer::NewRegistrationFormAnswerErrorKind::InvalidItem {
                id,
                kind,
            } => AnswerErrorKind::InvalidItem { id, kind },
            registration_form_answer::NewRegistrationFormAnswerErrorKind::AlreadyAnswered => {
                AnswerErrorKind::AlreadyAnswered
            }
            registration_form_answer::NewRegistrationFormAnswerErrorKind::OutOfProjectCreationPeriod => {
                AnswerErrorKind::OutOfProjectCreationPeriod
            }
        };

        AnswerError { kind }
    }
}

impl RegistrationForm {
    pub async fn answer_by<C>(
        &self,
        ctx: C,
        user: &User,
        pending_project: &PendingProject,
        items: FormAnswerItems,
    ) -> DomainResult<RegistrationFormAnswer, AnswerError>
    where
        C: RegistrationFormAnswerRepository + ConfigContext,
    {
        domain_ensure!(user.id() == pending_project.owner_id());

        if !self.query.check_pending_project(&pending_project) {
            return Err(DomainError::Domain(AnswerError {
                kind: AnswerErrorKind::NotTargeted,
            }));
        }

        RegistrationFormAnswer::new(ctx, user, pending_project, self, items)
            .await
            .map_err(|err| err.map_domain(AnswerError::from_new_registration_form_answer_error))
    }

    pub fn id(&self) -> RegistrationFormId {
        self.id
    }

    pub fn items(&self) -> &FormItems {
        &self.items
    }

    pub fn is_visible_to(&self, user: &User) -> bool {
        user.permissions()
            .contains(Permissions::READ_ALL_REGISTRATION_FORMS)
    }

    pub fn is_visible_to_with_project(&self, user: &User, project: &Project) -> bool {
        if self.is_visible_to(user) {
            return true;
        }

        self.query.check_project(project) && project.is_visible_to(user)
    }

    pub fn is_visible_to_with_pending_project(
        &self,
        user: &User,
        pending_project: &PendingProject,
    ) -> bool {
        if self.is_visible_to(user) {
            return true;
        }

        self.query.check_pending_project(pending_project) && pending_project.owner_id() == user.id()
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{
        project::{ProjectAttributes, ProjectCategory},
        project_query::{ProjectQuery, ProjectQueryConjunction},
    };
    use crate::test::model as test_model;

    #[test]
    fn test_visibility_general() {
        let user = test_model::new_general_user();
        let operator = test_model::new_operator_user();
        let registration_form = test_model::new_registration_form(operator.id().clone());
        assert!(!registration_form.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_committee() {
        let user = test_model::new_committee_user();
        let operator = test_model::new_operator_user();
        let registration_form = test_model::new_registration_form(operator.id().clone());
        assert!(registration_form.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_operator() {
        let user = test_model::new_operator_user();
        let operator = test_model::new_operator_user();
        let registration_form = test_model::new_registration_form(operator.id().clone());
        assert!(registration_form.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_general_via_matching_project() {
        let user = test_model::new_general_user();
        let user_project = test_model::new_general_project(user.id().clone());
        let operator = test_model::new_operator_user();
        let tautology_query = ProjectQuery::from_conjunctions(vec![ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        }])
        .unwrap();
        let registration_form =
            test_model::new_registration_form_with_query(operator.id().clone(), tautology_query);
        assert!(registration_form.is_visible_to_with_project(&user, &user_project));
    }

    #[test]
    fn test_visibility_general_via_matching_non_owner_project() {
        let user = test_model::new_general_user();
        let operator = test_model::new_operator_user();
        let operator_project = test_model::new_general_project(operator.id().clone());
        let tautology_query = ProjectQuery::from_conjunctions(vec![ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        }])
        .unwrap();
        let registration_form =
            test_model::new_registration_form_with_query(operator.id().clone(), tautology_query);
        assert!(!registration_form.is_visible_to_with_project(&user, &operator_project));
    }

    #[test]
    fn test_visibility_general_via_non_matching_project() {
        let user = test_model::new_general_user();
        let user_project = test_model::new_general_project(user.id().clone());
        let operator = test_model::new_operator_user();
        let query = ProjectQuery::from_conjunctions(vec![ProjectQueryConjunction {
            category: Some(ProjectCategory::Stage),
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        }])
        .unwrap();
        let registration_form =
            test_model::new_registration_form_with_query(operator.id().clone(), query);
        assert!(!registration_form.is_visible_to_with_project(&user, &user_project));
    }

    #[test]
    fn test_visibility_general_via_matching_pending_project() {
        let user = test_model::new_general_user();
        let user_pending_project = test_model::new_general_pending_project(user.id().clone());
        let operator = test_model::new_operator_user();
        let tautology_query = ProjectQuery::from_conjunctions(vec![ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        }])
        .unwrap();
        let registration_form =
            test_model::new_registration_form_with_query(operator.id().clone(), tautology_query);
        assert!(registration_form.is_visible_to_with_pending_project(&user, &user_pending_project));
    }

    #[test]
    fn test_visibility_general_via_matching_non_owner_pending_project() {
        let user = test_model::new_general_user();
        let operator = test_model::new_operator_user();
        let operator_pending_project =
            test_model::new_general_pending_project(operator.id().clone());
        let tautology_query = ProjectQuery::from_conjunctions(vec![ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        }])
        .unwrap();
        let registration_form =
            test_model::new_registration_form_with_query(operator.id().clone(), tautology_query);
        assert!(
            !registration_form.is_visible_to_with_pending_project(&user, &operator_pending_project)
        );
    }

    #[test]
    fn test_visibility_general_via_non_matching_pending_project() {
        let user = test_model::new_general_user();
        let user_pending_project = test_model::new_general_pending_project(user.id().clone());
        let operator = test_model::new_operator_user();
        let query = ProjectQuery::from_conjunctions(vec![ProjectQueryConjunction {
            category: Some(ProjectCategory::Stage),
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        }])
        .unwrap();
        let registration_form =
            test_model::new_registration_form_with_query(operator.id().clone(), query);
        assert!(!registration_form.is_visible_to_with_pending_project(&user, &user_pending_project));
    }
}
