use std::collections::HashSet;

use crate::model::bound::{Bounded, Unbounded};
use crate::model::collection::{self, LengthLimitedSet};
use crate::model::project::{Project, ProjectId};
use crate::model::project_query::ProjectQuery;

use serde::{
    de::{self, Deserializer},
    Deserialize, Serialize,
};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormCondition {
    pub query: ProjectQuery,
    pub includes: FormConditionProjectSet,
    pub excludes: FormConditionProjectSet,
}

impl FormCondition {
    pub fn check(&self, project: &Project) -> bool {
        if self.excludes.contains(project.id()) {
            return false;
        }

        self.query.check_project(project) || self.includes.contains(project.id())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct FormConditionProjectSet(LengthLimitedSet<Unbounded, Bounded<typenum::U1024>, ProjectId>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromProjectsErrorKind {
    Duplicated(ProjectId),
    TooLong,
}

#[derive(Debug, Error, Clone)]
#[error("invalid form condition project set")]
pub struct FromProjectsError {
    kind: FromProjectsErrorKind,
}

impl FromProjectsError {
    pub fn kind(&self) -> FromProjectsErrorKind {
        self.kind
    }

    fn from_length_error(e: collection::LengthError<Unbounded, Bounded<typenum::U1024>>) -> Self {
        let kind = match e.kind() {
            collection::LengthErrorKind::TooLong => FromProjectsErrorKind::TooLong,
            // TODO: statically assert unreachability
            collection::LengthErrorKind::TooShort => unreachable!(),
        };
        FromProjectsError { kind }
    }
}

impl FormConditionProjectSet {
    pub fn from_projects<I>(projects: I) -> Result<Self, FromProjectsError>
    where
        I: IntoIterator<Item = ProjectId>,
    {
        let mut result = LengthLimitedSet::new(HashSet::new()).unwrap();
        for project in projects {
            let has_inserted = result
                .insert(project)
                .map_err(FromProjectsError::from_length_error)?;
            if !has_inserted {
                return Err(FromProjectsError {
                    kind: FromProjectsErrorKind::Duplicated(project),
                });
            }
        }
        Ok(FormConditionProjectSet(result))
    }

    pub fn projects(&self) -> impl Iterator<Item = ProjectId> + '_ {
        self.0.iter().copied()
    }

    pub fn contains(&self, project_id: ProjectId) -> bool {
        self.0.contains(&project_id)
    }

    pub fn difference<'a>(
        &'a self,
        other: &'a FormConditionProjectSet,
    ) -> impl Iterator<Item = ProjectId> + 'a {
        self.0.difference(&other.0).copied()
    }
}

impl<'de> Deserialize<'de> for FormConditionProjectSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        FormConditionProjectSet::from_projects(Vec::<ProjectId>::deserialize(deserializer)?)
            .map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::{FormCondition, FormConditionProjectSet};
    use crate::{
        model::{
            project::{ProjectAttribute, ProjectAttributes},
            project_query::{ProjectQuery, ProjectQueryConjunction},
        },
        test::model as test_model,
    };

    #[test]
    fn test_tautology() {
        let conj = ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        };
        let query = ProjectQuery::from_conjunctions(vec![conj]).unwrap();
        let condition = FormCondition {
            query,
            includes: FormConditionProjectSet::from_projects(vec![]).unwrap(),
            excludes: FormConditionProjectSet::from_projects(vec![]).unwrap(),
        };

        let project1 = test_model::new_general_project(test_model::new_user_id());
        assert!(condition.check(&project1));
        let mut project2 = test_model::new_general_project(test_model::new_user_id());
        project2.set_attributes(
            ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        );
        assert!(condition.check(&project2));
    }

    #[test]
    fn test_exclude_tautology() {
        let project1 = test_model::new_general_project(test_model::new_user_id());

        let conj = ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![]).unwrap(),
        };
        let query = ProjectQuery::from_conjunctions(vec![conj]).unwrap();
        let condition = FormCondition {
            query,
            includes: FormConditionProjectSet::from_projects(vec![]).unwrap(),
            excludes: FormConditionProjectSet::from_projects(vec![project1.id()]).unwrap(),
        };

        assert!(!condition.check(&project1));

        let project2 = test_model::new_general_project(test_model::new_user_id());
        assert!(condition.check(&project2));
    }

    #[test]
    fn test_contradiction() {
        let query = ProjectQuery::from_conjunctions(vec![]).unwrap();
        let condition = FormCondition {
            query,
            includes: FormConditionProjectSet::from_projects(vec![]).unwrap(),
            excludes: FormConditionProjectSet::from_projects(vec![]).unwrap(),
        };

        let project1 = test_model::new_general_project(test_model::new_user_id());
        assert!(!condition.check(&project1));
        let mut project2 = test_model::new_general_project(test_model::new_user_id());
        project2.set_attributes(
            ProjectAttributes::from_attributes(vec![
                ProjectAttribute::Artistic,
                ProjectAttribute::Academic,
                ProjectAttribute::Committee,
            ])
            .unwrap(),
        );
        assert!(!condition.check(&project2));
    }

    #[test]
    fn test_include_contradiction() {
        let project1 = test_model::new_general_project(test_model::new_user_id());

        let query = ProjectQuery::from_conjunctions(vec![]).unwrap();
        let condition = FormCondition {
            query,
            includes: FormConditionProjectSet::from_projects(vec![project1.id()]).unwrap(),
            excludes: FormConditionProjectSet::from_projects(vec![]).unwrap(),
        };

        assert!(condition.check(&project1));
        let project2 = test_model::new_general_project(test_model::new_user_id());
        assert!(!condition.check(&project2));
    }

    #[test]
    fn test_include_exclude() {
        let mut project1 = test_model::new_general_project(test_model::new_user_id());
        project1.set_attributes(ProjectAttributes::from_attributes(vec![]).unwrap());
        let mut project2 = test_model::new_general_project(test_model::new_user_id());
        project2.set_attributes(
            ProjectAttributes::from_attributes(vec![ProjectAttribute::Academic]).unwrap(),
        );

        let conj = ProjectQueryConjunction {
            category: None,
            attributes: ProjectAttributes::from_attributes(vec![ProjectAttribute::Academic])
                .unwrap(),
        };
        let query = ProjectQuery::from_conjunctions(vec![conj]).unwrap();
        let condition = FormCondition {
            query,
            includes: FormConditionProjectSet::from_projects(vec![project1.id()]).unwrap(),
            excludes: FormConditionProjectSet::from_projects(vec![project2.id()]).unwrap(),
        };

        assert!(condition.check(&project1));
        assert!(!condition.check(&project2));
    }
}
