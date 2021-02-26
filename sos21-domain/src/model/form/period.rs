use crate::model::date_time::DateTime;

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct FormPeriod {
    starts_at: DateTime,
    ends_at: DateTime,
}

#[derive(Debug, Error, Clone)]
#[error("invalid form period")]
pub struct PeriodError {
    _priv: (),
}

impl FormPeriod {
    pub fn from_datetime(starts_at: DateTime, ends_at: DateTime) -> Result<Self, PeriodError> {
        if starts_at >= ends_at {
            return Err(PeriodError { _priv: () });
        }

        Ok(FormPeriod { starts_at, ends_at })
    }

    pub fn starts_at(&self) -> DateTime {
        self.starts_at
    }

    pub fn ends_at(&self) -> DateTime {
        self.ends_at
    }

    pub fn contains(&self, time: DateTime) -> bool {
        self.starts_at() <= time && self.ends_at() > time
    }
}
