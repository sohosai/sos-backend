use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectCategory {
    General,
    Stage,
    Cooking,
    Food,
}

impl ProjectCategory {
    pub fn enumerate() -> impl Iterator<Item = ProjectCategory> {
        [
            ProjectCategory::General,
            ProjectCategory::Stage,
            ProjectCategory::Cooking,
            ProjectCategory::Food,
        ]
        .iter()
        .copied()
    }
}

#[derive(Debug, Error, Clone)]
#[error("invalid project category")]
pub struct ParseCategoryError {
    _priv: (),
}

impl FromStr for ProjectCategory {
    type Err = ParseCategoryError;
    fn from_str(s: &str) -> Result<ProjectCategory, Self::Err> {
        match s {
            "general" | "General" => Ok(ProjectCategory::General),
            "stage" | "Stage" => Ok(ProjectCategory::Stage),
            "cooking" | "Cooking" => Ok(ProjectCategory::Cooking),
            "food" | "Food" => Ok(ProjectCategory::Food),
            _ => Err(ParseCategoryError { _priv: () }),
        }
    }
}
