use crate::model::project::{ProjectAttribute, ProjectCategory};

use sos21_domain::model::project_query as entity;

#[derive(Debug, Clone)]
pub struct ProjectQueryConjunction {
    pub category: Option<ProjectCategory>,
    pub attributes: Vec<ProjectAttribute>,
}

impl ProjectQueryConjunction {
    pub fn from_entity(conj: entity::ProjectQueryConjunction) -> Self {
        ProjectQueryConjunction {
            category: conj.category().map(ProjectCategory::from_entity),
            attributes: conj
                .attributes()
                .map(ProjectAttribute::from_entity)
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectQuery(pub Vec<ProjectQueryConjunction>);

impl ProjectQuery {
    pub fn from_entity(query: entity::ProjectQuery) -> Self {
        ProjectQuery(
            query
                .into_conjunctions()
                .map(ProjectQueryConjunction::from_entity)
                .collect(),
        )
    }
}
