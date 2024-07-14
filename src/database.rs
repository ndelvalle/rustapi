use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio::sync::OnceCell;

use std::time::Duration;

use crate::settings::SETTINGS;

static CONNECTION: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn connection() -> &'static DatabaseConnection {
    CONNECTION
        .get_or_init(|| async {
            let uri = SETTINGS.database.uri.as_str();
            let mut opt = ConnectOptions::new(uri);

            opt.min_connections(100)
                .max_connections(1)
                .connect_timeout(Duration::from_secs(10));

            Database::connect(opt)
                .await
                .expect("Failed to connect to PostgreSQL")
        })
        .await
}
