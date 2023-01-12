use mongodb::error::Error as MongoError;
use mongodb::Database as MongoDatabase;
use std::sync::atomic::{AtomicBool, Ordering};
use wither::mongodb;

use crate::settings::SETTINGS;

// The Rust compiler is allowed to assume that the value a shared reference
// points to will not change while that reference lives. CONNECTION is unsafely
// mutated only once on the setup function (This function is called only once).
static mut CONNECTION: Option<MongoDatabase> = None;
static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub async fn setup() -> Result<(), MongoError> {
  let exchange = IS_INITIALIZED.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed);
  let can_setup = exchange == Ok(false);

  if !can_setup {
    panic!("Database already initialized");
  }

  let db_uri = SETTINGS.database.uri.as_str();
  let db_name = SETTINGS.database.name.as_str();
  let connection = mongodb::Client::with_uri_str(db_uri)
    .await?
    .database(db_name);

  unsafe {
    CONNECTION = Some(connection);
  };

  Ok(())
}

pub fn get_connection() -> &'static MongoDatabase {
  unsafe {
    CONNECTION
      .as_ref()
      .expect("Database connection not initialized")
  }
}
