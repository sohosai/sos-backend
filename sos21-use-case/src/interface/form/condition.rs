use crate::interface::project_query::{to_project_query, ProjectQueryError};
use crate::model::form::FormCondition;
use crate::model::project::ProjectId;

use sos21_domain::model::form;

#[derive(Debug, Clone)]
pub enum FormConditionError {
    TooManyIncludeProjects,
    DuplicatedIncludeProjects,
    TooManyExcludeProjects,
    DuplicatedExcludeProjects,
    InvalidQuery(ProjectQueryError),
}

impl FormConditionError {
    fn from_query_error(err: ProjectQueryError) -> Self {
        FormConditionError::InvalidQuery(err)
    }

    fn from_includes_error(err: form::condition::FromProjectsError) -> Self {
        match err.kind() {
            form::condition::FromProjectsErrorKind::TooLong => {
                FormConditionError::TooManyIncludeProjects
            }
            form::condition::FromProjectsErrorKind::Duplicated(_) => {
                FormConditionError::DuplicatedIncludeProjects
            }
        }
    }

    fn from_excludes_error(err: form::condition::FromProjectsError) -> Self {
        match err.kind() {
            form::condition::FromProjectsErrorKind::TooLong => {
                FormConditionError::TooManyExcludeProjects
            }
            form::condition::FromProjectsErrorKind::Duplicated(_) => {
                FormConditionError::DuplicatedExcludeProjects
            }
        }
    }
}

pub fn to_form_condition(
    condition: FormCondition,
) -> Result<form::FormCondition, FormConditionError> {
    let query = to_project_query(condition.query).map_err(FormConditionError::from_query_error)?;
    let includes = form::FormConditionProjectSet::from_projects(
        condition.includes.into_iter().map(ProjectId::into_entity),
    )
    .map_err(FormConditionError::from_includes_error)?;
    let excludes = form::FormConditionProjectSet::from_projects(
        condition.excludes.into_iter().map(ProjectId::into_entity),
    )
    .map_err(FormConditionError::from_excludes_error)?;

    Ok(form::FormCondition {
        query,
        includes,
        excludes,
    })
}
