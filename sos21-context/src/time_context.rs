use chrono::{DateTime, Utc};

pub trait TimeContext {
    fn now(&self) -> DateTime<Utc>;
}
