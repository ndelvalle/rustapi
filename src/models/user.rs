use bson::serde_helpers::bson_datetime_as_rfc3339_string;
use bson::serde_helpers::serialize_object_id_as_hex_string;
use serde::{Deserialize, Serialize};
use wither::bson::{doc, oid::ObjectId};
use wither::Model as WitherModel;

use crate::database::Database;
use crate::lib::date;
use crate::lib::date::Date;
use crate::models::ModelExt;

#[derive(Clone)]
pub struct Model {
  pub db: Database,
}

impl Model {
  pub fn new(db: Database) -> Self {
    Self { db }
  }
}

impl ModelExt for Model {
  type T = User;
  fn get_database(&self) -> &Database {
    &self.db
  }
}

#[derive(WitherModel, Debug, Clone, Serialize, Deserialize)]
#[model(index(keys = r#"doc!{ "email": 1 }"#, options = r#"doc!{ "unique": true }"#))]
pub struct User {
  #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
  pub id: Option<ObjectId>,
  pub name: String,
  pub email: String,
  pub password: String,
  pub updated_at: Date,
  pub created_at: Date,
  pub locked_at: Option<Date>,
}

impl User {
  pub fn new(name: String, email: String) -> Self {
    let now = date::now();
    Self {
      id: None,
      name,
      password: String::from("Password1"), // TODO: Create and hash password.
      email,
      updated_at: now,
      created_at: now,
      locked_at: None,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicUser {
  #[serde(alias = "_id", serialize_with = "serialize_object_id_as_hex_string")]
  pub id: ObjectId,
  pub name: String,
  pub email: String,
  #[serde(with = "bson_datetime_as_rfc3339_string")]
  pub updated_at: Date,
  #[serde(with = "bson_datetime_as_rfc3339_string")]
  pub created_at: Date,
}

impl From<User> for PublicUser {
  fn from(cat: User) -> Self {
    Self {
      id: cat.id.unwrap(),
      name: cat.name.clone(),
      email: cat.email.clone(),
      updated_at: cat.updated_at,
      created_at: cat.created_at,
    }
  }
}
