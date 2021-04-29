use crate::error::{UseCaseError, UseCaseResult};
use crate::model::registration_form::{RegistrationForm, RegistrationFormId};

use anyhow::Context;
use sos21_domain::context::{Login, RegistrationFormRepository};
use sos21_domain::model::{permissions::Permissions, user};

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
    InsufficientPermissions,
}

impl Error {
    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        Error::InsufficientPermissions
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(
    ctx: &Login<C>,
    registration_form_id: RegistrationFormId,
) -> UseCaseResult<RegistrationForm, Error>
where
    C: RegistrationFormRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    login_user
        .require_permissions(Permissions::READ_ALL_REGISTRATION_FORMS)
        .map_err(|err| UseCaseError::UseCase(Error::from_permissions_error(err)))?;

    let result = ctx
        .get_registration_form(registration_form_id.into_entity())
        .await
        .context("Failed to get a registration_form")?;
    let registration_form = match result {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    use_case_ensure!(registration_form.is_visible_to(login_user));
    Ok(RegistrationForm::from_entity(registration_form))
}

#[cfg(test)]
mod tests {
    use crate::model::registration_form::RegistrationFormId;
    use crate::{get_registration_form, UseCaseError};
    use sos21_domain::test;

    // Checks that the normal user cannot read others' registration_form.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let registration_form_other = test::model::new_registration_form(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .registration_forms(vec![registration_form_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_registration_form::run(
                &app,
                RegistrationFormId::from_entity(registration_form_other.id)
            )
            .await,
            Err(UseCaseError::UseCase(
                get_registration_form::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user can read the others' registration_form.
    #[tokio::test]
    async fn test_committee_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let registration_form_other = test::model::new_registration_form(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .registration_forms(vec![registration_form_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let registration_form_id = RegistrationFormId::from_entity(registration_form_other.id);
        assert!(matches!(
            get_registration_form::run(&app, registration_form_id).await,
            Ok(got)
            if got.id == registration_form_id && got.name == registration_form_other.name.into_string()
        ));
    }

    // Checks that the privileged committee user can read the others' registration_form.
    #[tokio::test]
    async fn test_operator_other() {
        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let registration_form_other = test::model::new_registration_form(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .registration_forms(vec![registration_form_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let registration_form_id = RegistrationFormId::from_entity(registration_form_other.id);
        assert!(matches!(
            get_registration_form::run(&app, registration_form_id).await,
            Ok(got)
            if got.id == registration_form_id && got.name == registration_form_other.name.into_string()
        ));
    }

    // Checks that the `NotFound` is returned when the unprivileged committee user
    // attempt to read the non-existing others' project.
    #[tokio::test]
    async fn test_committee_nonexisting_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let registration_form_other = test::model::new_registration_form(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .registration_forms(vec![]) // no registration_forms created
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_registration_form::run(
                &app,
                RegistrationFormId::from_entity(registration_form_other.id)
            )
            .await,
            Err(UseCaseError::UseCase(
                get_registration_form::Error::NotFound
            ))
        ));
    }
}
