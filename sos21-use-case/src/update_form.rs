use crate::error::{UseCaseError, UseCaseResult};
use crate::interface;
use crate::model::form::{Form, FormCondition, FormId, FormItem};

use anyhow::Context;
use sos21_domain::context::{FormRepository, Login};
use sos21_domain::model::{date_time, form};

#[derive(Debug, Clone)]
pub struct Input {
    pub id: FormId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub starts_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ends_at: Option<chrono::DateTime<chrono::Utc>>,
    pub items: Option<Vec<FormItem>>,
    pub condition: Option<FormCondition>,
}

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
    InvalidName,
    InvalidDescription,
    InvalidPeriod,
    TooEarlyPeriodStart,
    AlreadyStarted,
    InvalidItems(interface::form::FormItemsError),
    InvalidCondition(interface::form::FormConditionError),
    InsufficientPermissions,
}

impl Error {
    fn from_name_error(_err: form::name::NameError) -> Self {
        Error::InvalidName
    }

    fn from_description_error(_err: form::description::DescriptionError) -> Self {
        Error::InvalidDescription
    }

    fn from_period_error(_err: form::period::PeriodError) -> Self {
        Error::InvalidPeriod
    }

    fn from_items_error(err: interface::form::FormItemsError) -> Self {
        Error::InvalidItems(err)
    }

    fn from_condition_error(err: interface::form::FormConditionError) -> Self {
        Error::InvalidCondition(err)
    }

    fn from_update_error(_err: form::NoUpdatePermissionError) -> Self {
        Error::InsufficientPermissions
    }

    fn from_set_period_error(err: form::SetPeriodError) -> Self {
        match err.kind() {
            form::SetPeriodErrorKind::InsufficientPermissions => Error::InsufficientPermissions,
            form::SetPeriodErrorKind::TooEarlyPeriodStart => Error::TooEarlyPeriodStart,
        }
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<Form, Error>
where
    C: FormRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let result = ctx
        .get_form(input.id.into_entity())
        .await
        .context("Failed to get a form")?;
    let mut form = match result {
        Some(form) if form.is_visible_to(login_user) => form,
        _ => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    if date_time::DateTime::now() >= form.period().starts_at() {
        return Err(UseCaseError::UseCase(Error::AlreadyStarted));
    }

    if let Some(name) = input.name {
        let name = form::FormName::from_string(name)
            .map_err(|err| UseCaseError::UseCase(Error::from_name_error(err)))?;
        form.set_name(login_user, name)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(description) = input.description {
        let description = form::FormDescription::from_string(description)
            .map_err(|err| UseCaseError::UseCase(Error::from_description_error(err)))?;
        form.set_description(login_user, description)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(items) = input.items {
        let items = interface::form::to_form_items(items)
            .map_err(|err| UseCaseError::UseCase(Error::from_items_error(err)))?;
        form.set_items(login_user, items)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(condition) = input.condition {
        let condition = interface::form::to_form_condition(condition)
            .map_err(|err| UseCaseError::UseCase(Error::from_condition_error(err)))?;
        form.set_condition(login_user, condition)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if input.starts_at.is_some() || input.ends_at.is_some() {
        let mut period = form.period();

        if let Some(starts_at) = input.starts_at {
            period
                .set_starts_at(date_time::DateTime::from_utc(starts_at))
                .map_err(|err| UseCaseError::UseCase(Error::from_period_error(err)))?;
        }

        if let Some(ends_at) = input.ends_at {
            period
                .set_ends_at(date_time::DateTime::from_utc(ends_at))
                .map_err(|err| UseCaseError::UseCase(Error::from_period_error(err)))?;
        }

        form.set_period(login_user, period)
            .map_err(|err| UseCaseError::UseCase(Error::from_set_period_error(err)))?;
    }

    ctx.store_form(form.clone())
        .await
        .context("Failed to store a form")?;
    use_case_ensure!(form.is_visible_to(login_user));
    Ok(Form::from_entity(form))
}

#[cfg(test)]
mod tests {
    use crate::model::form::FormId;
    use crate::{update_form, UseCaseError};

    use sos21_domain::test;

    #[tokio::test]
    async fn test_name_other_general() {
        let author = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let period = test::model::new_form_period_with_hours_from_now(1);
        let form = test::model::new_form_with_period(author.id().clone(), period);

        let app = test::build_mock_app()
            .users(vec![author.clone(), other.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(other)
            .await;

        let input = update_form::Input {
            id: FormId::from_entity(form.id()),
            name: Some(test::model::mock_form_name().into_string()),
            description: None,
            starts_at: None,
            ends_at: None,
            items: None,
            condition: None,
        };
        assert!(matches!(
            update_form::run(&app, input).await,
            Err(UseCaseError::UseCase(update_form::Error::NotFound))
        ));
    }

    #[tokio::test]
    async fn test_name_other_operator() {
        let author = test::model::new_operator_user();
        let other = test::model::new_operator_user();
        let period = test::model::new_form_period_with_hours_from_now(1);
        let form = test::model::new_form_with_period(author.id().clone(), period);

        let app = test::build_mock_app()
            .users(vec![author.clone(), other.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(other)
            .await;

        let input = update_form::Input {
            id: FormId::from_entity(form.id()),
            name: Some(test::model::mock_form_name().into_string()),
            description: None,
            starts_at: None,
            ends_at: None,
            items: None,
            condition: None,
        };
        assert!(matches!(
            update_form::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_form::Error::InsufficientPermissions
            ))
        ));
    }

    #[tokio::test]
    async fn test_name_author_committee() {
        let author = test::model::new_committee_user();
        let period = test::model::new_form_period_with_hours_from_now(1);
        let form = test::model::new_form_with_period(author.id().clone(), period);

        let app = test::build_mock_app()
            .users(vec![author.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(author)
            .await;

        let input = update_form::Input {
            id: FormId::from_entity(form.id()),
            name: Some(test::model::mock_form_name().into_string()),
            description: None,
            starts_at: None,
            ends_at: None,
            items: None,
            condition: None,
        };
        assert!(matches!(
            update_form::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_form::Error::InsufficientPermissions
            ))
        ));
    }

    #[tokio::test]
    async fn test_name_author_operator_before_start() {
        let author = test::model::new_operator_user();
        let period = test::model::new_form_period_with_hours_from_now(1);
        let form = test::model::new_form_with_period(author.id().clone(), period);

        let app = test::build_mock_app()
            .users(vec![author.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(author)
            .await;

        let name = "アアアアア".to_string();
        let input = update_form::Input {
            id: FormId::from_entity(form.id()),
            name: Some(name.clone()),
            description: None,
            starts_at: None,
            ends_at: None,
            items: None,
            condition: None,
        };
        assert!(matches!(
            update_form::run(&app, input).await,
            Ok(got)
            if got.name == name
        ));
    }

    #[tokio::test]
    async fn test_name_author_operator_after_start() {
        let author = test::model::new_operator_user();
        let period = test::model::new_form_period_with_hours_from_now(-1);
        let form = test::model::new_form_with_period(author.id().clone(), period);

        let app = test::build_mock_app()
            .users(vec![author.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(author)
            .await;

        let name = "アアアアア".to_string();
        let input = update_form::Input {
            id: FormId::from_entity(form.id()),
            name: Some(name.clone()),
            description: None,
            starts_at: None,
            ends_at: None,
            items: None,
            condition: None,
        };
        assert!(matches!(
            update_form::run(&app, input).await,
            Err(UseCaseError::UseCase(update_form::Error::AlreadyStarted))
        ));
    }

    #[tokio::test]
    async fn test_period_too_early() {
        let author = test::model::new_operator_user();
        let period = test::model::new_form_period_with_hours_from_now(1);
        let form = test::model::new_form_with_period(author.id().clone(), period);

        let app = test::build_mock_app()
            .users(vec![author.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(author)
            .await;

        let starts_at = chrono::Utc::now() - chrono::Duration::hours(1);
        let input = update_form::Input {
            id: FormId::from_entity(form.id()),
            name: None,
            description: None,
            starts_at: Some(starts_at),
            ends_at: None,
            items: None,
            condition: None,
        };
        assert!(matches!(
            update_form::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_form::Error::TooEarlyPeriodStart
            ))
        ));
    }
}
