use crate::error::{UseCaseError, UseCaseResult};
use crate::model::registration_form::RegistrationForm;

use anyhow::Context;
use sos21_domain::context::{Login, RegistrationFormRepository};
use sos21_domain::model::{permissions::Permissions, user};

#[derive(Debug, Clone)]
pub enum Error {
    InsufficientPermissions,
}

impl Error {
    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        Error::InsufficientPermissions
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>) -> UseCaseResult<Vec<RegistrationForm>, Error>
where
    C: RegistrationFormRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    login_user
        .require_permissions(Permissions::READ_ALL_FORMS)
        .map_err(|err| UseCaseError::UseCase(Error::from_permissions_error(err)))?;

    ctx.list_registration_forms()
        .await
        .context("Failed to list registration forms")?
        .into_iter()
        .map(|registration_form| {
            use_case_ensure!(registration_form.is_visible_to(login_user));
            Ok(RegistrationForm::from_entity(registration_form))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::model::registration_form::RegistrationFormId;
    use crate::{list_all_registration_forms, UseCaseError};
    use sos21_domain::context::Login;
    use sos21_domain::model as domain;
    use sos21_domain::test;

    async fn prepare_app(
        login_user: domain::user::User,
    ) -> (Login<test::context::MockApp>, HashSet<RegistrationFormId>) {
        let other = test::model::new_general_user();
        let registration_form1 = test::model::new_registration_form(login_user.id().clone());
        let registration_form2 = test::model::new_registration_form(other.id().clone());

        let mut expected = HashSet::new();
        expected.insert(RegistrationFormId::from_entity(registration_form1.id));
        expected.insert(RegistrationFormId::from_entity(registration_form2.id));

        let app = test::build_mock_app()
            .users(vec![login_user.clone(), other.clone()])
            .registration_forms(vec![registration_form1.clone(), registration_form2.clone()])
            .build()
            .login_as(login_user.clone())
            .await;
        (app, expected)
    }

    // Checks that the normal user cannot list registration forms.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let (app, _) = prepare_app(user).await;

        assert!(matches!(
            list_all_registration_forms::run(&app).await,
            Err(UseCaseError::UseCase(
                list_all_registration_forms::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user can list registration forms.
    #[tokio::test]
    async fn test_committee() {
        let user = test::model::new_committee_user();
        let (app, expected) = prepare_app(user).await;

        let result = list_all_registration_forms::run(&app).await;
        assert!(result.is_ok());

        let got: HashSet<_> = result
            .unwrap()
            .into_iter()
            .map(|registration_form| registration_form.id)
            .collect();
        assert_eq!(got, expected);
    }

    // Checks that the privileged committee user can list registration forms.
    #[tokio::test]
    async fn test_operator() {
        let user = test::model::new_operator_user();
        let (app, expected) = prepare_app(user).await;

        let result = list_all_registration_forms::run(&app).await;
        assert!(result.is_ok());

        let got: HashSet<_> = result
            .unwrap()
            .into_iter()
            .map(|registration_form| registration_form.id)
            .collect();
        assert_eq!(got, expected);
    }
}
