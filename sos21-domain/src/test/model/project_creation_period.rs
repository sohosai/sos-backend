use crate::model::{date_time::DateTime, project_creation_period::ProjectCreationPeriod};

pub fn mock_project_creation_period_with_start(starts_at: DateTime) -> ProjectCreationPeriod {
    let ends_at = DateTime::from_utc(starts_at.utc() + chrono::Duration::hours(1));
    ProjectCreationPeriod::from_datetime(starts_at, ends_at).unwrap()
}

pub fn mock_project_creation_period_with_end(ends_at: DateTime) -> ProjectCreationPeriod {
    let starts_at = DateTime::from_utc(ends_at.utc() - chrono::Duration::hours(1));
    ProjectCreationPeriod::from_datetime(starts_at, ends_at).unwrap()
}

pub fn new_project_creation_period_from_now() -> ProjectCreationPeriod {
    mock_project_creation_period_with_start(DateTime::now())
}

pub fn new_project_creation_period_to_now() -> ProjectCreationPeriod {
    mock_project_creation_period_with_end(DateTime::now())
}

pub fn new_project_creation_period_with_hours_from_now(hours: i64) -> ProjectCreationPeriod {
    mock_project_creation_period_with_start(DateTime::from_utc(
        chrono::Utc::now() + chrono::Duration::hours(hours),
    ))
}
