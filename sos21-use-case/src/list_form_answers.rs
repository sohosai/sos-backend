use crate::error::{UseCaseError, UseCaseResult};
use crate::model::{form::FormId, form_answer::FormAnswer};

use anyhow::Context;
use sos21_domain::context::{FormAnswerRepository, FormRepository, Login};
use sos21_domain::model::permissions::Permissions;

#[derive(Debug, Clone)]
pub enum Error {
    FormNotFound,
    InsufficientPermissions,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, form_id: FormId) -> UseCaseResult<Vec<FormAnswer>, Error>
where
    C: FormRepository + FormAnswerRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    if login_user
        .require_permissions(Permissions::READ_ALL_FORM_ANSWERS)
        .is_err()
    {
        return Err(UseCaseError::UseCase(Error::InsufficientPermissions));
    }

    let form_id = form_id.into_entity();
    let answers = ctx
        .list_form_answers(form_id)
        .await
        .context("Failed to list form answers")?;
    if answers.is_empty() {
        match ctx.get_form(form_id).await? {
            Some(form) if form.is_visible_to(login_user) => {}
            _ => return Err(UseCaseError::UseCase(Error::FormNotFound)),
        }
    }

    use_case_ensure!(answers
        .iter()
        .all(|answer| answer.is_visible_to(login_user)));
    Ok(answers.into_iter().map(FormAnswer::from_entity).collect())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::model::{form::FormId, form_answer::FormAnswerId};
    use crate::{list_form_answers, UseCaseError};
    use sos21_domain::context::Login;
    use sos21_domain::model as domain;
    use sos21_domain::test;

    async fn prepare_app(
        login_user: domain::user::User,
    ) -> (Login<test::context::MockApp>, FormId, HashSet<FormAnswerId>) {
        let operator = test::model::new_operator_user();

        let project1 = test::model::new_general_project(login_user.id().clone());
        let project2 = test::model::new_general_project(login_user.id().clone());

        let form1 = test::model::new_form(operator.id().clone());
        let form1_answer1 =
            test::model::new_form_answer(login_user.id().clone(), &project1, &form1);
        let form1_answer2 =
            test::model::new_form_answer(login_user.id().clone(), &project2, &form1);

        let form2 = test::model::new_form(operator.id().clone());
        let form2_answer1 =
            test::model::new_form_answer(login_user.id().clone(), &project1, &form2);
        let form2_answer2 =
            test::model::new_form_answer(login_user.id().clone(), &project2, &form2);

        let form1_id = FormId::from_entity(form1.id());
        let mut expected = HashSet::new();
        expected.insert(FormAnswerId::from_entity(form1_answer1.id()));
        expected.insert(FormAnswerId::from_entity(form1_answer2.id()));

        let app = test::build_mock_app()
            .users(vec![login_user.clone(), operator])
            .forms(vec![form1, form2])
            .projects(vec![project1, project2])
            .answers(vec![
                form1_answer1,
                form1_answer2,
                form2_answer1,
                form2_answer2,
            ])
            .build()
            .login_as(login_user.clone())
            .await;
        (app, form1_id, expected)
    }

    // Checks that the normal user cannot list form answers.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let (app, form_id, _) = prepare_app(user).await;

        assert!(matches!(
            list_form_answers::run(&app, form_id).await,
            Err(UseCaseError::UseCase(
                list_form_answers::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user can list form answers.
    #[tokio::test]
    async fn test_committee() {
        let user = test::model::new_committee_user();
        let (app, form_id, expected) = prepare_app(user).await;

        let result = list_form_answers::run(&app, form_id).await;
        assert!(result.is_ok());

        let got: HashSet<_> = result
            .unwrap()
            .into_iter()
            .map(|answer| answer.id)
            .collect();
        assert_eq!(got, expected);
    }

    // Checks that the privileged committee user can list form answers.
    #[tokio::test]
    async fn test_operator() {
        let user = test::model::new_operator_user();
        let (app, form_id, expected) = prepare_app(user).await;

        let result = list_form_answers::run(&app, form_id).await;
        assert!(result.is_ok());

        let got: HashSet<_> = result
            .unwrap()
            .into_iter()
            .map(|answer| answer.id)
            .collect();
        assert_eq!(got, expected);
    }
}
