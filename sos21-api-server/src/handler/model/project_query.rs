use crate::handler::model::project::{ProjectAttribute, ProjectCategory};

use serde::{Deserialize, Serialize};
use sos21_use_case::model::project_query as use_case;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectQueryConjunction {
    pub category: Option<ProjectCategory>,
    pub attributes: Vec<ProjectAttribute>,
}

impl ProjectQueryConjunction {
    pub fn from_use_case(conj: use_case::ProjectQueryConjunction) -> Self {
        let category = conj.category.map(ProjectCategory::from_use_case);
        let attributes = conj
            .attributes
            .into_iter()
            .map(ProjectAttribute::from_use_case)
            .collect();
        ProjectQueryConjunction {
            category,
            attributes,
        }
    }

    pub fn into_use_case(self) -> use_case::ProjectQueryConjunction {
        let category = self.category.map(ProjectCategory::into_use_case);
        let attributes = self
            .attributes
            .into_iter()
            .map(ProjectAttribute::into_use_case)
            .collect();
        use_case::ProjectQueryConjunction {
            category,
            attributes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectQuery(pub Vec<ProjectQueryConjunction>);

impl ProjectQuery {
    pub fn from_use_case(query: use_case::ProjectQuery) -> Self {
        let query = query
            .0
            .into_iter()
            .map(ProjectQueryConjunction::from_use_case)
            .collect();
        ProjectQuery(query)
    }

    pub fn into_use_case(self) -> use_case::ProjectQuery {
        let query = self
            .0
            .into_iter()
            .map(ProjectQueryConjunction::into_use_case)
            .collect();
        use_case::ProjectQuery(query)
    }
}
