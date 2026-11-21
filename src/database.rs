use sqlx::migrate::MigrateError;
use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn connect(uri: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new().connect(uri).await
}

pub async fn migrate(db: &PgPool) -> Result<(), MigrateError> {
    sqlx::migrate!().run(db).await
}
