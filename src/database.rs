use diesel_async::pooled_connection::deadpool::{self, Pool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use once_cell::sync::OnceCell;

use crate::settings::SETTINGS;

type PooledConnection = deadpool::Object<AsyncPgConnection>;

static CONNECTION: OnceCell<Pool<AsyncPgConnection>> = OnceCell::new();

pub async fn get_connection() -> PooledConnection {
    let conn = get_connection_pool()
        .await
        .get()
        .await
        .expect("Failed to get connection from the Pool");

    conn
}

async fn get_connection_pool() -> &'static Pool<AsyncPgConnection> {
    CONNECTION.get_or_init(|| {
        let db_uri = SETTINGS.database.uri.as_str();
        let db_name = SETTINGS.database.name.as_str();
        let db_url = format!("{}/{}", db_uri, db_name);

        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
        Pool::builder(config)
            .build()
            .expect("Failed to initialize PostgreSQL connection pool")
    })
}
