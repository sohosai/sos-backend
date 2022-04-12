use crate::error::{UseCaseError, UseCaseResult};
use crate::model::{
    project::ProjectId, registration_form::RegistrationFormId,
    registration_form_answer::RegistrationFormAnswer,
};

use anyhow::Context;
use sos21_domain::context::{
    Login, ProjectRepository, RegistrationFormAnswerRepository, RegistrationFormRepository,
};

#[derive(Debug, Clone)]
pub enum Error {
    ProjectNotFound,
    RegistrationFormNotFound,
    RegistrationFormAnswerNotFound,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(
    ctx: &Login<C>,
    project_id: ProjectId,
    registration_form_id: RegistrationFormId,
) -> UseCaseResult<RegistrationFormAnswer, Error>
where
    C: ProjectRepository
        + RegistrationFormRepository
        + RegistrationFormAnswerRepository
        + Send
        + Sync,
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
        .get_registration_form_answer_by_registration_form_and_project(
            registration_form_id.into_entity(),
            project_id.into_entity(),
        )
        .await
        .context("Failed to get a registration form answer")?;
    let answer = match result {
        Some(answer) => answer,
        None => {
            if let Some(registration_form) = ctx
                .get_registration_form(registration_form_id.into_entity())
                .await?
            {
                if registration_form.is_visible_to_with_project(login_user, &project) {
                    return Err(UseCaseError::UseCase(Error::RegistrationFormAnswerNotFound));
                }
            }
            return Err(UseCaseError::UseCase(Error::RegistrationFormNotFound));
        }
    };

    if !answer.is_visible_to_with_project(login_user, &project) {
        return Err(UseCaseError::UseCase(Error::RegistrationFormAnswerNotFound));
    }

    Ok(RegistrationFormAnswer::from_entity(answer))
}

#[cfg(test)]
mod tests {
    use crate::model::{
        project::ProjectId, registration_form::RegistrationFormId,
        registration_form_answer::RegistrationFormAnswerId,
    };
    use crate::{get_project_registration_form_answer, UseCaseError};
    use sos21_domain::test;

    // Checks that the normal user cannot read registration form answers of others' projects.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_online_project(other.id().clone());
        let registration_form = test::model::new_registration_form(other.id().clone());
        let answer_other = test::model::new_registration_form_answer_with_project(
            other.id().clone(),
            project_other.id(),
            &registration_form,
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_registration_form_answer::run(
                &app,
                ProjectId::from_entity(project_other.id()),
                RegistrationFormId::from_entity(registration_form.id),
            )
            .await,
            Err(UseCaseError::UseCase(
                get_project_registration_form_answer::Error::ProjectNotFound
            ))
        ));
    }

    // Checks that the normal user can read registration form answers of owning projects.
    #[tokio::test]
    async fn test_general_owner() {
        let user = test::model::new_general_user();
        let subowner = test::model::new_general_user();
        let operator = test::model::new_operator_user();
        let project = test::model::new_general_online_project_with_subowner(
            user.id().clone(),
            subowner.id().clone(),
        );
        let registration_form = test::model::new_registration_form(operator.id().clone());
        let answer_other = test::model::new_registration_form_answer_with_project(
            subowner.id().clone(),
            project.id(),
            &registration_form,
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), operator.clone(), subowner.clone()])
            .projects(vec![project.clone()])
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_registration_form_answer::run(
                &app,
                ProjectId::from_entity(project.id()),
                RegistrationFormId::from_entity(registration_form.id),
            )
            .await,
            Ok(got)
            if got.id == RegistrationFormAnswerId::from_entity(answer_other.id())
        ));
    }

    // Checks that the normal user can read registration form answers of owning projects.
    #[tokio::test]
    async fn test_general_subowner() {
        let owner = test::model::new_general_user();
        let user = test::model::new_general_user();
        let operator = test::model::new_operator_user();
        let project = test::model::new_general_online_project_with_subowner(
            owner.id().clone(),
            user.id().clone(),
        );
        let registration_form = test::model::new_registration_form(operator.id().clone());
        let answer_other = test::model::new_registration_form_answer_with_project(
            owner.id().clone(),
            project.id(),
            &registration_form,
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), operator.clone(), owner.clone()])
            .projects(vec![project.clone()])
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_registration_form_answer::run(
                &app,
                ProjectId::from_entity(project.id()),
                RegistrationFormId::from_entity(registration_form.id),
            )
            .await,
            Ok(got)
            if got.id == RegistrationFormAnswerId::from_entity(answer_other.id())
        ));
    }

    // Checks that the (unprivileged) committee user can read registration form answers of the others' projects.
    #[tokio::test]
    async fn test_committee_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_online_project(other.id().clone());
        let registration_form = test::model::new_registration_form(other.id().clone());
        let answer_other = test::model::new_registration_form_answer_with_project(
            other.id().clone(),
            project_other.id(),
            &registration_form,
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_registration_form_answer::run(
                &app,
                ProjectId::from_entity(project_other.id()),
                RegistrationFormId::from_entity(registration_form.id)
            )
            .await,
            Ok(got)
            if got.id == RegistrationFormAnswerId::from_entity(answer_other.id())
        ));
    }

    // Checks that the privileged committee user can read registration_form answers of the others' projects.
    #[tokio::test]
    async fn test_operator_other() {
        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_online_project(other.id().clone());
        let registration_form = test::model::new_registration_form(other.id().clone());
        let answer_other = test::model::new_registration_form_answer_with_project(
            other.id().clone(),
            project_other.id(),
            &registration_form,
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_registration_form_answer::run(
                &app,
                ProjectId::from_entity(project_other.id()),
                RegistrationFormId::from_entity(registration_form.id)
            )
            .await,
            Ok(got)
            if got.id == RegistrationFormAnswerId::from_entity(answer_other.id())
        ));
    }
}
