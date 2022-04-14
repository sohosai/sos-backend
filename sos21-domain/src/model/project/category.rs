use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectCategory {
    GeneralOnline,
    GeneralPhysical,
    StageOnline,
    StagePhysical,
    CookingPhysical,
    FoodPhysical,
}

impl ProjectCategory {
    pub fn enumerate() -> impl Iterator<Item = ProjectCategory> {
        [
            ProjectCategory::GeneralOnline,
            ProjectCategory::GeneralPhysical,
            ProjectCategory::StageOnline,
            ProjectCategory::StagePhysical,
            ProjectCategory::CookingPhysical,
            ProjectCategory::FoodPhysical,
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
            "general_online" | "GeneralOnline" => Ok(ProjectCategory::GeneralOnline),
            "general_physical" | "GeneralPhysical" => Ok(ProjectCategory::GeneralPhysical),
            "stage_online" | "Stage_online" => Ok(ProjectCategory::StageOnline),
            "stage_physical" | "StagePhysical" => Ok(ProjectCategory::StagePhysical),
            "cooking_physical" | "CookingPhysical" => Ok(ProjectCategory::CookingPhysical),
            "food_physical" | "FoodPhysical" => Ok(ProjectCategory::FoodPhysical),
            _ => Err(ParseCategoryError { _priv: () }),
        }
    }
}
