use chrono::{DateTime,Utc};

#[derive(Debug, Clone, Copy)]
pub struct ProjectCreationAvailability {
    pub timestamp: DateTime<Utc>,
    pub general_online: bool,
    pub general_physical: bool,
    pub stage_online: bool,
    pub stage_physical: bool,
    pub cooking_physical: bool,
    pub food_physical: bool,
}
