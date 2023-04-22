use crate::error::{UseCaseError, UseCaseResult};
use crate::model::{project::ProjectId, registration_form::RegistrationForm};

use anyhow::Context;
use sos21_domain::context::{Login, ProjectRepository, RegistrationFormRepository};

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(
    ctx: &Login<C>,
    project_id: ProjectId,
) -> UseCaseResult<Vec<RegistrationForm>, Error>
where
    C: ProjectRepository + RegistrationFormRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let result = ctx
        .get_project(project_id.into_entity())
        .await
        .context("Failed to get a project")?;
    let project = match result {
        Some(result) if result.project.is_visible_to(login_user) => result.project,
        _ => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    ctx.list_registration_forms_by_project(project.id())
        .await
        .context("Failed to list registration forms")?
        .into_iter()
        .map(|registration_form| {
            use_case_ensure!(registration_form.is_visible_to_with_project(login_user, &project));
            Ok(RegistrationForm::from_entity(registration_form))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::list_project_registration_forms;
    use crate::model::{project::ProjectId, registration_form::RegistrationFormId};
    use sos21_domain::model::{project, project_query};
    use sos21_domain::test;

    #[tokio::test]
    async fn test_general_any() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let project = test::model::new_general_project(user.id().clone());
        let registration_form1 = test::model::new_registration_form(user.id().clone());
        let registration_form2 = test::model::new_registration_form(user.id().clone());
        let registration_form3 = test::model::new_registration_form(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .registration_forms(vec![
                registration_form1.clone(),
                registration_form2.clone(),
                registration_form3.clone(),
            ])
            .projects(vec![project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let result =
            list_project_registration_forms::run(&app, ProjectId::from_entity(project.id())).await;
        assert!(result.is_ok());

        let got: HashSet<_> = result
            .unwrap()
            .into_iter()
            .map(|registration_form| registration_form.id)
            .collect();
        let expected: HashSet<_> = vec![registration_form1, registration_form2, registration_form3]
            .into_iter()
            .map(|registration_form| RegistrationFormId::from_entity(registration_form.id))
            .collect();
        assert_eq!(got, expected);
    }

    #[tokio::test]
    async fn test_general_query() {
        let user = test::model::new_general_user();
        let project = test::model::new_project_with_attributes(
            user.id().clone(),
            project::ProjectCategory::General,
            &[
                project::ProjectAttribute::Academic,
                project::ProjectAttribute::Artistic,
            ],
        );

        let query1 = project_query::ProjectQuery::from_conjunctions(vec![
            project_query::ProjectQueryConjunction {
                category: None,
                attributes: project::ProjectAttributes::from_attributes(vec![
                    project::ProjectAttribute::Academic,
                ])
                .unwrap(),
            },
        ])
        .unwrap();
        let registration_form1 =
            test::model::new_registration_form_with_query(user.id().clone(), query1);

        let query2 = project_query::ProjectQuery::from_conjunctions(vec![
            project_query::ProjectQueryConjunction {
                category: None,
                attributes: project::ProjectAttributes::from_attributes(vec![
                    project::ProjectAttribute::Committee,
                ])
                .unwrap(),
            },
        ])
        .unwrap();
        let registration_form2 =
            test::model::new_registration_form_with_query(user.id().clone(), query2);
        let registration_form3 = test::model::new_registration_form(user.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .registration_forms(vec![
                registration_form1.clone(),
                registration_form2.clone(),
                registration_form3.clone(),
            ])
            .projects(vec![project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let result =
            list_project_registration_forms::run(&app, ProjectId::from_entity(project.id())).await;
        assert!(result.is_ok());

        let got: HashSet<_> = result
            .unwrap()
            .into_iter()
            .map(|registration_form| registration_form.id)
            .collect();
        let expected: HashSet<_> = vec![registration_form1, registration_form3]
            .into_iter()
            .map(|registration_form| RegistrationFormId::from_entity(registration_form.id))
            .collect();
        assert_eq!(got, expected);
    }
}
