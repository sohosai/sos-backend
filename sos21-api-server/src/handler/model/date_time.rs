use chrono::{serde::ts_milliseconds, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DateTime(#[serde(with = "ts_milliseconds")] pub chrono::DateTime<Utc>);

impl DateTime {
    pub fn from_use_case(utc: chrono::DateTime<Utc>) -> Self {
        DateTime(utc)
    }

    pub fn into_use_case(self) -> chrono::DateTime<Utc> {
        self.0
    }
}
