use crate::error::{UseCaseError, UseCaseResult};
use crate::model::{
    registration_form::RegistrationFormId, registration_form_answer::RegistrationFormAnswer,
};

use anyhow::Context;
use sos21_domain::context::{Login, RegistrationFormAnswerRepository, RegistrationFormRepository};
use sos21_domain::model::{permissions::Permissions, user};

#[derive(Debug, Clone)]
pub enum Error {
    RegistrationFormNotFound,
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
) -> UseCaseResult<Vec<RegistrationFormAnswer>, Error>
where
    C: RegistrationFormRepository + RegistrationFormAnswerRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    login_user
        .require_permissions(Permissions::READ_ALL_REGISTRATION_FORM_ANSWERS)
        .map_err(|err| UseCaseError::UseCase(Error::from_permissions_error(err)))?;

    let registration_form_id = registration_form_id.into_entity();
    let answers = ctx
        .list_registration_form_answers(registration_form_id)
        .await
        .context("Failed to list registration form answers")?;
    if answers.is_empty() {
        match ctx.get_registration_form(registration_form_id).await? {
            Some(registration_form) if registration_form.is_visible_to(login_user) => {}
            _ => return Err(UseCaseError::UseCase(Error::RegistrationFormNotFound)),
        }
    }

    answers
        .into_iter()
        .map(|answer| {
            use_case_ensure!(answer.is_visible_to(login_user));
            Ok(RegistrationFormAnswer::from_entity(answer))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::model::{
        registration_form::RegistrationFormId, registration_form_answer::RegistrationFormAnswerId,
    };
    use crate::{list_registration_form_answers, UseCaseError};
    use sos21_domain::context::Login;
    use sos21_domain::model as domain;
    use sos21_domain::test;

    async fn prepare_app(
        login_user: domain::user::User,
    ) -> (
        Login<test::context::MockApp>,
        RegistrationFormId,
        HashSet<RegistrationFormAnswerId>,
    ) {
        let operator = test::model::new_operator_user();

        let project = test::model::new_general_project(login_user.id().clone());
        let pending_project = test::model::new_general_pending_project(login_user.id().clone());

        let registration_form1 = test::model::new_registration_form(operator.id().clone());
        let registration_form1_answer1 = test::model::new_registration_form_answer_with_project(
            login_user.id().clone(),
            project.id(),
            &registration_form1,
        );
        let registration_form1_answer2 =
            test::model::new_registration_form_answer_with_pending_project(
                login_user.id().clone(),
                pending_project.id(),
                &registration_form1,
            );

        let registration_form2 = test::model::new_registration_form(operator.id().clone());
        let registration_form2_answer1 = test::model::new_registration_form_answer_with_project(
            login_user.id().clone(),
            project.id(),
            &registration_form2,
        );
        let registration_form2_answer2 =
            test::model::new_registration_form_answer_with_pending_project(
                login_user.id().clone(),
                pending_project.id(),
                &registration_form2,
            );

        let registration_form1_id = RegistrationFormId::from_entity(registration_form1.id);
        let mut expected = HashSet::new();
        expected.insert(RegistrationFormAnswerId::from_entity(
            registration_form1_answer1.id,
        ));
        expected.insert(RegistrationFormAnswerId::from_entity(
            registration_form1_answer2.id,
        ));

        let app = test::build_mock_app()
            .users(vec![login_user.clone(), operator])
            .registration_forms(vec![registration_form1, registration_form2])
            .projects(vec![project])
            .pending_projects(vec![pending_project])
            .registration_form_answers(vec![
                registration_form1_answer1,
                registration_form1_answer2,
                registration_form2_answer1,
                registration_form2_answer2,
            ])
            .build()
            .login_as(login_user.clone())
            .await;
        (app, registration_form1_id, expected)
    }

    // Checks that the normal user cannot list registration_form answers.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let (app, registration_form_id, _) = prepare_app(user).await;

        assert!(matches!(
            list_registration_form_answers::run(&app, registration_form_id).await,
            Err(UseCaseError::UseCase(
                list_registration_form_answers::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user can list registration_form answers.
    #[tokio::test]
    async fn test_committee() {
        let user = test::model::new_committee_user();
        let (app, registration_form_id, expected) = prepare_app(user).await;

        let result = list_registration_form_answers::run(&app, registration_form_id).await;
        assert!(result.is_ok());

        let got: HashSet<_> = result
            .unwrap()
            .into_iter()
            .map(|answer| answer.id)
            .collect();
        assert_eq!(got, expected);
    }

    // Checks that the privileged committee user can list registration_form answers.
    #[tokio::test]
    async fn test_operator() {
        let user = test::model::new_operator_user();
        let (app, registration_form_id, expected) = prepare_app(user).await;

        let result = list_registration_form_answers::run(&app, registration_form_id).await;
        assert!(result.is_ok());

        let got: HashSet<_> = result
            .unwrap()
            .into_iter()
            .map(|answer| answer.id)
            .collect();
        assert_eq!(got, expected);
    }
}
