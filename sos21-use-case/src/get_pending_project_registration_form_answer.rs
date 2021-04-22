use crate::error::{UseCaseError, UseCaseResult};
use crate::model::{
    pending_project::PendingProjectId, registration_form::RegistrationFormId,
    registration_form_answer::RegistrationFormAnswer,
};

use anyhow::Context;
use sos21_domain::context::{
    Login, PendingProjectRepository, RegistrationFormAnswerRepository, RegistrationFormRepository,
};

#[derive(Debug, Clone)]
pub enum Error {
    PendingProjectNotFound,
    RegistrationFormNotFound,
    RegistrationFormAnswerNotFound,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(
    ctx: &Login<C>,
    pending_project_id: PendingProjectId,
    registration_form_id: RegistrationFormId,
) -> UseCaseResult<RegistrationFormAnswer, Error>
where
    C: PendingProjectRepository
        + RegistrationFormRepository
        + RegistrationFormAnswerRepository
        + Send
        + Sync,
{
    let login_user = ctx.login_user();

    let result = ctx
        .get_pending_project(pending_project_id.into_entity())
        .await
        .context("Failed to get a pending project")?;
    let pending_project = match result {
        Some(result) if result.pending_project.is_visible_to(login_user) => result.pending_project,
        _ => return Err(UseCaseError::UseCase(Error::PendingProjectNotFound)),
    };

    let result = ctx
        .get_registration_form_answer_by_registration_form_and_pending_project(
            registration_form_id.into_entity(),
            pending_project_id.into_entity(),
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
                if registration_form
                    .is_visible_to_with_pending_project(login_user, &pending_project)
                {
                    return Err(UseCaseError::UseCase(Error::RegistrationFormAnswerNotFound));
                }
            }
            return Err(UseCaseError::UseCase(Error::RegistrationFormNotFound));
        }
    };

    if !answer.is_visible_to_with_pending_project(login_user, &pending_project) {
        return Err(UseCaseError::UseCase(Error::RegistrationFormAnswerNotFound));
    }

    Ok(RegistrationFormAnswer::from_entity(answer))
}

#[cfg(test)]
mod tests {
    use crate::model::{
        pending_project::PendingProjectId, registration_form::RegistrationFormId,
        registration_form_answer::RegistrationFormAnswerId,
    };
    use crate::{get_pending_project_registration_form_answer, UseCaseError};
    use sos21_domain::test;

    // Checks that the normal user cannot read registration_form answers of others' pending_projects.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let pending_project_other = test::model::new_general_pending_project(other.id.clone());
        let registration_form = test::model::new_registration_form(other.id.clone());
        let answer_other = test::model::new_registration_form_answer_with_pending_project(
            other.id.clone(),
            pending_project_other.id,
            &registration_form,
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![pending_project_other.clone()])
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_pending_project_registration_form_answer::run(
                &app,
                PendingProjectId::from_entity(pending_project_other.id),
                RegistrationFormId::from_entity(registration_form.id),
            )
            .await,
            Err(UseCaseError::UseCase(
                get_pending_project_registration_form_answer::Error::RegistrationFormAnswerNotFound
            ))
        ));
    }

    // Checks that the (unprivileged) committee user can read registration_form answers of the others' pending_projects.
    #[tokio::test]
    async fn test_committee_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let pending_project_other = test::model::new_general_pending_project(other.id.clone());
        let registration_form = test::model::new_registration_form(other.id.clone());
        let answer_other = test::model::new_registration_form_answer_with_pending_project(
            other.id.clone(),
            pending_project_other.id,
            &registration_form,
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![pending_project_other.clone()])
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_pending_project_registration_form_answer::run(
                &app,
                PendingProjectId::from_entity(pending_project_other.id),
                RegistrationFormId::from_entity(registration_form.id)
            )
            .await,
            Ok(got)
            if got.id == RegistrationFormAnswerId::from_entity(answer_other.id)
        ));
    }

    // Checks that the privileged committee user can read registration_form answers of the others' pending_projects.
    #[tokio::test]
    async fn test_operator_other() {
        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let pending_project_other = test::model::new_general_pending_project(other.id.clone());
        let registration_form = test::model::new_registration_form(other.id.clone());
        let answer_other = test::model::new_registration_form_answer_with_pending_project(
            other.id.clone(),
            pending_project_other.id,
            &registration_form,
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![pending_project_other.clone()])
            .registration_forms(vec![registration_form.clone()])
            .registration_form_answers(vec![answer_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_pending_project_registration_form_answer::run(
                &app,
                PendingProjectId::from_entity(pending_project_other.id),
                RegistrationFormId::from_entity(registration_form.id)
            )
            .await,
            Ok(got)
            if got.id == RegistrationFormAnswerId::from_entity(answer_other.id)
        ));
    }
}
