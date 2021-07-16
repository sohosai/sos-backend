use serde::{Deserialize, Serialize};

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
