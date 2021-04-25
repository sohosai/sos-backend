use crate::model::{
    project::ProjectAttributes,
    project_query::{ProjectQuery, ProjectQueryConjunction},
};

pub fn mock_project_query() -> ProjectQuery {
    ProjectQuery::from_conjunctions(vec![ProjectQueryConjunction {
        category: None,
        attributes: ProjectAttributes::from_attributes(Vec::new()).unwrap(),
    }])
    .unwrap()
}
