use std::convert::TryInto;

use crate::error::{UseCaseError, UseCaseResult};
use crate::model::project::{Project, ProjectAttribute, ProjectCategory};

use anyhow::Context;
use sos21_domain::context::{Login, ProjectRepository};
use sos21_domain::model::{date_time::DateTime, project};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub name: String,
    pub kana_name: String,
    pub group_name: String,
    pub kana_group_name: String,
    pub description: String,
    pub category: ProjectCategory,
    pub attributes: Vec<ProjectAttribute>,
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidName,
    InvalidKanaName,
    InvalidGroupName,
    InvalidKanaGroupName,
    InvalidDescription,
    DuplicatedAttributes,
    TooManyProjects,
}

impl Error {
    fn from_count_integer_error(_err: std::num::TryFromIntError) -> Self {
        Error::TooManyProjects
    }

    fn from_index_error(_err: project::index::FromU16Error) -> Self {
        Error::TooManyProjects
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<Project, Error>
where
    C: ProjectRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let count = ctx
        .count_projects()
        .await
        .context("Failed to count projects")?;
    let count = count
        .try_into()
        .map_err(|err| UseCaseError::UseCase(Error::from_count_integer_error(err)))?;
    let index = project::ProjectIndex::from_u16(count)
        .map_err(|err| UseCaseError::UseCase(Error::from_index_error(err)))?;

    let name = project::ProjectName::from_string(input.name)
        .map_err(|_| UseCaseError::UseCase(Error::InvalidName))?;
    let kana_name = project::ProjectKanaName::from_string(input.kana_name)
        .map_err(|_| UseCaseError::UseCase(Error::InvalidKanaName))?;
    let group_name = project::ProjectGroupName::from_string(input.group_name)
        .map_err(|_| UseCaseError::UseCase(Error::InvalidGroupName))?;
    let kana_group_name = project::ProjectKanaGroupName::from_string(input.kana_group_name)
        .map_err(|_| UseCaseError::UseCase(Error::InvalidKanaGroupName))?;
    let description = project::ProjectDescription::from_string(input.description)
        .map_err(|_| UseCaseError::UseCase(Error::InvalidDescription))?;

    let attributes = project::ProjectAttributes::from_attributes(
        input
            .attributes
            .into_iter()
            .map(ProjectAttribute::into_entity),
    )
    .map_err(|_: project::attribute::DuplicatedAttributesError| {
        UseCaseError::UseCase(Error::DuplicatedAttributes)
    })?;

    let project = project::Project {
        id: project::ProjectId::from_uuid(Uuid::new_v4()),
        index,
        created_at: DateTime::now(),
        owner_id: login_user.id.clone(),
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        category: input.category.into_entity(),
        attributes,
    };
    ctx.store_project(project.clone())
        .await
        .context("Failed to create a project")?;
    use_case_ensure!(project.is_visible_to(login_user));
    Ok(Project::from_entity(
        project,
        login_user.name.clone(),
        login_user.kana_name.clone(),
    ))
}

#[cfg(test)]
mod tests {
    use crate::model::{project::ProjectCategory, user::UserId};
    use crate::{create_project, get_project};
    use sos21_domain::test;

    #[tokio::test]
    async fn test_create() {
        let user = test::model::new_general_user();
        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let name = "テストテスト".to_string();
        let input = create_project::Input {
            name: name.clone(),
            kana_name: test::model::mock_project_kana_name().into_string(),
            group_name: test::model::mock_project_group_name().into_string(),
            kana_group_name: test::model::mock_project_kana_group_name().into_string(),
            description: test::model::mock_project_description().into_string(),
            category: ProjectCategory::General,
            attributes: Vec::new(),
        };

        let result = create_project::run(&app, input).await;
        assert!(result.is_ok());

        let got = result.unwrap();
        assert!(got.name == name);
        assert!(got.owner_id == UserId::from_entity(user.id));

        assert!(matches!(get_project::run(&app, got.id).await, Ok(_)));
    }
}
