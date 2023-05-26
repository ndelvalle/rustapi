use bson::serde_helpers::bson_datetime_as_rfc3339_string;
use bson::serde_helpers::serialize_object_id_as_hex_string;
use serde::{Deserialize, Serialize};
use validator::Validate;
use wither::bson::{doc, oid::ObjectId};
use wither::Model as WitherModel;

use crate::utils::date;
use crate::utils::date::Date;
use crate::utils::models::ModelExt;

impl ModelExt for Cat {}

#[derive(Debug, Clone, Serialize, Deserialize, WitherModel, Validate)]
#[model(index(keys = r#"doc!{ "user": 1, "created_at": 1 }"#))]
pub struct Cat {
  #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
  pub id: Option<ObjectId>,
  pub user: ObjectId,
  pub name: String,
  pub updated_at: Date,
  pub created_at: Date,
}

impl Cat {
  pub fn new(user: ObjectId, name: String) -> Self {
    let now = date::now();
    Self {
      id: None,
      user,
      name,
      updated_at: now,
      created_at: now,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicCat {
  #[serde(alias = "_id", serialize_with = "serialize_object_id_as_hex_string")]
  pub id: ObjectId,
  #[serde(serialize_with = "serialize_object_id_as_hex_string")]
  pub user: ObjectId,
  pub name: String,
  #[serde(with = "bson_datetime_as_rfc3339_string")]
  pub updated_at: Date,
  #[serde(with = "bson_datetime_as_rfc3339_string")]
  pub created_at: Date,
}

impl From<Cat> for PublicCat {
  fn from(cat: Cat) -> Self {
    Self {
      id: cat.id.unwrap(),
      user: cat.user,
      name: cat.name.clone(),
      updated_at: cat.updated_at,
      created_at: cat.created_at,
    }
  }
}
