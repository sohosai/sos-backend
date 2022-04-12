use crate::error::{UseCaseError, UseCaseResult};
use crate::interface;
use crate::model::form::FormItem;
use crate::model::project_query::ProjectQuery;
use crate::model::registration_form::RegistrationForm;

use anyhow::Context;
use sos21_domain::context::{ConfigContext, Login, RegistrationFormRepository};
use sos21_domain::model::permissions::Permissions;
use sos21_domain::model::{date_time::DateTime, registration_form, user};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub name: String,
    pub description: String,
    pub items: Vec<FormItem>,
    pub query: ProjectQuery,
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidName,
    InvalidDescription,
    InvalidItems(interface::form::FormItemsError),
    InvalidQuery(interface::project_query::ProjectQueryError),
    InsufficientPermissions,
    AlreadyStartedProjectCreationPeriod,
}

impl Error {
    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        Error::InsufficientPermissions
    }

    fn from_name_error(_err: registration_form::name::NameError) -> Self {
        Error::InvalidName
    }

    fn from_description_error(_err: registration_form::description::DescriptionError) -> Self {
        Error::InvalidDescription
    }

    fn from_items_error(err: interface::form::FormItemsError) -> Self {
        Error::InvalidItems(err)
    }

    fn from_query_error(err: interface::project_query::ProjectQueryError) -> Self {
        Error::InvalidQuery(err)
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<RegistrationForm, Error>
where
    C: RegistrationFormRepository + ConfigContext + Send + Sync,
{
    let login_user = ctx.login_user();

    login_user
        .require_permissions(Permissions::CREATE_REGISTRATION_FORMS)
        .map_err(|err| UseCaseError::UseCase(Error::from_permissions_error(err)))?;

    let query = interface::project_query::to_project_query(input.query)
        .map_err(|err| UseCaseError::UseCase(Error::from_query_error(err)))?;

    // TODO: Move this constraint to domain
    for category in query.possible_categories() {
        if ctx
            .project_creation_period_for(category)
            .contains(DateTime::now())
        {
            return Err(UseCaseError::UseCase(
                Error::AlreadyStartedProjectCreationPeriod,
            ));
        }
    }

    let name = registration_form::RegistrationFormName::from_string(input.name)
        .map_err(|err| UseCaseError::UseCase(Error::from_name_error(err)))?;
    let description =
        registration_form::RegistrationFormDescription::from_string(input.description)
            .map_err(|err| UseCaseError::UseCase(Error::from_description_error(err)))?;
    let items = interface::form::to_form_items(input.items)
        .map_err(|err| UseCaseError::UseCase(Error::from_items_error(err)))?;

    let registration_form = registration_form::RegistrationForm {
        id: registration_form::RegistrationFormId::from_uuid(Uuid::new_v4()),
        created_at: DateTime::now(),
        author_id: login_user.id().clone(),
        name,
        description,
        items,
        query,
    };
    ctx.store_registration_form(registration_form.clone())
        .await
        .context("Failed to store a registration form")?;
    use_case_ensure!(registration_form.is_visible_to(login_user));
    Ok(RegistrationForm::from_entity(registration_form))
}

#[cfg(test)]
mod tests {
    use crate::model::{
        form::FormItem,
        project::ProjectCategory,
        project_query::{ProjectQuery, ProjectQueryConjunction},
        user::UserId,
    };
    use crate::{create_registration_form, get_registration_form, UseCaseError};
    use sos21_domain::{model::project, test};

    // Checks that the normal user cannot create registration forms.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let period = test::model::new_project_creation_period_with_hours_from_now(1);

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .project_creation_period_for(project::ProjectCategory::GeneralOnline, period)
            .build()
            .login_as(user.clone())
            .await;

        let input = create_registration_form::Input {
            name: test::model::mock_registration_form_name().into_string(),
            description: test::model::mock_registration_form_description().into_string(),
            items: test::model::new_form_items()
                .into_items()
                .map(FormItem::from_entity)
                .collect(),
            query: ProjectQuery::from_entity(test::model::mock_project_query()),
        };

        assert!(matches!(
            create_registration_form::run(&app, input).await,
            Err(UseCaseError::UseCase(
                create_registration_form::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user cannot create registration forms.
    #[tokio::test]
    async fn test_committee() {
        let user = test::model::new_committee_user();
        let period = test::model::new_project_creation_period_with_hours_from_now(1);

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .project_creation_period_for(project::ProjectCategory::GeneralOnline, period)
            .build()
            .login_as(user.clone())
            .await;

        let input = create_registration_form::Input {
            name: test::model::mock_registration_form_name().into_string(),
            description: test::model::mock_registration_form_description().into_string(),
            items: test::model::new_form_items()
                .into_items()
                .map(FormItem::from_entity)
                .collect(),
            query: ProjectQuery::from_entity(test::model::mock_project_query()),
        };

        assert!(matches!(
            create_registration_form::run(&app, input).await,
            Err(UseCaseError::UseCase(
                create_registration_form::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the privileged committee user can create registration forms.
    #[tokio::test]
    async fn test_operator() {
        let user = test::model::new_operator_user();
        let period = test::model::new_project_creation_period_with_hours_from_now(1);

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .project_creation_period_for(project::ProjectCategory::GeneralOnline, period)
            .build()
            .login_as(user.clone())
            .await;

        let name = "テストテストテスト".to_string();
        let input = create_registration_form::Input {
            name: name.clone(),
            description: test::model::mock_registration_form_description().into_string(),
            items: test::model::new_form_items()
                .into_items()
                .map(FormItem::from_entity)
                .collect(),
            query: ProjectQuery(vec![ProjectQueryConjunction {
                category: Some(ProjectCategory::GeneralOnline),
                attributes: vec![],
            }]),
        };

        let got = create_registration_form::run(&app, input).await.unwrap();
        assert!(got.name == name);
        assert!(got.author_id == UserId::from_entity(user.id().clone()));

        assert!(matches!(
            get_registration_form::run(&app, got.id).await,
            Ok(_)
        ));
    }

    // TODO: test in period
}
