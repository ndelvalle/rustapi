use chrono::{DateTime, Utc};

pub type Date = DateTime<Utc>;

pub fn now() -> Date {
    Utc::now()
}
