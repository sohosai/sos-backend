use crate::context::FormAnswerRepository;
use crate::model::date_time::DateTime;
use crate::model::form_answer::item::FormAnswerItems;
use crate::model::form_answer::{self, FormAnswer};
use crate::model::permissions::Permissions;
use crate::model::project::Project;
use crate::model::user::{self, User, UserId};
use crate::{DomainError, DomainResult};

use thiserror::Error;
use uuid::Uuid;

pub mod condition;
pub mod description;
pub mod item;
pub mod name;
pub mod period;

pub use condition::{FormCondition, FormConditionProjectSet};
pub use description::FormDescription;
pub use item::{FormItem, FormItems};
pub use name::FormName;
pub use period::FormPeriod;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormId(Uuid);

impl FormId {
    pub fn from_uuid(uuid: Uuid) -> Self {
        FormId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Form {
    content: FormContent,
}

#[derive(Debug, Clone)]
pub struct FormContent {
    pub id: FormId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub name: FormName,
    pub description: FormDescription,
    pub period: FormPeriod,
    pub items: FormItems,
    pub condition: FormCondition,
    pub answer_notification_webhook: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewFormErrorKind {
    TooEarlyPeriodStart,
    InsufficientPermissions,
}

#[derive(Debug, Error, Clone)]
#[error("failed to create a form")]
pub struct NewFormError {
    kind: NewFormErrorKind,
}

impl NewFormError {
    pub fn kind(&self) -> NewFormErrorKind {
        self.kind
    }

    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        NewFormError {
            kind: NewFormErrorKind::InsufficientPermissions,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnswerErrorKind {
    OutOfAnswerPeriod,
    NotTargeted,
    AlreadyAnswered,
    MismatchedItemsLength,
    MismatchedItemId {
        expected: item::FormItemId,
        got: item::FormItemId,
    },
    InvalidItem {
        id: item::FormItemId,
        kind: item::CheckAnswerItemErrorKind,
    },
}

#[derive(Debug, Error, Clone)]
#[error("Cannot answer the form")]
pub struct AnswerError {
    kind: AnswerErrorKind,
}

impl AnswerError {
    pub fn kind(&self) -> AnswerErrorKind {
        self.kind
    }

    fn from_new_form_answer_error(err: form_answer::NewFormAnswerError) -> Self {
        let kind = match err.kind() {
            form_answer::NewFormAnswerErrorKind::MismatchedItemsLength => {
                AnswerErrorKind::MismatchedItemsLength
            }
            form_answer::NewFormAnswerErrorKind::MismatchedItemId { expected, got } => {
                AnswerErrorKind::MismatchedItemId { expected, got }
            }
            form_answer::NewFormAnswerErrorKind::InvalidItem { id, kind } => {
                AnswerErrorKind::InvalidItem { id, kind }
            }
            form_answer::NewFormAnswerErrorKind::AlreadyAnswered => {
                AnswerErrorKind::AlreadyAnswered
            }
        };

        AnswerError { kind }
    }
}

impl Form {
    pub fn new(
        author: &User,
        name: FormName,
        description: FormDescription,
        period: FormPeriod,
        items: FormItems,
        condition: FormCondition,
        answer_notification_webhook: Option<String>,
    ) -> Result<Self, NewFormError> {
        author
            .require_permissions(Permissions::CREATE_FORMS)
            .map_err(NewFormError::from_permissions_error)?;

        let created_at = DateTime::now();

        if period.starts_at() <= created_at {
            return Err(NewFormError {
                kind: NewFormErrorKind::TooEarlyPeriodStart,
            });
        }

        Ok(Form::from_content(FormContent {
            id: FormId::from_uuid(Uuid::new_v4()),
            created_at,
            author_id: author.id().clone(),
            name,
            description,
            period,
            items,
            condition,
            answer_notification_webhook,
        }))
    }

    pub async fn answer_by<C>(
        &self,
        ctx: C,
        user: &User,
        project: &Project,
        items: FormAnswerItems,
    ) -> DomainResult<FormAnswer, AnswerError>
    where
        C: FormAnswerRepository,
    {
        if !self.can_be_answered_by(user, project) {
            return Err(DomainError::Domain(AnswerError {
                kind: AnswerErrorKind::NotTargeted,
            }));
        }

        domain_ensure!(self.is_visible_to_with_project(user, project));

        // TODO: Move this to FormAnswer
        let created_at = DateTime::now();
        if !self.period().contains(created_at) {
            return Err(DomainError::Domain(AnswerError {
                kind: AnswerErrorKind::OutOfAnswerPeriod,
            }));
        }

        FormAnswer::new(ctx, user, project, self, items)
            .await
            .map_err(|err| err.map_domain(AnswerError::from_new_form_answer_error))
    }

    pub fn can_be_answered_by(&self, user: &User, project: &Project) -> bool {
        project.is_member(user) && self.condition().check(project)
    }

    /// Restore `Form` from `FormContent`.
    ///
    /// This is intended to be used when the data is taken out of the implementation by [`Form::into_content`]
    /// for persistence, internal serialization, etc.
    /// Use [`Form::new`] to create a form.
    pub fn from_content(content: FormContent) -> Self {
        Form { content }
    }

    /// Convert `Form` into `FormContent`.
    pub fn into_content(self) -> FormContent {
        self.content
    }

    pub fn id(&self) -> FormId {
        self.content.id
    }

    pub fn created_at(&self) -> DateTime {
        self.content.created_at
    }

    pub fn author_id(&self) -> &UserId {
        &self.content.author_id
    }

    pub fn name(&self) -> &FormName {
        &self.content.name
    }

    pub fn description(&self) -> &FormDescription {
        &self.content.description
    }

    pub fn period(&self) -> FormPeriod {
        self.content.period
    }

    pub fn items(&self) -> &FormItems {
        &self.content.items
    }

    pub fn condition(&self) -> &FormCondition {
        &self.content.condition
    }

    pub fn into_items(self) -> FormItems {
        self.content.items
    }

    pub fn answer_notification_webhook(&self) -> &Option<String> {
        &self.content.answer_notification_webhook
    }
}

#[derive(Debug, Clone, Error)]
#[error("insufficient permissions to update forms")]
pub struct NoUpdatePermissionError {
    _priv: (),
}

impl NoUpdatePermissionError {
    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        NoUpdatePermissionError { _priv: () }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetPeriodErrorKind {
    TooEarlyPeriodStart,
    InsufficientPermissions,
}

#[derive(Debug, Clone, Error)]
#[error("failed to update form period")]
pub struct SetPeriodError {
    kind: SetPeriodErrorKind,
}

impl SetPeriodError {
    pub fn kind(&self) -> SetPeriodErrorKind {
        self.kind
    }

    fn from_permissions_error(_err: NoUpdatePermissionError) -> Self {
        SetPeriodError {
            kind: SetPeriodErrorKind::InsufficientPermissions,
        }
    }
}

impl Form {
    fn require_update_permission(&self, user: &User) -> Result<(), NoUpdatePermissionError> {
        let now = DateTime::now();
        let permission = if self.author_id() == user.id() && now < self.period().starts_at() {
            Permissions::UPDATE_NOT_STARTED_OWNING_FORMS
        } else {
            Permissions::UPDATE_ALL_FORMS
        };

        user.require_permissions(permission)
            .map_err(NoUpdatePermissionError::from_permissions_error)
    }

    pub fn set_name(&mut self, user: &User, name: FormName) -> Result<(), NoUpdatePermissionError> {
        self.require_update_permission(user)?;
        self.content.name = name;
        Ok(())
    }

    pub fn set_description(
        &mut self,
        user: &User,
        description: FormDescription,
    ) -> Result<(), NoUpdatePermissionError> {
        self.require_update_permission(user)?;
        self.content.description = description;
        Ok(())
    }

    pub fn set_period(&mut self, user: &User, period: FormPeriod) -> Result<(), SetPeriodError> {
        self.require_update_permission(user)
            .map_err(SetPeriodError::from_permissions_error)?;

        if period.starts_at() <= DateTime::now() {
            return Err(SetPeriodError {
                kind: SetPeriodErrorKind::TooEarlyPeriodStart,
            });
        }

        self.content.period = period;
        Ok(())
    }

    pub fn set_items(
        &mut self,
        user: &User,
        items: FormItems,
    ) -> Result<(), NoUpdatePermissionError> {
        self.require_update_permission(user)?;
        self.content.items = items;
        Ok(())
    }

    pub fn set_condition(
        &mut self,
        user: &User,
        condition: FormCondition,
    ) -> Result<(), NoUpdatePermissionError> {
        self.require_update_permission(user)?;
        self.content.condition = condition;
        Ok(())
    }

    pub fn is_visible_to(&self, user: &User) -> bool {
        user.permissions().contains(Permissions::READ_ALL_FORMS)
    }

    pub fn is_visible_to_with_project(&self, user: &User, project: &Project) -> bool {
        if self.is_visible_to(user) {
            return true;
        }

        self.condition().check(project) && project.is_visible_to(user)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AnswerErrorKind, Form, FormName, FormPeriod, NewFormErrorKind, NoUpdatePermissionError,
        SetPeriodErrorKind,
    };

    use crate::model::{
        date_time::DateTime,
        project::{ProjectAttributes, ProjectCategory},
        project_query::{ProjectQuery, ProjectQueryConjunction},
    };
    use crate::test::model as test_model;
    use crate::DomainError;

    #[test]
    fn test_visibility_general() {
        let user = test_model::new_general_user();
        let operator = test_model::new_operator_user();
        let form = test_model::new_form(operator.id().clone());
        assert!(!form.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_committee() {
        let user = test_model::new_committee_user();
        let operator = test_model::new_operator_user();
        let form = test_model::new_form(operator.id().clone());
        assert!(form.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_operator() {
        let user = test_model::new_operator_user();
        let operator = test_model::new_operator_user();
        let form = test_model::new_form(operator.id().clone());
        assert!(form.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_general_via_project() {
        let user = test_model::new_general_user();
        let user_project = test_model::new_general_project(user.id().clone());
        let operator = test_model::new_operator_user();
        let tautology_query = ProjectQuery::from_conjunctions(vec![ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        }])
        .unwrap();
        let form = test_model::new_form_with_query(operator.id().clone(), tautology_query);
        assert!(form.is_visible_to_with_project(&user, &user_project));
    }

    #[test]
    fn test_create_too_early_period() {
        let author = test_model::new_operator_user();
        let starts_at = DateTime::from_utc(chrono::Utc::now() - chrono::Duration::hours(1));
        let period = FormPeriod::from_datetime(starts_at, DateTime::now()).unwrap();
        assert!(matches!(
            Form::new(
                &author,
                test_model::mock_form_name(),
                test_model::mock_form_description(),
                period,
                test_model::new_form_items(),
                test_model::mock_form_condition(),
                test_model::mock_form_answer_notification_webhook()
            ),
            Err(err)
            if err.kind() == NewFormErrorKind::TooEarlyPeriodStart
        ));
    }

    #[test]
    fn test_create_permission_general() {
        let author = test_model::new_general_user();
        assert!(matches!(
            Form::new(
                &author,
                test_model::mock_form_name(),
                test_model::mock_form_description(),
                test_model::new_form_period_from_now(),
                test_model::new_form_items(),
                test_model::mock_form_condition(),
                test_model::mock_form_answer_notification_webhook()
            ),
            Err(err)
            if err.kind() == NewFormErrorKind::InsufficientPermissions
        ));
    }

    #[test]
    fn test_create_permission_committee() {
        let author = test_model::new_committee_user();
        assert!(matches!(
            Form::new(
                &author,
                test_model::mock_form_name(),
                test_model::mock_form_description(),
                test_model::new_form_period_from_now(),
                test_model::new_form_items(),
                test_model::mock_form_condition(),
                test_model::mock_form_answer_notification_webhook()
            ),
            Err(err)
            if err.kind() == NewFormErrorKind::InsufficientPermissions
        ));
    }

    #[test]
    fn test_create_ok() {
        let author = test_model::new_operator_user();
        let starts_at = DateTime::from_utc(chrono::Utc::now() + chrono::Duration::hours(1));
        let ends_at = DateTime::from_utc(chrono::Utc::now() + chrono::Duration::hours(2));
        let period = FormPeriod::from_datetime(starts_at, ends_at).unwrap();
        assert!(matches!(
            Form::new(
                &author,
                test_model::mock_form_name(),
                test_model::mock_form_description(),
                period,
                test_model::new_form_items(),
                test_model::mock_form_condition(),
                test_model::mock_form_answer_notification_webhook()
            ),
            Ok(got)
            if got.author_id() == author.id()
        ));
    }

    // TODO: test answer period

    #[tokio::test]
    async fn test_answer_not_targeted() {
        let user = test_model::new_general_user();
        let project = test_model::new_stage_project(user.id().clone());
        let query = ProjectQuery::from_conjunctions(vec![ProjectQueryConjunction {
            category: Some(ProjectCategory::General),
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        }])
        .unwrap();
        let operator = test_model::new_operator_user();
        let form = test_model::new_form_with_query(operator.id().clone(), query);
        let app = crate::test::build_mock_app()
            .users(vec![user.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .build();
        let items = test_model::mock_form_answer_items(form.items());
        assert!(matches!(
            form.answer_by(&app, &user, &project, items).await,
            Err(DomainError::Domain(err))
            if err.kind() == AnswerErrorKind::NotTargeted
        ));
    }

    #[tokio::test]
    async fn test_answer_already_answered() {
        let user = test_model::new_general_user();
        let project = test_model::new_stage_project(user.id().clone());
        let query = ProjectQuery::from_conjunctions(vec![ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        }])
        .unwrap();
        let operator = test_model::new_operator_user();
        let form = test_model::new_form_with_query(operator.id().clone(), query);
        let answer = test_model::new_form_answer(user.id().clone(), &project, &form);
        let app = crate::test::build_mock_app()
            .users(vec![user.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .answers(vec![answer.clone()])
            .build();
        let items = test_model::mock_form_answer_items(form.items());
        assert!(matches!(
            form.answer_by(&app, &user, &project, items).await,
            Err(DomainError::Domain(err))
            if err.kind() == AnswerErrorKind::AlreadyAnswered
        ));
    }

    #[tokio::test]
    async fn test_answer_ok() {
        let user = test_model::new_general_user();
        let project = test_model::new_stage_project(user.id().clone());
        let query = ProjectQuery::from_conjunctions(vec![ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        }])
        .unwrap();
        let operator = test_model::new_operator_user();
        let form = test_model::new_form_with_query(operator.id().clone(), query);
        let app = crate::test::build_mock_app()
            .users(vec![user.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .build();
        let items = test_model::mock_form_answer_items(form.items());
        assert!(matches!(
            form.answer_by(&app, &user, &project, items).await,
            Ok(answer)
            if answer.form_id() == form.id()
        ));
    }

    #[test]
    fn test_update_name_other_operator() {
        let author = test_model::new_operator_user();
        let other = test_model::new_operator_user();
        let period = test_model::new_form_period_with_hours_from_now(1);
        let mut form = test_model::new_form_with_period(author.id().clone(), period);
        let name = test_model::mock_form_name();
        assert!(matches!(
            form.set_name(&other, name),
            Err(NoUpdatePermissionError { .. })
        ));
    }

    #[test]
    fn test_update_name_other_admin_after_start() {
        let author = test_model::new_operator_user();
        let other = test_model::new_admin_user();
        let period = test_model::new_form_period_with_hours_from_now(-1);
        let mut form = test_model::new_form_with_period(author.id().clone(), period);
        let name = FormName::from_string("アアア").unwrap();
        form.set_name(&other, name.clone()).unwrap();
        assert_eq!(form.name(), &name);
    }

    #[test]
    fn test_update_name_author_committee() {
        let author = test_model::new_committee_user();
        let period = test_model::new_form_period_with_hours_from_now(1);
        let mut form = test_model::new_form_with_period(author.id().clone(), period);
        let name = test_model::mock_form_name();
        assert!(matches!(
            form.set_name(&author, name),
            Err(NoUpdatePermissionError { .. })
        ));
    }

    #[test]
    fn test_update_name_author_operator_before_start() {
        let author = test_model::new_operator_user();
        let period = test_model::new_form_period_with_hours_from_now(1);
        let mut form = test_model::new_form_with_period(author.id().clone(), period);
        let name = FormName::from_string("アアア").unwrap();
        form.set_name(&author, name.clone()).unwrap();
        assert_eq!(form.name(), &name);
    }

    #[test]
    fn test_update_name_author_operator_after_start() {
        let author = test_model::new_operator_user();
        let period = test_model::new_form_period_with_hours_from_now(-1);
        let mut form = test_model::new_form_with_period(author.id().clone(), period);
        let name = FormName::from_string("アアア").unwrap();
        assert!(matches!(
            form.set_name(&author, name),
            Err(NoUpdatePermissionError { .. })
        ));
    }

    #[test]
    fn test_update_period_too_early() {
        let author = test_model::new_operator_user();
        let period = test_model::new_form_period_with_hours_from_now(1);
        let mut form = test_model::new_form_with_period(author.id().clone(), period);
        let starts_at = DateTime::from_utc(chrono::Utc::now() - chrono::Duration::hours(1));
        let period = FormPeriod::from_datetime(starts_at, DateTime::now()).unwrap();
        assert!(matches!(
            form.set_period(&author, period),
            Err(err)
            if err.kind() == SetPeriodErrorKind::TooEarlyPeriodStart
        ));
    }
}
