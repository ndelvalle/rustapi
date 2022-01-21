use serde::{Deserialize, Serialize};
use wither::bson::DateTime;
use wither::bson::{doc, oid::ObjectId};
use wither::Model as WitherModel;

use crate::database::Database;
use crate::lib::serde::serialize_bson_datetime_as_iso_string;
use crate::lib::serde::serialize_oid_as_hex_string;
use crate::models::ModelExt;

#[derive(Clone)]
pub struct Cat {
  pub db: Database,
}

impl Cat {
  pub fn new(db: Database) -> Self {
    Self { db }
  }
}

impl ModelExt for Cat {
  type T = Model;
  fn get_database(&self) -> &Database {
    &self.db
  }
}

#[derive(WitherModel, Debug, Clone, Serialize, Deserialize)]
#[model(index(keys = r#"doc!{ "user": 1 }"#))]
pub struct Model {
  #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
  pub id: Option<ObjectId>,
  pub user: ObjectId,
  pub updated_at: DateTime,
  pub created_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicModel {
  #[serde(alias = "_id", serialize_with = "serialize_oid_as_hex_string")]
  pub id: ObjectId,
  #[serde(serialize_with = "serialize_oid_as_hex_string")]
  pub user: ObjectId,
  #[serde(serialize_with = "serialize_bson_datetime_as_iso_string")]
  pub updated_at: DateTime,
  #[serde(serialize_with = "serialize_bson_datetime_as_iso_string")]
  pub created_at: DateTime,
}

impl From<Model> for PublicModel {
  fn from(cat: Model) -> Self {
    Self {
      id: cat.id.clone().unwrap(),
      user: cat.user.clone(),
      updated_at: cat.updated_at,
      created_at: cat.created_at,
    }
  }
}
