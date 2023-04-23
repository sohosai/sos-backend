use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy)]
pub struct ProjectCreationAvailability {
    pub timestamp: DateTime<Utc>,
    pub general: bool,
    pub cooking_requiring_preparation_area: bool,
    pub cooking: bool,
    pub food: bool,
    pub stage: bool,
}
