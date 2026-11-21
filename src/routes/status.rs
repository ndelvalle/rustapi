use axum::extract::State;
use axum::http::StatusCode;
use axum::{routing::get, Json, Router};
use serde::Serialize;
use tracing::{debug, error};

use crate::app::Context;

pub fn create_route() -> Router<Context> {
    Router::new().route("/status", get(get_status))
}

async fn get_status(State(ctx): State<Context>) -> (StatusCode, Json<Status>) {
    let result = sqlx::query("select 1").execute(&ctx.db).await;

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
