use serde::Serializer;
use wither::bson::{oid::ObjectId, DateTime};

use crate::lib::date;

pub fn serialize_bson_datetime_as_iso_string<S: Serializer>(
  date: &DateTime,
  serializer: S,
) -> Result<S::Ok, S::Error> {
  let iso_string = date::to_rfc3339(*date);
  serializer.serialize_str(&iso_string)
}

pub fn serialize_oid_as_hex_string<S: Serializer>(
  id: &ObjectId,
  serializer: S,
) -> Result<S::Ok, S::Error> {
  let hex_string = id.to_hex();
  serializer.serialize_str(&hex_string)
}
