use crate::model::date_time::DateTime;

use thiserror::Error;

#[derive(Debug, Clone, Copy)]
enum ProjectCreationPeriodInner {
    Always,
    Never,
    Range {
        starts_at: DateTime,
        ends_at: DateTime,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectCreationPeriod {
    inner: ProjectCreationPeriodInner,
}

#[derive(Debug, Error, Clone)]
#[error("invalid project creation period")]
pub struct PeriodError {
    _priv: (),
}

impl ProjectCreationPeriod {
    pub fn from_datetime(starts_at: DateTime, ends_at: DateTime) -> Result<Self, PeriodError> {
        if starts_at >= ends_at {
            return Err(PeriodError { _priv: () });
        }

        Ok(ProjectCreationPeriod {
            inner: ProjectCreationPeriodInner::Range { starts_at, ends_at },
        })
    }

    pub fn always() -> Self {
        ProjectCreationPeriod {
            inner: ProjectCreationPeriodInner::Always,
        }
    }

    pub fn never() -> Self {
        ProjectCreationPeriod {
            inner: ProjectCreationPeriodInner::Never,
        }
    }

    pub fn is_after(&self, time: DateTime) -> bool {
        match self.inner {
            ProjectCreationPeriodInner::Always => false,
            ProjectCreationPeriodInner::Never => true,
            ProjectCreationPeriodInner::Range { starts_at, .. } => starts_at > time,
        }
    }

    pub fn contains(&self, time: DateTime) -> bool {
        match self.inner {
            ProjectCreationPeriodInner::Always => true,
            ProjectCreationPeriodInner::Never => false,
            ProjectCreationPeriodInner::Range { starts_at, ends_at } => {
                starts_at <= time && ends_at > time
            }
        }
    }
}
