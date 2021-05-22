use crate::error::{UseCaseError, UseCaseResult};
use crate::model::{form::FormId, form_answer::FormAnswer, project::ProjectId};

use anyhow::Context;
use sos21_domain::context::{FormAnswerRepository, FormRepository, Login, ProjectRepository};

#[derive(Debug, Clone)]
pub enum Error {
    ProjectNotFound,
    FormNotFound,
    FormAnswerNotFound,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(
    ctx: &Login<C>,
    project_id: ProjectId,
    form_id: FormId,
) -> UseCaseResult<FormAnswer, Error>
where
    C: ProjectRepository + FormRepository + FormAnswerRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let result = ctx
        .get_project(project_id.into_entity())
        .await
        .context("Failed to get a project")?;
    let project = match result {
        Some(result) if result.project.is_visible_to(login_user) => result.project,
        _ => return Err(UseCaseError::UseCase(Error::ProjectNotFound)),
    };

    let result = ctx
        .get_form_answer_by_form_and_project(form_id.into_entity(), project_id.into_entity())
        .await
        .context("Failed to get a form answer")?;
    let answer = match result {
        Some(answer) => answer,
        None => {
            if let Some(form) = ctx.get_form(form_id.into_entity()).await? {
                if form.is_visible_to(login_user) {
                    return Err(UseCaseError::UseCase(Error::FormAnswerNotFound));
                }
            }
            return Err(UseCaseError::UseCase(Error::FormNotFound));
        }
    };

    if !answer.is_visible_to_with_project(login_user, &project) {
        return Err(UseCaseError::UseCase(Error::FormAnswerNotFound));
    }

    Ok(FormAnswer::from_entity(answer))
}

#[cfg(test)]
mod tests {
    use crate::model::{form::FormId, form_answer::FormAnswerId, project::ProjectId};
    use crate::{get_project_form_answer, UseCaseError};
    use sos21_domain::test;

    // Checks that the normal user cannot read form answers of invisible projects.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id().clone());
        assert!(!project_other.is_visible_to(&user));
        let form = test::model::new_form(other.id().clone());
        let answer_other = test::model::new_form_answer(other.id().clone(), &project_other, &form);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .forms(vec![form.clone()])
            .answers(vec![answer_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_form_answer::run(
                &app,
                ProjectId::from_entity(project_other.id()),
                FormId::from_entity(form.id()),
            )
            .await,
            Err(UseCaseError::UseCase(
                get_project_form_answer::Error::ProjectNotFound
            ))
        ));
    }

    // Checks that the (unprivileged) committee user can read form answers of the others' projects.
    #[tokio::test]
    async fn test_committee_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id().clone());
        let form = test::model::new_form(other.id().clone());
        let answer_other = test::model::new_form_answer(other.id().clone(), &project_other, &form);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .forms(vec![form.clone()])
            .answers(vec![answer_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_form_answer::run(&app, ProjectId::from_entity(project_other.id()), FormId::from_entity(form.id())).await,
            Ok(got)
            if got.id == FormAnswerId::from_entity(answer_other.id())
        ));
    }

    // Checks that the privileged committee user can read form answers of the others' projects.
    #[tokio::test]
    async fn test_operator_other() {
        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id().clone());
        let form = test::model::new_form(other.id().clone());
        let answer_other = test::model::new_form_answer(other.id().clone(), &project_other, &form);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .forms(vec![form.clone()])
            .answers(vec![answer_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_form_answer::run(&app, ProjectId::from_entity(project_other.id()), FormId::from_entity(form.id())).await,
            Ok(got)
            if got.id == FormAnswerId::from_entity(answer_other.id())
        ));
    }
}
