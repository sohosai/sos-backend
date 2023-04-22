use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectCategory {
    General,                         // 一般企画（飲食物取扱い企画、調理企画を除く）
    CookingRequiringPreparationArea, // 一般企画（調理企画（仕込場が必要））
    Cooking,                         // 一般企画（調理企画（仕込場が不要））
    Food,                            // 一般企画（飲食物取扱い企画）
    Stage,                           // ステージ企画
}

impl ProjectCategory {
    pub fn enumerate() -> impl Iterator<Item = ProjectCategory> {
        [
            ProjectCategory::General,
            ProjectCategory::CookingRequiringPreparationArea,
            ProjectCategory::Cooking,
            ProjectCategory::Food,
            ProjectCategory::Stage,
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
            "food" | "Food" => Ok(ProjectCategory::Food),
            "cooking_requiring_preparation_area" | "CookingRequiringPreparationArea" => {
                Ok(ProjectCategory::CookingRequiringPreparationArea)
            }
            "cooking" | "Cooking" => Ok(ProjectCategory::Cooking),
            _ => Err(ParseCategoryError { _priv: () }),
        }
    }
}
