use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ProjectAttribute {
    Academic,
    Artistic,
    Committee,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProjectAttributes(HashSet<ProjectAttribute>);

#[derive(Debug, Error, Clone)]
#[error("duplicated project attributes")]
pub struct DuplicatedAttributesError {
    _priv: (),
}

impl ProjectAttributes {
    pub fn from_attributes<I>(attributes: I) -> Result<Self, DuplicatedAttributesError>
    where
        I: IntoIterator<Item = ProjectAttribute>,
    {
        let mut result = HashSet::new();
        for attribute in attributes {
            if !result.insert(attribute) {
                return Err(DuplicatedAttributesError { _priv: () });
            }
        }
        Ok(ProjectAttributes(result))
    }

    pub fn contains(&self, attribute: ProjectAttribute) -> bool {
        self.0.contains(&attribute)
    }

    pub fn is_subset(&self, attributes: &ProjectAttributes) -> bool {
        self.0.is_subset(&attributes.0)
    }

    pub fn attributes(&self) -> impl Iterator<Item = ProjectAttribute> + '_ {
        self.0.iter().copied()
    }
}
