use sqlx::migrate::MigrateError;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tokio::sync::OnceCell;

use crate::settings::SETTINGS;

static CONNECTION: OnceCell<PgPool> = OnceCell::const_new();

pub async fn connection() -> &'static PgPool {
    CONNECTION
        .get_or_init(|| async {
            PgPoolOptions::new()
                .connect(SETTINGS.database.uri.as_str())
                .await
                .expect("Failed to initialize PostgreSQL connection")
        })
        .await
}

// Applies the pending migrations from the migrations folder.
pub async fn migrate() -> Result<(), MigrateError> {
    sqlx::migrate!().run(connection().await).await
}
