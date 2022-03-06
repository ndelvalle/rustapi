use bson::serde_helpers::bson_datetime_as_rfc3339_string;
use bson::serde_helpers::serialize_object_id_as_hex_string;
use serde::{Deserialize, Serialize};
use tokio::task;
use validator::Validate;
use wither::bson::{doc, oid::ObjectId};
use wither::Model as WitherModel;

use crate::database::Database;
use crate::errors::Error;
use crate::lib::date;
use crate::lib::date::Date;
use crate::lib::models::ModelExt;

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

#[derive(Debug, Clone, Serialize, Deserialize, WitherModel, Validate)]
#[model(index(keys = r#"doc!{ "email": 1 }"#, options = r#"doc!{ "unique": true }"#))]
pub struct User {
  #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
  pub id: Option<ObjectId>,
  #[validate(length(min = 1))]
  pub name: String,
  #[validate(email)]
  pub email: String,
  pub password: String,
  pub updated_at: Date,
  pub created_at: Date,
  pub locked_at: Option<Date>,
}

impl User {
  pub fn new(name: String, email: String, password_hash: String) -> Self {
    let now = date::now();
    Self {
      id: None,
      name,
      email,
      password: password_hash,
      updated_at: now,
      created_at: now,
      locked_at: None,
    }
  }

  pub fn is_password_match(&self, password: &str) -> bool {
    bcrypt::verify(password, self.password.as_ref()).unwrap_or(false)
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
  fn from(user: User) -> Self {
    Self {
      id: user.id.unwrap(),
      name: user.name.clone(),
      email: user.email.clone(),
      updated_at: user.updated_at,
      created_at: user.created_at,
    }
  }
}

pub async fn hash_password<P>(password: P) -> Result<String, Error>
where
  P: AsRef<str> + Send + 'static,
{
  // TODO: Hash password with salt.
  // https://docs.rs/bcrypt/latest/bcrypt/fn.hash_with_salt.html
  task::spawn_blocking(move || bcrypt::hash(password.as_ref(), bcrypt::DEFAULT_COST))
    .await
    .map_err(Error::RunSyncTask)?
    .map_err(Error::HashPassword)
}
