use serde::{Deserialize, Serialize};
use wither::bson::DateTime;
use wither::bson::{doc, oid::ObjectId};
use wither::Model;

use crate::lib::serde::serialize_bson_datetime_as_iso_string;
use crate::lib::serde::serialize_oid_as_hex_string;

#[derive(Debug, Clone, Model, Serialize, Deserialize)]
#[model(index(keys = r#"doc!{ "user": 1 }"#))]
pub struct Cat {
  #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
  pub id: Option<ObjectId>,
  pub user: ObjectId,
  pub updated_at: DateTime,
  pub created_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatSchema {
  #[serde(alias = "_id", serialize_with = "serialize_oid_as_hex_string")]
  pub id: ObjectId,
  #[serde(serialize_with = "serialize_oid_as_hex_string")]
  pub user: ObjectId,
  #[serde(serialize_with = "serialize_bson_datetime_as_iso_string")]
  pub updated_at: DateTime,
  #[serde(serialize_with = "serialize_bson_datetime_as_iso_string")]
  pub created_at: DateTime,
}

impl From<Cat> for CatSchema {
  fn from(cat: Cat) -> Self {
    Self {
      id: cat.id.unwrap(),
      user: cat.user,
      updated_at: cat.updated_at,
      created_at: cat.created_at,
    }
  }
}
