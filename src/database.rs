use mongodb::error::Error as MongoError;
use mongodb::Database as MongoDatabase;
use wither::mongodb;

use crate::settings::get_settings;

// The Rust compiler is allowed to assume that the value a shared reference
// points to will not change while that reference lives.
static mut GLOBAL_CONNECTION: Option<MongoDatabase> = None;

pub async fn setup() -> Result<(), MongoError> {
  unsafe {
    if GLOBAL_CONNECTION.is_some() {
      panic!("Database already initialized");
    }
  };

  let settings = get_settings();
  let db_uri = settings.database.uri.as_str();
  let db_name = settings.database.name.as_str();
  let connection = mongodb::Client::with_uri_str(db_uri)
    .await?
    .database(db_name);

  unsafe {
    GLOBAL_CONNECTION = Some(connection);
  };

  Ok(())
}

pub fn get_connection() -> &'static MongoDatabase {
  unsafe {
    GLOBAL_CONNECTION
      .as_ref()
      .expect("Database connection not initialized")
  }
}
