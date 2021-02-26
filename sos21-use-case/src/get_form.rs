use crate::error::{UseCaseError, UseCaseResult};
use crate::model::form::{Form, FormId};

use anyhow::Context;
use sos21_domain::context::{FormRepository, Login};
use sos21_domain::model::permissions::Permissions;

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
    InsufficientPermissions,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, form_id: FormId) -> UseCaseResult<Form, Error>
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

    let result = ctx
        .get_form(form_id.into_entity())
        .await
        .context("Failed to get a form")?;
    let form = match result {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    use_case_ensure!(form.is_visible_to(login_user));
    Ok(Form::from_entity(form))
}

#[cfg(test)]
mod tests {
    use crate::model::form::FormId;
    use crate::{get_form, UseCaseError};
    use sos21_domain::test;

    // Checks that the normal user cannot read others' form.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let form_other = test::model::new_form(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .forms(vec![form_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_form::run(&app, FormId::from_entity(form_other.id)).await,
            Err(UseCaseError::UseCase(
                get_form::Error::InsufficientPermissions
            ))
        ));
    }
    // Checks that the (unprivileged) committee user can read the others' form.
    #[tokio::test]
    async fn test_committee_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let form_other = test::model::new_form(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .forms(vec![form_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let form_id = FormId::from_entity(form_other.id);
        assert!(matches!(
            get_form::run(&app, form_id).await,
            Ok(got)
            if got.id == form_id && got.name == form_other.name.into_string()
        ));
    }

    // Checks that the privileged committee user can read the others' form.
    #[tokio::test]
    async fn test_operator_other() {
        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let form_other = test::model::new_form(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .forms(vec![form_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let form_id = FormId::from_entity(form_other.id);
        assert!(matches!(
            get_form::run(&app, form_id).await,
            Ok(got)
            if got.id == form_id && got.name == form_other.name.into_string()
        ));
    }

    // Checks that the `NotFound` is returned when the unprivileged committee user
    // attempt to read the non-existing others' project.
    #[tokio::test]
    async fn test_committee_nonexisting_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let form_other = test::model::new_form(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .forms(vec![]) // no forms created
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_form::run(&app, FormId::from_entity(form_other.id)).await,
            Err(UseCaseError::UseCase(get_form::Error::NotFound))
        ));
    }
}
