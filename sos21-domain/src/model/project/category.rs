use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectCategory {
    General,
    Stage,
    Cooking,
    Food,
}
