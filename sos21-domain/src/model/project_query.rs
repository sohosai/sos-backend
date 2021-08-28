use std::collections::HashSet;

use crate::model::bound::{Bounded, Unbounded};
use crate::model::collection::LengthLimitedVec;
use crate::model::pending_project::PendingProject;
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

    fn check_category_attributes(
        &self,
        category: ProjectCategory,
        attributes: &ProjectAttributes,
    ) -> bool {
        if let Some(expected) = self.category {
            if expected != category {
                return false;
            }
        }

        self.attributes.is_subset(attributes)
    }

    #[auto_enums::auto_enum(Iterator)]
    pub fn possible_categories(&self) -> impl Iterator<Item = ProjectCategory> {
        if let Some(expected) = self.category {
            std::iter::once(expected)
        } else {
            ProjectCategory::enumerate()
        }
    }

    pub fn check_project(&self, project: &Project) -> bool {
        self.check_category_attributes(project.category(), project.attributes())
    }

    pub fn check_pending_project(&self, pending_project: &PendingProject) -> bool {
        self.check_category_attributes(pending_project.category(), pending_project.attributes())
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

    pub fn possible_categories(&self) -> impl Iterator<Item = ProjectCategory> {
        let categories: HashSet<_> = self
            .conjunctions()
            .map(|conj| conj.possible_categories())
            .flatten()
            .collect();
        categories.into_iter()
    }

    pub fn check_project(&self, project: &Project) -> bool {
        self.conjunctions().any(|conj| conj.check_project(project))
    }

    pub fn check_pending_project(&self, pending_project: &PendingProject) -> bool {
        self.conjunctions()
            .any(|conj| conj.check_pending_project(pending_project))
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
        let project1 = test_model::new_general_project(test_model::new_user_id());
        assert!(conj.check_project(&project1));
        let project2 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ],
        );
        assert!(conj.check_project(&project2));
    }

    #[test]
    fn test_pending_project_conj_tautology() {
        let conj = ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        };
        let pending_project = test_model::new_general_pending_project(test_model::new_user_id());
        assert!(conj.check_pending_project(&pending_project));
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ],
        );
        assert!(conj.check_pending_project(&pending_project));
    }

    #[test]
    fn test_conj_attribute_single() {
        let conj = ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![ProjectAttribute::Artistic])
                .unwrap(),
        };
        let project1 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[],
        );
        assert!(!conj.check_project(&project1));
        let project2 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[ProjectAttribute::Artistic, ProjectAttribute::Committee],
        );
        assert!(conj.check_project(&project2));
        let project3 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ],
        );
        assert!(conj.check_project(&project3));
        let project4 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[ProjectAttribute::Academic, ProjectAttribute::Committee],
        );
        assert!(!conj.check_project(&project4));
    }

    #[test]
    fn test_pending_project_conj_attribute_single() {
        let conj = ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![ProjectAttribute::Artistic])
                .unwrap(),
        };
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[],
        );
        assert!(!conj.check_pending_project(&pending_project));
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[ProjectAttribute::Artistic, ProjectAttribute::Committee],
        );
        assert!(conj.check_pending_project(&pending_project));
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ],
        );
        assert!(conj.check_pending_project(&pending_project));
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[ProjectAttribute::Academic, ProjectAttribute::Committee],
        );
        assert!(!conj.check_pending_project(&pending_project));
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
        let project1 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[],
        );
        assert!(!conj.check_project(&project1));
        let project2 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[ProjectAttribute::Artistic, ProjectAttribute::Committee],
        );
        assert!(conj.check_project(&project2));
        let project3 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ],
        );
        assert!(conj.check_project(&project3));
        let project4 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[ProjectAttribute::Artistic, ProjectAttribute::Academic],
        );
        assert!(!conj.check_project(&project4));
    }

    #[test]
    fn test_pending_project_conj_attribute_multiple() {
        let conj = ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Artistic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        };
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[],
        );
        assert!(!conj.check_pending_project(&pending_project));
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[ProjectAttribute::Artistic, ProjectAttribute::Committee],
        );
        assert!(conj.check_pending_project(&pending_project));
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ],
        );
        assert!(conj.check_pending_project(&pending_project));
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[ProjectAttribute::Artistic, ProjectAttribute::Academic],
        );
        assert!(!conj.check_pending_project(&pending_project));
    }

    #[test]
    fn test_conj_category() {
        let conj = ProjectQueryConjunction {
            category: Some(ProjectCategory::General),
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        };
        assert!(conj.check_project(&test_model::new_general_project(test_model::new_user_id())));
        assert!(!conj.check_project(&test_model::new_stage_project(test_model::new_user_id())));
    }

    #[test]
    fn test_pending_project_conj_category() {
        let conj = ProjectQueryConjunction {
            category: Some(ProjectCategory::General),
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        };
        assert!(
            conj.check_pending_project(&test_model::new_general_pending_project(
                test_model::new_user_id()
            ))
        );
        assert!(
            !conj.check_pending_project(&test_model::new_stage_pending_project(
                test_model::new_user_id()
            ))
        );
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
        let project1 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[],
        );
        assert!(!conj.check_project(&project1));
        let project2 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[ProjectAttribute::Academic, ProjectAttribute::Committee],
        );
        assert!(!conj.check_project(&project2));
        let project3 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::Stage,
            &[ProjectAttribute::Academic, ProjectAttribute::Committee],
        );
        assert!(conj.check_project(&project3));
        let project4 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::Stage,
            &[ProjectAttribute::Academic],
        );
        assert!(!conj.check_project(&project4));
    }

    #[test]
    fn test_pending_project_conj_complex() {
        let conj = ProjectQueryConjunction {
            category: Some(ProjectCategory::Stage),
            attributes: ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        };
        let pending_project = test_model::new_general_pending_project(test_model::new_user_id());
        assert!(!conj.check_pending_project(&pending_project));
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[ProjectAttribute::Academic, ProjectAttribute::Committee],
        );
        assert!(!conj.check_pending_project(&pending_project));
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::Stage,
            &[ProjectAttribute::Academic, ProjectAttribute::Committee],
        );
        assert!(conj.check_pending_project(&pending_project));
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::Stage,
            &[ProjectAttribute::Academic],
        );
        assert!(!conj.check_pending_project(&pending_project));
    }

    #[test]
    fn test_tautology() {
        let conj = ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        };
        let query = ProjectQuery::from_conjunctions(vec![conj]).unwrap();
        let project1 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[],
        );
        assert!(query.check_project(&project1));
        let project2 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ],
        );
        assert!(query.check_project(&project2));
    }

    #[test]
    fn test_pending_project_tautology() {
        let conj = ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        };
        let query = ProjectQuery::from_conjunctions(vec![conj]).unwrap();
        let pending_project = test_model::new_general_pending_project(test_model::new_user_id());
        assert!(query.check_pending_project(&pending_project));
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ],
        );
        assert!(query.check_pending_project(&pending_project));
    }

    #[test]
    fn test_contradiction() {
        let query = ProjectQuery::from_conjunctions(vec![]).unwrap();
        let project1 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[],
        );
        assert!(!query.check_project(&project1));
        let project2 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ],
        );
        assert!(!query.check_project(&project2));
    }

    #[test]
    fn test_pending_project_contradiction() {
        let query = ProjectQuery::from_conjunctions(vec![]).unwrap();
        let pending_project = test_model::new_general_pending_project(test_model::new_user_id());
        assert!(!query.check_pending_project(&pending_project));
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ],
        );
        assert!(!query.check_pending_project(&pending_project));
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
        let project1 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[ProjectAttribute::Academic],
        );
        assert!(!query.check_project(&project1));
        let project2 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::Stage,
            &[ProjectAttribute::Academic],
        );
        assert!(query.check_project(&project2));
        let project3 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::Stage,
            &[ProjectAttribute::Academic, ProjectAttribute::Committee],
        );
        assert!(query.check_project(&project3));
        let project4 = test_model::new_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[ProjectAttribute::Academic, ProjectAttribute::Committee],
        );
        assert!(query.check_project(&project4));
    }

    #[test]
    fn test_pending_project_complex() {
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
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[ProjectAttribute::Academic],
        );
        assert!(!query.check_pending_project(&pending_project));
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::Stage,
            &[ProjectAttribute::Academic],
        );
        assert!(query.check_pending_project(&pending_project));
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::Stage,
            &[ProjectAttribute::Academic, ProjectAttribute::Committee],
        );
        assert!(query.check_pending_project(&pending_project));
        let pending_project = test_model::new_pending_project_with_attributes(
            test_model::new_user_id(),
            ProjectCategory::General,
            &[ProjectAttribute::Academic, ProjectAttribute::Committee],
        );
        assert!(query.check_pending_project(&pending_project));
    }

    #[test]
    fn test_conj_possible_categories() {
        use std::collections::HashSet;

        let conj1 = ProjectQueryConjunction {
            category: Some(ProjectCategory::Stage),
            attributes: ProjectAttributes::from_attributes(vec![ProjectAttribute::Academic])
                .unwrap(),
        };
        assert_eq!(
            conj1.possible_categories().collect::<Vec<_>>(),
            vec![ProjectCategory::Stage]
        );
        let conj2 = ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![ProjectAttribute::Academic])
                .unwrap(),
        };
        assert_eq!(
            conj2.possible_categories().collect::<HashSet<_>>(),
            ProjectCategory::enumerate().collect::<HashSet<_>>()
        );
    }

    #[test]
    fn test_possible_categories() {
        use std::collections::HashSet;

        let conj1 = ProjectQueryConjunction {
            category: Some(ProjectCategory::Stage),
            attributes: ProjectAttributes::from_attributes(vec![ProjectAttribute::Academic])
                .unwrap(),
        };
        let conj2 = ProjectQueryConjunction {
            category: Some(ProjectCategory::Food),
            attributes: ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        };
        let query1 = ProjectQuery::from_conjunctions(vec![conj1, conj2]).unwrap();
        assert_eq!(
            query1.possible_categories().collect::<HashSet<_>>(),
            [ProjectCategory::Stage, ProjectCategory::Food]
                .iter()
                .copied()
                .collect::<HashSet<_>>()
        );

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
        let query2 = ProjectQuery::from_conjunctions(vec![conj1, conj2]).unwrap();
        assert_eq!(
            query2.possible_categories().collect::<HashSet<_>>(),
            ProjectCategory::enumerate().collect::<HashSet<_>>()
        );
    }
}
