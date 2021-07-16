use crate::context::{ConfigContext, RegistrationFormAnswerRepository};
use crate::model::date_time::DateTime;
use crate::model::form;
use crate::model::form_answer::FormAnswerItems;
use crate::model::pending_project::PendingProject;
use crate::model::permissions::Permissions;
use crate::model::project::Project;
use crate::model::registration_form::{RegistrationForm, RegistrationFormId};
use crate::model::user::{self, User, UserId};
use crate::{DomainError, DomainResult};

use anyhow::Context;
use thiserror::Error;
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
pub struct RegistrationFormAnswerContent {
    pub id: RegistrationFormAnswerId,
    pub respondent: RegistrationFormAnswerRespondent,
    pub registration_form_id: RegistrationFormId,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub author_id: UserId,
    pub items: FormAnswerItems,
}

#[derive(Debug, Clone)]
pub struct RegistrationFormAnswer {
    content: RegistrationFormAnswerContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewRegistrationFormAnswerErrorKind {
    AlreadyAnswered,
    OutOfProjectCreationPeriod,
    MismatchedItemsLength,
    MismatchedItemId {
        expected: form::item::FormItemId,
        got: form::item::FormItemId,
    },
    InvalidItem {
        id: form::item::FormItemId,
        kind: form::item::CheckAnswerItemErrorKind,
    },
}

#[derive(Debug, Error, Clone)]
#[error("unable to create registration form answer")]
pub struct NewRegistrationFormAnswerError {
    kind: NewRegistrationFormAnswerErrorKind,
}

impl NewRegistrationFormAnswerError {
    pub fn kind(&self) -> NewRegistrationFormAnswerErrorKind {
        self.kind
    }

    fn from_check_error(err: form::item::CheckAnswerError) -> Self {
        let kind = match err.kind() {
            form::item::CheckAnswerErrorKind::MismatchedItemsLength => {
                NewRegistrationFormAnswerErrorKind::MismatchedItemsLength
            }
            form::item::CheckAnswerErrorKind::MismatchedItemId { expected, got } => {
                NewRegistrationFormAnswerErrorKind::MismatchedItemId { expected, got }
            }
            form::item::CheckAnswerErrorKind::Item(id, kind) => {
                NewRegistrationFormAnswerErrorKind::InvalidItem { id, kind }
            }
        };

        NewRegistrationFormAnswerError { kind }
    }
}

impl RegistrationFormAnswer {
    pub async fn new<C>(
        ctx: C,
        author: &User,
        pending_project: &PendingProject,
        registration_form: &RegistrationForm,
        items: FormAnswerItems,
    ) -> DomainResult<Self, NewRegistrationFormAnswerError>
    where
        C: RegistrationFormAnswerRepository + ConfigContext,
    {
        if ctx
            .get_registration_form_answer_by_registration_form_and_pending_project(
                registration_form.id(),
                pending_project.id(),
            )
            .await?
            .is_some()
        {
            return Err(DomainError::Domain(NewRegistrationFormAnswerError {
                kind: NewRegistrationFormAnswerErrorKind::AlreadyAnswered,
            }));
        }

        let created_at = DateTime::now();
        if !ctx
            .project_creation_period_for(pending_project.category())
            .contains(created_at)
        {
            return Err(DomainError::Domain(NewRegistrationFormAnswerError {
                kind: NewRegistrationFormAnswerErrorKind::OutOfProjectCreationPeriod,
            }));
        }

        registration_form
            .items()
            .check_answer(&items)
            .context("Failed to check registration form answers unexpectedly")?
            .map_err(|err| {
                DomainError::Domain(NewRegistrationFormAnswerError::from_check_error(err))
            })?;

        Ok(RegistrationFormAnswer::from_content(
            RegistrationFormAnswerContent {
                id: RegistrationFormAnswerId::from_uuid(Uuid::new_v4()),
                respondent: RegistrationFormAnswerRespondent::PendingProject(pending_project.id()),
                registration_form_id: registration_form.id(),
                created_at,
                updated_at: created_at,
                author_id: author.id().clone(),
                items,
            },
        ))
    }

    /// Restore `RegistrationFormAnswer` from `RegistrationFormAnswerContent`.
    ///
    /// This is intended to be used when the data is taken out of the implementation
    /// by [`RegistrationFormAnswer::into_content`] for persistence, internal serialization, etc.
    /// Use [`RegistrationFormAnswer::new`] to create a registration form answer.
    pub fn from_content(content: RegistrationFormAnswerContent) -> Self {
        RegistrationFormAnswer { content }
    }

    /// Convert `RegistrationFormAnswer` into `RegistrationFormAnswerContent`.
    pub fn into_content(self) -> RegistrationFormAnswerContent {
        self.content
    }

    pub fn id(&self) -> RegistrationFormAnswerId {
        self.content.id
    }

    pub fn created_at(&self) -> DateTime {
        self.content.created_at
    }

    pub fn updated_at(&self) -> DateTime {
        self.content.updated_at
    }

    pub fn author_id(&self) -> &UserId {
        &self.content.author_id
    }

    pub fn respondent(&self) -> RegistrationFormAnswerRespondent {
        self.content.respondent
    }

    pub fn registration_form_id(&self) -> RegistrationFormId {
        self.content.registration_form_id
    }

    pub fn items(&self) -> &FormAnswerItems {
        &self.content.items
    }

    pub fn into_items(self) -> FormAnswerItems {
        self.content.items
    }

    pub fn is_visible_to(&self, user: &User) -> bool {
        user.permissions()
            .contains(Permissions::READ_ALL_REGISTRATION_FORM_ANSWERS)
    }

    pub fn is_visible_to_with_project(&self, user: &User, project: &Project) -> bool {
        if self.is_visible_to(user) {
            return true;
        }

        self.respondent().is_project(project) && project.is_visible_to(user)
    }

    pub fn is_visible_to_with_pending_project(
        &self,
        user: &User,
        pending_project: &PendingProject,
    ) -> bool {
        if self.is_visible_to(user) {
            return true;
        }

        self.respondent().is_pending_project(pending_project)
            && pending_project.owner_id() == user.id()
    }

    // TODO: restrict user
    pub fn replace_respondent_to_project(&mut self, project: &Project) {
        self.content.respondent.replace_to_project(project);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetItemsErrorKind {
    InsufficientPermissions,
    MismatchedItemsLength,
    MismatchedItemId {
        expected: form::item::FormItemId,
        got: form::item::FormItemId,
    },
    InvalidItem {
        id: form::item::FormItemId,
        kind: form::item::CheckAnswerItemErrorKind,
    },
}

#[derive(Debug, Clone, Error)]
#[error("failed to set registration form answer items")]
pub struct SetItemsError {
    kind: SetItemsErrorKind,
}

impl SetItemsError {
    pub fn kind(&self) -> SetItemsErrorKind {
        self.kind
    }

    fn from_check_error(err: form::item::CheckAnswerError) -> Self {
        let kind = match err.kind() {
            form::item::CheckAnswerErrorKind::MismatchedItemsLength => {
                SetItemsErrorKind::MismatchedItemsLength
            }
            form::item::CheckAnswerErrorKind::MismatchedItemId { expected, got } => {
                SetItemsErrorKind::MismatchedItemId { expected, got }
            }
            form::item::CheckAnswerErrorKind::Item(id, kind) => {
                SetItemsErrorKind::InvalidItem { id, kind }
            }
        };

        SetItemsError { kind }
    }

    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        SetItemsError {
            kind: SetItemsErrorKind::InsufficientPermissions,
        }
    }
}

impl RegistrationFormAnswer {
    // TODO: Fetch form and (pending) project in set_items_with_*

    pub fn set_items_with_pending_project<C>(
        &mut self,
        ctx: C,
        user: &User,
        registration_form: &RegistrationForm,
        pending_project: &PendingProject,
        items: FormAnswerItems,
    ) -> DomainResult<(), SetItemsError>
    where
        C: ConfigContext,
    {
        domain_ensure!(registration_form.id() == self.registration_form_id());
        domain_ensure!(self.respondent().is_pending_project(pending_project));

        let now = DateTime::now();
        let permission = if ctx
            .project_creation_period_for(pending_project.category())
            .contains(now)
            && pending_project.owner_id() == user.id()
        {
            Permissions::UPDATE_REGISTRATION_FORM_ANSWERS_IN_PERIOD
        } else {
            Permissions::UPDATE_ALL_FORM_ANSWERS
        };

        user.require_permissions(permission)
            .map_err(|err| DomainError::Domain(SetItemsError::from_permissions_error(err)))?;

        registration_form
            .items()
            .check_answer(&items)
            .context("Failed to check registration form answers unexpectedly")?
            .map_err(|err| DomainError::Domain(SetItemsError::from_check_error(err)))?;

        self.content.items = items;
        self.content.updated_at = now;
        Ok(())
    }

    pub fn set_items_with_project<C>(
        &mut self,
        ctx: C,
        user: &User,
        registration_form: &RegistrationForm,
        project: &Project,
        items: FormAnswerItems,
    ) -> DomainResult<(), SetItemsError>
    where
        C: ConfigContext,
    {
        domain_ensure!(registration_form.id() == self.registration_form_id());
        domain_ensure!(self.respondent().is_project(project));

        let now = DateTime::now();
        let permission = if ctx
            .project_creation_period_for(project.category())
            .contains(now)
            && project.is_member(user)
        {
            Permissions::UPDATE_REGISTRATION_FORM_ANSWERS_IN_PERIOD
        } else {
            Permissions::UPDATE_ALL_FORM_ANSWERS
        };

        user.require_permissions(permission)
            .map_err(|err| DomainError::Domain(SetItemsError::from_permissions_error(err)))?;

        registration_form
            .items()
            .check_answer(&items)
            .context("Failed to check registration form answers unexpectedly")?
            .map_err(|err| DomainError::Domain(SetItemsError::from_check_error(err)))?;

        self.content.items = items;
        self.content.updated_at = now;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test::model as test_model;

    #[test]
    fn test_visibility_general() {
        let user = test_model::new_general_user();
        let user_project = test_model::new_general_project(user.id().clone());
        let operator = test_model::new_operator_user();
        let registration_form = test_model::new_registration_form(operator.id().clone());
        let registration_form_answer = test_model::new_registration_form_answer_with_project(
            user.id().clone(),
            user_project.id(),
            &registration_form,
        );
        assert!(!registration_form_answer.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_committee() {
        let user = test_model::new_committee_user();
        let user_project = test_model::new_general_project(user.id().clone());
        let operator = test_model::new_operator_user();
        let registration_form = test_model::new_registration_form(operator.id().clone());
        let registration_form_answer = test_model::new_registration_form_answer_with_project(
            user.id().clone(),
            user_project.id(),
            &registration_form,
        );
        assert!(registration_form_answer.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_operator() {
        let user = test_model::new_operator_user();
        let user_project = test_model::new_general_project(user.id().clone());
        let operator = test_model::new_operator_user();
        let registration_form = test_model::new_registration_form(operator.id().clone());
        let registration_form_answer = test_model::new_registration_form_answer_with_project(
            user.id().clone(),
            user_project.id(),
            &registration_form,
        );
        assert!(registration_form_answer.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_general_via_owning_project() {
        let user = test_model::new_general_user();
        let user_project = test_model::new_general_project(user.id().clone());
        let operator = test_model::new_operator_user();
        let registration_form = test_model::new_registration_form(operator.id().clone());
        let registration_form_answer = test_model::new_registration_form_answer_with_project(
            user.id().clone(),
            user_project.id(),
            &registration_form,
        );
        assert!(registration_form_answer.is_visible_to_with_project(&user, &user_project));
    }

    #[test]
    fn test_visibility_general_via_owning_pending_project() {
        let user = test_model::new_general_user();
        let user_pending_project = test_model::new_general_pending_project(user.id().clone());
        let operator = test_model::new_operator_user();
        let registration_form = test_model::new_registration_form(operator.id().clone());
        let registration_form_answer =
            test_model::new_registration_form_answer_with_pending_project(
                user.id().clone(),
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
        let operator_project = test_model::new_general_project(operator.id().clone());
        let registration_form = test_model::new_registration_form(operator.id().clone());
        let registration_form_answer = test_model::new_registration_form_answer_with_project(
            user.id().clone(),
            operator_project.id(),
            &registration_form,
        );
        assert!(!registration_form_answer.is_visible_to_with_project(&user, &operator_project));
    }

    #[test]
    fn test_visibility_general_via_non_owning_pending_project() {
        let user = test_model::new_general_user();
        let operator = test_model::new_operator_user();
        let operator_pending_project =
            test_model::new_general_pending_project(operator.id().clone());
        let registration_form = test_model::new_registration_form(operator.id().clone());
        let registration_form_answer =
            test_model::new_registration_form_answer_with_pending_project(
                user.id().clone(),
                operator_pending_project.id(),
                &registration_form,
            );
        assert!(!registration_form_answer
            .is_visible_to_with_pending_project(&user, &operator_pending_project));
    }

    // TODO: test new out of period
    // TODO: test set_items_*
}
