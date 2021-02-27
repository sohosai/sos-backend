use crate::model::bound::{Bounded, Unbounded};
use crate::model::collection::LengthLimitedVec;
use crate::model::project::{Project, ProjectAttribute, ProjectAttributes, ProjectCategory};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectQueryConjunction {
    pub category: Option<ProjectCategory>,
    pub attributes: ProjectAttributes,
}

impl ProjectQueryConjunction {
    pub fn category(&self) -> Option<ProjectCategory> {
        self.category
    }

    pub fn attributes(&self) -> impl Iterator<Item = ProjectAttribute> + '_ {
        self.attributes.attributes()
    }

    pub fn check(&self, project: &Project) -> bool {
        if let Some(category) = self.category {
            if category != project.category {
                return false;
            }
        }

        self.attributes.is_subset(&project.attributes)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectQuery(
    LengthLimitedVec<Unbounded, Bounded<typenum::U32>, ProjectQueryConjunction>,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromConjunctionsErrorKind {
    TooBigDisjunction,
}

#[derive(Debug, Error, Clone)]
#[error("invalid project query")]
pub struct FromConjunctionsError {
    kind: FromConjunctionsErrorKind,
}

impl FromConjunctionsError {
    pub fn kind(&self) -> FromConjunctionsErrorKind {
        self.kind
    }
}

impl ProjectQuery {
    pub fn from_conjunctions<I>(conj: I) -> Result<Self, FromConjunctionsError>
    where
        I: IntoIterator<Item = ProjectQueryConjunction>,
    {
        let dnf = LengthLimitedVec::new(conj.into_iter().collect()).map_err(|_| {
            FromConjunctionsError {
                kind: FromConjunctionsErrorKind::TooBigDisjunction,
            }
        })?;
        Ok(ProjectQuery(dnf))
    }

    pub fn conjunctions(&self) -> impl Iterator<Item = &'_ ProjectQueryConjunction> + '_ {
        self.0.iter()
    }

    pub fn into_conjunctions(self) -> impl Iterator<Item = ProjectQueryConjunction> {
        self.0.into_inner().into_iter()
    }

    pub fn check(&self, project: &Project) -> bool {
        self.conjunctions().any(|conj| conj.check(project))
    }
}

#[cfg(test)]
mod tests {
    use super::{ProjectQuery, ProjectQueryConjunction};
    use crate::{
        model::project::{ProjectAttribute, ProjectAttributes, ProjectCategory},
        test::model as test_model,
    };

    #[test]
    fn test_conj_tautology() {
        let conj = ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        };
        let mut project = test_model::new_general_project(test_model::new_user_id());
        assert!(conj.check(&project));
        project.set_attributes(
            ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        );
        assert!(conj.check(&project));
    }

    #[test]
    fn test_conj_attribute_single() {
        let conj = ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![ProjectAttribute::Artistic])
                .unwrap(),
        };
        let mut project = test_model::new_general_project(test_model::new_user_id());
        project.set_attributes(ProjectAttributes::from_attributes(vec![]).unwrap());
        assert!(!conj.check(&project));
        project.set_attributes(
            ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Artistic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        );
        assert!(conj.check(&project));
        project.set_attributes(
            ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        );
        assert!(conj.check(&project));
        project.set_attributes(
            ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        );
        assert!(!conj.check(&project));
    }

    #[test]
    fn test_conj_attribute_multiple() {
        let conj = ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Artistic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        };
        let mut project = test_model::new_general_project(test_model::new_user_id());
        project.set_attributes(ProjectAttributes::from_attributes(vec![]).unwrap());
        assert!(!conj.check(&project));
        project.set_attributes(
            ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Artistic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        );
        assert!(conj.check(&project));
        project.set_attributes(
            ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        );
        assert!(conj.check(&project));
        project.set_attributes(
            ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
            ])
            .unwrap(),
        );
        assert!(!conj.check(&project));
    }

    #[test]
    fn test_conj_category() {
        let conj = ProjectQueryConjunction {
            category: Some(ProjectCategory::General),
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        };
        assert!(conj.check(&test_model::new_general_project(test_model::new_user_id())));
        assert!(!conj.check(&test_model::new_stage_project(test_model::new_user_id())));
    }

    #[test]
    fn test_conj_complex() {
        let conj = ProjectQueryConjunction {
            category: Some(ProjectCategory::Stage),
            attributes: ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        };
        let mut project = test_model::new_general_project(test_model::new_user_id());
        assert!(!conj.check(&project));
        project.set_attributes(
            ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        );
        assert!(!conj.check(&project));
        project.set_category(ProjectCategory::Stage);
        assert!(conj.check(&project));
        project.set_attributes(
            ProjectAttributes::from_attributes(vec![ProjectAttribute::Academic]).unwrap(),
        );
        assert!(!conj.check(&project));
    }

    #[test]
    fn test_tautology() {
        let conj = ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        };
        let query = ProjectQuery::from_conjunctions(vec![conj]).unwrap();
        let mut project = test_model::new_general_project(test_model::new_user_id());
        assert!(query.check(&project));
        project.set_attributes(
            ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        );
        assert!(query.check(&project));
    }

    #[test]
    fn test_contradiction() {
        let query = ProjectQuery::from_conjunctions(vec![]).unwrap();
        let mut project = test_model::new_general_project(test_model::new_user_id());
        assert!(!query.check(&project));
        project.set_attributes(
            ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        );
        assert!(!query.check(&project));
    }

    #[test]
    fn test_complex() {
        let conj1 = ProjectQueryConjunction {
            category: Some(ProjectCategory::Stage),
            attributes: ProjectAttributes::from_attributes(vec![ProjectAttribute::Academic])
                .unwrap(),
        };
        let conj2 = ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        };
        let query = ProjectQuery::from_conjunctions(vec![conj1, conj2]).unwrap();
        let mut project = test_model::new_general_project(test_model::new_user_id());
        project.set_attributes(
            ProjectAttributes::from_attributes(vec![ProjectAttribute::Academic]).unwrap(),
        );
        assert!(!query.check(&project));
        project.set_category(ProjectCategory::Stage);
        assert!(query.check(&project));
        project.set_attributes(
            ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        );
        assert!(query.check(&project));
        project.set_category(ProjectCategory::General);
        assert!(query.check(&project));
    }
}
