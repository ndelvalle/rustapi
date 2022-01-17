use wither::bson::DateTime;

pub fn to_rfc3339(date: DateTime) -> String {
  date.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}