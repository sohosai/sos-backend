use crate::error::{UseCaseError, UseCaseResult};
use crate::model::{
    form::{Form, FormId},
    project::ProjectId,
};

use anyhow::Context;
use sos21_domain::context::{FormRepository, Login, ProjectRepository};

#[derive(Debug, Clone)]
pub enum Error {
    ProjectNotFound,
    FormNotFound,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(
    ctx: &Login<C>,
    project_id: ProjectId,
    form_id: FormId,
) -> UseCaseResult<Form, Error>
where
    C: ProjectRepository + FormRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let result = ctx
        .get_project(project_id.into_entity())
        .await
        .context("Failed to get a project")?;
    let (project, _) = match result {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::ProjectNotFound)),
    };

    if !project.is_visible_to(login_user) {
        return Err(UseCaseError::UseCase(Error::ProjectNotFound));
    }

    let result = ctx
        .get_form(form_id.into_entity())
        .await
        .context("Failed to get a form")?;
    let form = match result {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::FormNotFound)),
    };

    if !form.condition.check(&project) || !form.is_visible_to_with_project(login_user, &project) {
        return Err(UseCaseError::UseCase(Error::FormNotFound));
    }

    Ok(Form::from_entity(form))
}

#[cfg(test)]
mod tests {
    use crate::model::{form::FormId, project::ProjectId};
    use crate::{get_project_form, UseCaseError};
    use sos21_domain::model::{project, project_query};
    use sos21_domain_test as test;

    // Checks that the normal user cannot read the form via others' project.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id.clone());
        let form = test::model::new_form(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_form::run(
                &app,
                ProjectId::from_entity(project_other.id),
                FormId::from_entity(form.id)
            )
            .await,
            Err(UseCaseError::UseCase(
                get_project_form::Error::ProjectNotFound
            ))
        ));
    }

    // Checks that the normal user can read the form via owning project.
    #[tokio::test]
    async fn test_general_owner() {
        let user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id.clone());
        let form = test::model::new_form(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let form_id = FormId::from_entity(form.id);
        assert!(matches!(
            get_project_form::run(&app, ProjectId::from_entity(project.id), form_id).await,
            Ok(got)
            if got.id == form_id && got.name == form.name.into_string()
        ));
    }

    // Checks that the privileged committee user cannot read the unmatching form via owning project.
    #[tokio::test]
    async fn test_operator_owner_unmatched() {
        let user = test::model::new_operator_user();
        let project = test::model::new_project_with_attributes(
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
        let form = test::model::new_form_with_query(user.id.clone(), query);

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_form::run(
                &app,
                ProjectId::from_entity(project.id),
                FormId::from_entity(form.id)
            )
            .await,
            Err(UseCaseError::UseCase(get_project_form::Error::FormNotFound))
        ));
    }

    // Checks that the (unprivileged) committee user can read the form via others' project.
    #[tokio::test]
    async fn test_committee_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id.clone());
        let form = test::model::new_form(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let form_id = FormId::from_entity(form.id);
        assert!(matches!(
            get_project_form::run(&app, ProjectId::from_entity(project_other.id), form_id).await,
            Ok(got)
            if got.id == form_id && got.name == form.name.into_string()
        ));
    }

    // Checks that the privileged committee user can read the form via others' project.
    #[tokio::test]
    async fn test_operator_other() {
        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id.clone());
        let form = test::model::new_form(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .forms(vec![form.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let form_id = FormId::from_entity(form.id);
        assert!(matches!(
            get_project_form::run(&app, ProjectId::from_entity(project_other.id), form_id).await,
            Ok(got)
            if got.id == form_id && got.name == form.name.into_string()
        ));
    }
}
