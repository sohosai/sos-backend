use crate::error::{UseCaseError, UseCaseResult};
use crate::model::form_answer::{FormAnswer, FormAnswerId};

use anyhow::Context;
use sos21_domain::context::{FormAnswerRepository, Login};
use sos21_domain::model::permissions::Permissions;

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
    InsufficientPermissions,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, answer_id: FormAnswerId) -> UseCaseResult<FormAnswer, Error>
where
    C: FormAnswerRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    if login_user
        .require_permissions(Permissions::READ_ALL_FORM_ANSWERS)
        .is_err()
    {
        return Err(UseCaseError::UseCase(Error::InsufficientPermissions));
    }

    let result = ctx
        .get_form_answer(answer_id.into_entity())
        .await
        .context("Failed to get a form answer")?;
    let answer = match result {
        Some(answer) => answer,
        _ => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    use_case_ensure!(answer.is_visible_to(login_user));
    Ok(FormAnswer::from_entity(answer))
}

#[cfg(test)]
mod tests {
    use crate::model::form_answer::FormAnswerId;
    use crate::{get_form_answer, UseCaseError};
    use sos21_domain::test;

    // Checks that the normal user cannot read the form answers directly.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let operator = test::model::new_operator_user();
        let project = test::model::new_general_project(user.id().clone());
        let form = test::model::new_form(operator.id().clone());
        let answer = test::model::new_form_answer(user.id().clone(), project.id(), &form);

        let app = test::build_mock_app()
            .users(vec![user.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .answers(vec![answer.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_form_answer::run(&app, FormAnswerId::from_entity(answer.id())).await,
            Err(UseCaseError::UseCase(
                get_form_answer::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user can read the form answers directly.
    #[tokio::test]
    async fn test_committee_other() {
        let user = test::model::new_committee_user();
        let operator = test::model::new_operator_user();
        let project = test::model::new_general_project(user.id().clone());
        let form = test::model::new_form(operator.id().clone());
        let answer = test::model::new_form_answer(user.id().clone(), project.id(), &form);

        let app = test::build_mock_app()
            .users(vec![user.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .answers(vec![answer.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let answer_id = FormAnswerId::from_entity(answer.id());
        assert!(matches!(
            get_form_answer::run(&app, answer_id).await,
            Ok(got)
            if got.id == answer_id
        ));
    }

    // Checks that the privileged committee user can read form answers of the others' projects.
    #[tokio::test]
    async fn test_operator_other() {
        let user = test::model::new_operator_user();
        let operator = test::model::new_operator_user();
        let project = test::model::new_general_project(user.id().clone());
        let form = test::model::new_form(operator.id().clone());
        let answer = test::model::new_form_answer(user.id().clone(), project.id(), &form);

        let app = test::build_mock_app()
            .users(vec![user.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .answers(vec![answer.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let answer_id = FormAnswerId::from_entity(answer.id());
        assert!(matches!(
            get_form_answer::run(&app, answer_id).await,
            Ok(got)
            if got.id == answer_id
        ));
    }
}
