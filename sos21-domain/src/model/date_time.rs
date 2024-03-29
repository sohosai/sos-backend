/// A point of time without timezone-related semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime(chrono::DateTime<chrono::Utc>);

impl DateTime {
    pub fn now() -> Self {
        Self::from_utc(chrono::Utc::now())
    }

    pub fn from_utc(utc: chrono::DateTime<chrono::Utc>) -> Self {
        DateTime(utc)
    }

    pub fn utc(&self) -> chrono::DateTime<chrono::Utc> {
        self.0
    }

    pub fn jst(&self) -> chrono::DateTime<chrono::FixedOffset> {
        // FIXME: 範囲外の場合のハンドリングをする
        let jst = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
        self.0.with_timezone(&jst)
    }
}
