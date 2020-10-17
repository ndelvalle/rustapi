use wither::mongodb;

use crate::settings::Settings;

#[derive(Clone)]
pub struct Database {
    pub conn: mongodb::Database,
}

impl Database {
    pub async fn new(settings: &Settings) -> Result<Self, mongodb::error::Error> {
        let db_uri = settings.database.uri.as_str();
        let db_name = settings.database.name.as_str();
        let connection = mongodb::Client::with_uri_str(db_uri)
            .await?
            .database(db_name);

        Ok(Self { conn: connection })
    }
}
