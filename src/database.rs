use mongodb::Database;
use tokio::sync::OnceCell;
use wither::mongodb;

use crate::settings::SETTINGS;

static CONNECTION: OnceCell<Database> = OnceCell::const_new();

pub async fn connection() -> &'static Database {
  CONNECTION
    .get_or_init(|| async {
      let db_uri = SETTINGS.database.uri.as_str();
      let db_name = SETTINGS.database.name.as_str();

      mongodb::Client::with_uri_str(db_uri)
        .await
        .expect("Failed to initialize MongoDB connection")
        .database(db_name)
    })
    .await
}
