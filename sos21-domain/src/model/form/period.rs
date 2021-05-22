use crate::model::date_time::DateTime;

use thiserror::Error;

#[derive(Debug, Clone, Copy)]
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

    pub fn set_starts_at(&mut self, starts_at: DateTime) -> Result<(), PeriodError> {
        if starts_at >= self.ends_at {
            return Err(PeriodError { _priv: () });
        }

        self.starts_at = starts_at;
        Ok(())
    }

    pub fn set_ends_at(&mut self, ends_at: DateTime) -> Result<(), PeriodError> {
        if self.starts_at >= ends_at {
            return Err(PeriodError { _priv: () });
        }

        self.ends_at = ends_at;
        Ok(())
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
