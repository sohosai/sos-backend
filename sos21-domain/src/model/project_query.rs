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
