use crate::error::{UseCaseError, UseCaseResult};
use crate::model::{
    pending_project::PendingProjectId,
    registration_form::{RegistrationForm, RegistrationFormId},
};

use anyhow::Context;
use sos21_domain::context::{Login, PendingProjectRepository, RegistrationFormRepository};

#[derive(Debug, Clone)]
pub enum Error {
    PendingProjectNotFound,
    RegistrationFormNotFound,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(
    ctx: &Login<C>,
    pending_project_id: PendingProjectId,
    registration_form_id: RegistrationFormId,
) -> UseCaseResult<RegistrationForm, Error>
where
    C: PendingProjectRepository + RegistrationFormRepository + Send + Sync,
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
        .get_registration_form(registration_form_id.into_entity())
        .await
        .context("Failed to get a registration form")?;
    let registration_form = match result {
        Some(registration_form)
            if registration_form
                .is_visible_to_with_pending_project(login_user, &pending_project) =>
        {
            registration_form
        }
        _ => return Err(UseCaseError::UseCase(Error::RegistrationFormNotFound)),
    };

    if !registration_form
        .query
        .check_pending_project(&pending_project)
    {
        return Err(UseCaseError::UseCase(Error::RegistrationFormNotFound));
    }

    Ok(RegistrationForm::from_entity(registration_form))
}

#[cfg(test)]
mod tests {
    use crate::model::{pending_project::PendingProjectId, registration_form::RegistrationFormId};
    use crate::{get_pending_project_registration_form, UseCaseError};
    use sos21_domain::model::{project, project_query};
    use sos21_domain::test;

    // Checks that the normal user cannot read the registration_form via others' pending_project.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let pending_project_other = test::model::new_general_pending_project(other.id.clone());
        let registration_form = test::model::new_registration_form(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![pending_project_other.clone()])
            .registration_forms(vec![registration_form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_pending_project_registration_form::run(
                &app,
                PendingProjectId::from_entity(pending_project_other.id()),
                RegistrationFormId::from_entity(registration_form.id)
            )
            .await,
            Err(UseCaseError::UseCase(
                get_pending_project_registration_form::Error::RegistrationFormNotFound
            ))
        ));
    }

    // Checks that the normal user can read the registration_form via owning pending_project.
    #[tokio::test]
    async fn test_general_author() {
        let user = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(user.id.clone());
        let registration_form = test::model::new_registration_form(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![registration_form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let registration_form_id = RegistrationFormId::from_entity(registration_form.id);
        assert!(matches!(
            get_pending_project_registration_form::run(
                &app,
                PendingProjectId::from_entity(pending_project.id()),
                registration_form_id
            )
            .await,
            Ok(got)
            if got.id == registration_form_id && got.name == registration_form.name.into_string()
        ));
    }

    // Checks that the privileged committee user cannot read the unmatching registration_form via owning pending_project.
    #[tokio::test]
    async fn test_operator_author_unmatched() {
        let user = test::model::new_operator_user();
        let pending_project = test::model::new_pending_project_with_attributes(
            user.id.clone(),
            project::ProjectCategory::General,
            &[
                project::ProjectAttribute::Academic,
                project::ProjectAttribute::Artistic,
            ],
        );
        let query = project_query::ProjectQuery::from_conjunctions(vec![
            project_query::ProjectQueryConjunction {
                category: None,
                attributes: project::ProjectAttributes::from_attributes(vec![
                    project::ProjectAttribute::Committee,
                ])
                .unwrap(),
            },
        ])
        .unwrap();
        let registration_form =
            test::model::new_registration_form_with_query(user.id.clone(), query);

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![registration_form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_pending_project_registration_form::run(
                &app,
                PendingProjectId::from_entity(pending_project.id()),
                RegistrationFormId::from_entity(registration_form.id)
            )
            .await,
            Err(UseCaseError::UseCase(
                get_pending_project_registration_form::Error::RegistrationFormNotFound
            ))
        ));
    }

    // Checks that the (unprivileged) committee user can read the registration_form via others' pending_project.
    #[tokio::test]
    async fn test_committee_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let pending_project_other = test::model::new_general_pending_project(other.id.clone());
        let registration_form = test::model::new_registration_form(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![pending_project_other.clone()])
            .registration_forms(vec![registration_form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let registration_form_id = RegistrationFormId::from_entity(registration_form.id);
        assert!(matches!(
            get_pending_project_registration_form::run(
                &app,
                PendingProjectId::from_entity(pending_project_other.id()),
                registration_form_id,
            )
            .await,
            Ok(got)
            if got.id == registration_form_id && got.name == registration_form.name.into_string()
        ));
    }

    // Checks that the privileged committee user can read the registration_form via others' pending_project.
    #[tokio::test]
    async fn test_operator_other() {
        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let pending_project_other = test::model::new_general_pending_project(other.id.clone());
        let registration_form = test::model::new_registration_form(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![pending_project_other.clone()])
            .registration_forms(vec![registration_form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let registration_form_id = RegistrationFormId::from_entity(registration_form.id);
        assert!(matches!(
            get_pending_project_registration_form::run(
                &app,
                PendingProjectId::from_entity(pending_project_other.id()),
                registration_form_id,
            )
            .await,
            Ok(got)
            if got.id == registration_form_id && got.name == registration_form.name.into_string()
        ));
    }
}
