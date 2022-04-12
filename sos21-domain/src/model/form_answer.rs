use crate::context::FormAnswerRepository;
use crate::model::date_time::DateTime;
use crate::model::form::{self, Form, FormId};
use crate::model::permissions::Permissions;
use crate::model::project::{Project, ProjectId};
use crate::model::user::{self, User, UserId};
use crate::{DomainError, DomainResult};

use anyhow::Context;
use thiserror::Error;
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
pub struct FormAnswerContent {
    pub id: FormAnswerId,
    pub project_id: ProjectId,
    pub form_id: FormId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub items: FormAnswerItems,
}

#[derive(Debug, Clone)]
pub struct FormAnswer {
    content: FormAnswerContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewFormAnswerErrorKind {
    AlreadyAnswered,
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
#[error("unable to create form answer")]
pub struct NewFormAnswerError {
    kind: NewFormAnswerErrorKind,
}

impl NewFormAnswerError {
    pub fn kind(&self) -> NewFormAnswerErrorKind {
        self.kind
    }

    fn from_check_error(err: form::item::CheckAnswerError) -> Self {
        let kind = match err.kind() {
            form::item::CheckAnswerErrorKind::MismatchedItemsLength => {
                NewFormAnswerErrorKind::MismatchedItemsLength
            }
            form::item::CheckAnswerErrorKind::MismatchedItemId { expected, got } => {
                NewFormAnswerErrorKind::MismatchedItemId { expected, got }
            }
            form::item::CheckAnswerErrorKind::Item(id, kind) => {
                NewFormAnswerErrorKind::InvalidItem { id, kind }
            }
        };

        NewFormAnswerError { kind }
    }
}

impl FormAnswer {
    pub async fn new<C>(
        ctx: C,
        author: &User,
        project: &Project,
        form: &Form,
        items: FormAnswerItems,
    ) -> DomainResult<Self, NewFormAnswerError>
    where
        C: FormAnswerRepository,
    {
        if ctx
            .get_form_answer_by_form_and_project(form.id(), project.id())
            .await?
            .is_some()
        {
            return Err(DomainError::Domain(NewFormAnswerError {
                kind: NewFormAnswerErrorKind::AlreadyAnswered,
            }));
        }

        form.items()
            .check_answer(&items)
            .context("Failed to check form answers unexpectedly")?
            .map_err(|err| DomainError::Domain(NewFormAnswerError::from_check_error(err)))?;

        Ok(FormAnswer::from_content(FormAnswerContent {
            id: FormAnswerId::from_uuid(Uuid::new_v4()),
            created_at: DateTime::now(),
            author_id: author.id().clone(),
            project_id: project.id(),
            form_id: form.id(),
            items,
        }))
    }

    /// Restore `FormAnswer` from `FormAnswerContent`.
    ///
    /// This is intended to be used when the data is taken out of the implementation by [`FormAnswer::into_content`]
    /// for persistence, internal serialization, etc.
    /// Use [`FormAnswer::new`] to create a form answer.
    pub fn from_content(content: FormAnswerContent) -> Self {
        FormAnswer { content }
    }

    /// Convert `FormAnswer` into `FormAnswerContent`.
    pub fn into_content(self) -> FormAnswerContent {
        self.content
    }

    pub fn id(&self) -> FormAnswerId {
        self.content.id
    }

    pub fn project_id(&self) -> ProjectId {
        self.content.project_id
    }

    pub fn form_id(&self) -> FormId {
        self.content.form_id
    }

    pub fn created_at(&self) -> DateTime {
        self.content.created_at
    }

    pub fn author_id(&self) -> &UserId {
        &self.content.author_id
    }

    pub fn items(&self) -> &FormAnswerItems {
        &self.content.items
    }

    pub fn into_items(self) -> FormAnswerItems {
        self.content.items
    }

    pub fn is_visible_to(&self, user: &User) -> bool {
        user.permissions()
            .contains(Permissions::READ_ALL_FORM_ANSWERS)
    }

    pub fn is_visible_to_with_project(&self, user: &User, project: &Project) -> bool {
        if self.is_visible_to(user) {
            return true;
        }

        self.project_id() == project.id() && project.is_visible_to(user)
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
#[error("failed to set form answer items")]
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

impl FormAnswer {
    // TODO: Fetch form and project in set_items
    pub fn set_items(
        &mut self,
        user: &User,
        form: &Form,
        project: &Project,
        items: FormAnswerItems,
    ) -> DomainResult<(), SetItemsError> {
        domain_ensure!(form.id() == self.form_id());
        domain_ensure!(project.id() == self.project_id());

        let now = DateTime::now();
        let permission = if form.period().contains(now) && project.is_member(user) {
            Permissions::UPDATE_FORM_ANSWERS_IN_PERIOD
        } else {
            Permissions::UPDATE_ALL_FORM_ANSWERS
        };

        user.require_permissions(permission)
            .map_err(|err| DomainError::Domain(SetItemsError::from_permissions_error(err)))?;

        form.items()
            .check_answer(&items)
            .context("Failed to check form answers unexpectedly")?
            .map_err(|err| DomainError::Domain(SetItemsError::from_check_error(err)))?;

        self.content.items = items;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SetItemsErrorKind;

    use crate::test::model as test_model;
    use crate::DomainError;

    #[test]
    fn test_visibility_general() {
        let user = test_model::new_general_user();
        let user_project = test_model::new_general_online_project(user.id().clone());
        let operator = test_model::new_operator_user();
        let form = test_model::new_form(operator.id().clone());
        let form_answer = test_model::new_form_answer(user.id().clone(), &user_project, &form);
        assert!(!form_answer.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_committee() {
        let user = test_model::new_committee_user();
        let user_project = test_model::new_general_online_project(user.id().clone());
        let operator = test_model::new_operator_user();
        let form = test_model::new_form(operator.id().clone());
        let form_answer = test_model::new_form_answer(user.id().clone(), &user_project, &form);
        assert!(form_answer.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_operator() {
        let user = test_model::new_operator_user();
        let user_project = test_model::new_general_online_project(user.id().clone());
        let operator = test_model::new_operator_user();
        let form = test_model::new_form(operator.id().clone());
        let form_answer = test_model::new_form_answer(user.id().clone(), &user_project, &form);
        assert!(form_answer.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_general_via_owning_project() {
        let user = test_model::new_general_user();
        let user_project = test_model::new_general_online_project(user.id().clone());
        let operator = test_model::new_operator_user();
        let form = test_model::new_form(operator.id().clone());
        let form_answer = test_model::new_form_answer(user.id().clone(), &user_project, &form);
        assert!(form_answer.is_visible_to_with_project(&user, &user_project));
    }

    #[test]
    fn test_visibility_general_via_non_owning_project() {
        let user = test_model::new_general_user();
        let operator = test_model::new_operator_user();
        let operator_project = test_model::new_general_online_project(operator.id().clone());
        let form = test_model::new_form(operator.id().clone());
        let form_answer = test_model::new_form_answer(user.id().clone(), &operator_project, &form);
        assert!(!form_answer.is_visible_to_with_project(&user, &operator_project));
    }

    #[test]
    fn test_set_items_general_in_period() {
        use crate::model::form::item;
        use crate::model::form_answer::item as answer_item;

        let user = test_model::new_general_user();
        let user_project = test_model::new_general_online_project(user.id().clone());
        let operator = test_model::new_operator_user();

        let (items, answer_items, item_id) = {
            let body = item::FormItemBody::Integer(
                item::IntegerFormItem::from_content(item::integer::IntegerFormItemContent {
                    is_required: false,
                    max: None,
                    min: None,
                    placeholder: None,
                    unit: None,
                })
                .unwrap(),
            );
            let item = test_model::new_form_item_with_body(body);
            let item_id = item.id;
            let items = item::FormItems::from_items(vec![item]).unwrap();
            let answer_item = answer_item::FormAnswerItem {
                item_id,
                body: Some(answer_item::FormAnswerItemBody::Integer(Some(10))),
            };
            let answer_items = answer_item::FormAnswerItems::from_items(vec![answer_item]).unwrap();
            (items, answer_items, item_id)
        };

        let form = test_model::new_form_with_items(operator.id().clone(), items);
        let mut form_answer = test_model::new_form_answer_with_items(
            user.id().clone(),
            &user_project,
            &form,
            answer_items,
        );

        let new_items = {
            let answer_item = answer_item::FormAnswerItem {
                item_id,
                body: Some(answer_item::FormAnswerItemBody::Integer(Some(20))),
            };
            answer_item::FormAnswerItems::from_items(vec![answer_item]).unwrap()
        };

        form_answer
            .set_items(&user, &form, &user_project, new_items.clone())
            .unwrap();

        assert_eq!(form_answer.items(), &new_items);
    }

    #[test]
    fn test_set_items_general_after_period() {
        let user = test_model::new_general_user();
        let user_project = test_model::new_general_online_project(user.id().clone());
        let operator = test_model::new_operator_user();
        let period = test_model::new_form_period_to_now();
        let form = test_model::new_form_with_period(operator.id().clone(), period);
        let mut form_answer = test_model::new_form_answer(user.id().clone(), &user_project, &form);
        assert!(matches!(
            form_answer
                .set_items(
                    &user,
                    &form,
                    &user_project,
                    test_model::mock_form_answer_items(form.items()),
                ),
            Err(DomainError::Domain(err))
            if err.kind() == SetItemsErrorKind::InsufficientPermissions
        ));
    }
}
