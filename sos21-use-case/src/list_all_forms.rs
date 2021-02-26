use crate::error::{UseCaseError, UseCaseResult};
use crate::model::form::Form;

use anyhow::Context;
use sos21_domain::context::{FormRepository, Login};
use sos21_domain::model::permissions::Permissions;

#[derive(Debug, Clone)]
pub enum Error {
    InsufficientPermissions,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>) -> UseCaseResult<Vec<Form>, Error>
where
    C: FormRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    if login_user
        .require_permissions(Permissions::READ_ALL_FORMS)
        .is_err()
    {
        return Err(UseCaseError::UseCase(Error::InsufficientPermissions));
    }

    let forms = ctx.list_forms().await.context("Failed to list forms")?;
    use_case_ensure!(forms.iter().all(|form| form.is_visible_to(login_user)));
    Ok(forms.into_iter().map(Form::from_entity).collect())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::model::form::FormId;
    use crate::{list_all_forms, UseCaseError};
    use sos21_domain::context::Login;
    use sos21_domain::model as domain;
    use sos21_domain::test;

    async fn prepare_app(
        login_user: domain::user::User,
    ) -> (Login<test::context::MockApp>, HashSet<FormId>) {
        let other = test::model::new_general_user();
        let form1 = test::model::new_form(login_user.id.clone());
        let form2 = test::model::new_form(other.id.clone());

        let mut expected = HashSet::new();
        expected.insert(FormId::from_entity(form1.id));
        expected.insert(FormId::from_entity(form2.id));

        let app = test::build_mock_app()
            .users(vec![login_user.clone(), other.clone()])
            .forms(vec![form1.clone(), form2.clone()])
            .build()
            .login_as(login_user.clone())
            .await;
        (app, expected)
    }

    // Checks that the normal user cannot list forms.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let (app, _) = prepare_app(user).await;

        assert!(matches!(
            list_all_forms::run(&app).await,
            Err(UseCaseError::UseCase(
                list_all_forms::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user can list forms.
    #[tokio::test]
    async fn test_committee() {
        let user = test::model::new_committee_user();
        let (app, expected) = prepare_app(user).await;

        let result = list_all_forms::run(&app).await;
        assert!(result.is_ok());

        let got: HashSet<_> = result.unwrap().into_iter().map(|form| form.id).collect();
        assert_eq!(got, expected);
    }

    // Checks that the privileged committee user can list forms.
    #[tokio::test]
    async fn test_operator() {
        let user = test::model::new_operator_user();
        let (app, expected) = prepare_app(user).await;

        let result = list_all_forms::run(&app).await;
        assert!(result.is_ok());

        let got: HashSet<_> = result.unwrap().into_iter().map(|form| form.id).collect();
        assert_eq!(got, expected);
    }
}
