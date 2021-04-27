use crate::error::{UseCaseError, UseCaseResult};
use crate::model::registration_form_answer::{RegistrationFormAnswer, RegistrationFormAnswerId};

use anyhow::Context;
use sos21_domain::context::{Login, RegistrationFormAnswerRepository};
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
    answer_id: RegistrationFormAnswerId,
) -> UseCaseResult<RegistrationFormAnswer, Error>
where
    C: RegistrationFormAnswerRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    login_user
        .require_permissions(Permissions::READ_ALL_REGISTRATION_FORM_ANSWERS)
        .map_err(|err| UseCaseError::UseCase(Error::from_permissions_error(err)))?;

    let result = ctx
        .get_registration_form_answer(answer_id.into_entity())
        .await
        .context("Failed to get a registration form answer")?;
    let answer = match result {
        Some(answer) => answer,
        _ => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    use_case_ensure!(answer.is_visible_to(login_user));
    Ok(RegistrationFormAnswer::from_entity(answer))
}

#[cfg(test)]
mod tests {
    use crate::model::registration_form_answer::RegistrationFormAnswerId;
    use crate::{get_registration_form_answer, UseCaseError};
    use sos21_domain::test;

    // Checks that the normal user cannot read the form answers directly.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let operator = test::model::new_operator_user();
        let pending_project = test::model::new_general_pending_project(user.id.clone());
        let registration_form = test::model::new_registration_form(operator.id.clone());
        let answer = test::model::new_registration_form_answer_with_pending_project(
            user.id.clone(),
            pending_project.id(),
            &registration_form,
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), operator.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_registration_form_answer::run(
                &app,
                RegistrationFormAnswerId::from_entity(answer.id)
            )
            .await,
            Err(UseCaseError::UseCase(
                get_registration_form_answer::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user can read the registration form answers directly.
    #[tokio::test]
    async fn test_committee_other() {
        let user = test::model::new_committee_user();
        let operator = test::model::new_operator_user();
        let pending_project = test::model::new_general_pending_project(user.id.clone());
        let registration_form = test::model::new_registration_form(operator.id.clone());
        let answer = test::model::new_registration_form_answer_with_pending_project(
            user.id.clone(),
            pending_project.id(),
            &registration_form,
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), operator.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let answer_id = RegistrationFormAnswerId::from_entity(answer.id);
        assert!(matches!(
            get_registration_form_answer::run(&app, answer_id).await,
            Ok(got)
            if got.id == answer_id
        ));
    }

    // Checks that the privileged committee user can read registration form answers of the others' pending projects.
    #[tokio::test]
    async fn test_operator_other() {
        let user = test::model::new_operator_user();
        let operator = test::model::new_operator_user();
        let pending_project = test::model::new_general_pending_project(user.id.clone());
        let registration_form = test::model::new_registration_form(operator.id.clone());
        let answer = test::model::new_registration_form_answer_with_pending_project(
            user.id.clone(),
            pending_project.id(),
            &registration_form,
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), operator.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let answer_id = RegistrationFormAnswerId::from_entity(answer.id);
        assert!(matches!(
            get_registration_form_answer::run(&app, answer_id).await,
            Ok(got)
            if got.id == answer_id
        ));
    }
}
