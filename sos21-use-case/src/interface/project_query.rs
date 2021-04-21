use crate::model::project::{ProjectAttribute, ProjectCategory};
use crate::model::project_query::{ProjectQuery, ProjectQueryConjunction};

use sos21_domain::model::{project, project_query};

#[derive(Debug, Clone)]
pub enum ProjectQueryError {
    TooBigQuery,
    DuplicatedAttributes,
}

impl ProjectQueryError {
    fn from_query_error(err: project_query::FromConjunctionsError) -> Self {
        match err.kind() {
            project_query::FromConjunctionsErrorKind::TooBigDisjunction => {
                ProjectQueryError::TooBigQuery
            }
        }
    }

    fn from_attributes_error(_err: project::attribute::DuplicatedAttributesError) -> Self {
        ProjectQueryError::DuplicatedAttributes
    }
}

pub fn to_project_query(
    query: ProjectQuery,
) -> Result<project_query::ProjectQuery, ProjectQueryError> {
    let dnf = query
        .0
        .into_iter()
        .map(to_project_query_conjunction)
        .collect::<Result<Vec<_>, _>>()?;
    project_query::ProjectQuery::from_conjunctions(dnf).map_err(ProjectQueryError::from_query_error)
}

pub fn to_project_query_conjunction(
    conj: ProjectQueryConjunction,
) -> Result<project_query::ProjectQueryConjunction, ProjectQueryError> {
    let category = conj.category.map(ProjectCategory::into_entity);
    let attributes = project::ProjectAttributes::from_attributes(
        conj.attributes
            .into_iter()
            .map(ProjectAttribute::into_entity),
    )
    .map_err(ProjectQueryError::from_attributes_error)?;

    Ok(project_query::ProjectQueryConjunction {
        category,
        attributes,
    })
}
