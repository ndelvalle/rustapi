use axum::http::StatusCode;
use axum::{routing::get, Json, Router};
use serde::Serialize;
use tracing::{debug, error};

use crate::database;

pub fn create_route() -> Router {
    Router::new().route("/status", get(get_status))
}

// Reports whether the API can reach the database, which makes it usable as a readiness probe.
async fn get_status() -> (StatusCode, Json<Status>) {
    let result = sqlx::query("select 1")
        .execute(database::connection().await)
        .await;

    match result {
        Ok(_) => {
            debug!("Returning status");
            (StatusCode::OK, Json(Status { status: "ok" }))
        }
        Err(err) => {
            error!("Database is unreachable: {err:?}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(Status {
                    status: "unavailable",
                }),
            )
        }
    }
}

#[derive(Serialize, Debug)]
struct Status {
    status: &'static str,
}
