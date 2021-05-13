use crate::error::{UseCaseError, UseCaseResult};
use crate::interface;
use crate::model::form::{Form, FormCondition, FormItem};

use anyhow::Context;
use sos21_domain::context::{FormRepository, Login};
use sos21_domain::model::{date_time::DateTime, form};

#[derive(Debug, Clone)]
pub struct Input {
    pub name: String,
    pub description: String,
    pub starts_at: chrono::DateTime<chrono::Utc>,
    pub ends_at: chrono::DateTime<chrono::Utc>,
    pub items: Vec<FormItem>,
    pub condition: FormCondition,
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidName,
    InvalidDescription,
    InvalidPeriod,
    TooEarlyPeriodStart,
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

    fn from_new_form_error(err: form::NewFormError) -> Self {
        match err.kind() {
            form::NewFormErrorKind::TooEarlyPeriodStart => Error::TooEarlyPeriodStart,
            form::NewFormErrorKind::InsufficientPermissions => Error::InsufficientPermissions,
        }
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<Form, Error>
where
    C: FormRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let name = form::FormName::from_string(input.name)
        .map_err(|err| UseCaseError::UseCase(Error::from_name_error(err)))?;
    let description = form::FormDescription::from_string(input.description)
        .map_err(|err| UseCaseError::UseCase(Error::from_description_error(err)))?;
    let items = interface::form::to_form_items(input.items)
        .map_err(|err| UseCaseError::UseCase(Error::from_items_error(err)))?;
    let condition = interface::form::to_form_condition(input.condition)
        .map_err(|err| UseCaseError::UseCase(Error::from_condition_error(err)))?;
    let starts_at = DateTime::from_utc(input.starts_at);
    let ends_at = DateTime::from_utc(input.ends_at);
    let period = form::FormPeriod::from_datetime(starts_at, ends_at)
        .map_err(|err| UseCaseError::UseCase(Error::from_period_error(err)))?;

    let form = form::Form::new(login_user, name, description, period, items, condition)
        .map_err(|err| UseCaseError::UseCase(Error::from_new_form_error(err)))?;
    ctx.store_form(form.clone())
        .await
        .context("Failed to store a form")?;
    use_case_ensure!(form.is_visible_to(login_user));
    Ok(Form::from_entity(form))
}

#[cfg(test)]
mod tests {
    use crate::model::{
        form::{FormCondition, FormItem},
        user::UserId,
    };
    use crate::{create_form, get_form, UseCaseError};
    use sos21_domain::{model::date_time, test};

    // Checks that the normal user cannot create forms.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let period = test::model::mock_form_period_with_start(date_time::DateTime::from_utc(
            chrono::Utc::now() + chrono::Duration::hours(1),
        ));
        let input = create_form::Input {
            name: test::model::mock_form_name().into_string(),
            description: test::model::mock_form_description().into_string(),
            starts_at: period.starts_at().utc(),
            ends_at: period.ends_at().utc(),
            items: test::model::new_form_items()
                .into_items()
                .map(FormItem::from_entity)
                .collect(),
            condition: FormCondition::from_entity(test::model::mock_form_condition()),
        };

        assert!(matches!(
            create_form::run(&app, input).await,
            Err(UseCaseError::UseCase(
                create_form::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user cannot create forms.
    #[tokio::test]
    async fn test_committee() {
        let user = test::model::new_committee_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let period = test::model::mock_form_period_with_start(date_time::DateTime::from_utc(
            chrono::Utc::now() + chrono::Duration::hours(1),
        ));
        let input = create_form::Input {
            name: test::model::mock_form_name().into_string(),
            description: test::model::mock_form_description().into_string(),
            starts_at: period.starts_at().utc(),
            ends_at: period.ends_at().utc(),
            items: test::model::new_form_items()
                .into_items()
                .map(FormItem::from_entity)
                .collect(),
            condition: FormCondition::from_entity(test::model::mock_form_condition()),
        };

        assert!(matches!(
            create_form::run(&app, input).await,
            Err(UseCaseError::UseCase(
                create_form::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the privileged committee user can create forms.
    #[tokio::test]
    async fn test_operator() {
        let user = test::model::new_operator_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let period = test::model::mock_form_period_with_start(date_time::DateTime::from_utc(
            chrono::Utc::now() + chrono::Duration::hours(1),
        ));
        let name = "テストテストテスト".to_string();
        let input = create_form::Input {
            name: name.clone(),
            description: test::model::mock_form_description().into_string(),
            starts_at: period.starts_at().utc(),
            ends_at: period.ends_at().utc(),
            items: test::model::new_form_items()
                .into_items()
                .map(FormItem::from_entity)
                .collect(),
            condition: FormCondition::from_entity(test::model::mock_form_condition()),
        };

        let result = create_form::run(&app, input).await;
        assert!(result.is_ok());

        let got = result.unwrap();
        assert!(got.name == name);
        assert!(got.author_id == UserId::from_entity(user.id().clone()));

        assert!(matches!(get_form::run(&app, got.id).await, Ok(_)));
    }

    #[tokio::test]
    async fn test_create_too_early_period() {
        let user = test::model::new_operator_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let period = test::model::mock_form_period_with_start(date_time::DateTime::from_utc(
            chrono::Utc::now() - chrono::Duration::hours(1),
        ));
        let input = create_form::Input {
            name: test::model::mock_form_name().into_string(),
            description: test::model::mock_form_description().into_string(),
            starts_at: period.starts_at().utc(),
            ends_at: period.ends_at().utc(),
            items: test::model::new_form_items()
                .into_items()
                .map(FormItem::from_entity)
                .collect(),
            condition: FormCondition::from_entity(test::model::mock_form_condition()),
        };

        assert!(matches!(
            create_form::run(&app, input).await,
            Err(UseCaseError::UseCase(
                create_form::Error::TooEarlyPeriodStart
            ))
        ));
    }
}
