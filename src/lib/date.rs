use chrono::Utc;

pub type Date = bson::DateTime;

pub fn now() -> Date {
  Utc::now().into()
}
