use axum::extract::Extension;
use axum::Router;
use http::header;
use std::net::SocketAddr;
use tower_http::{
  compression::CompressionLayer, propagate_header::PropagateHeaderLayer,
  sensitive_headers::SetSensitiveHeadersLayer, trace,
};
use tracing::info;

mod context;
mod database;
mod errors;
mod lib;
mod logger;
mod models;
mod routes;
mod settings;

use context::Context;
use database::Database;
use errors::Error;
use logger::Logger;
use models::Models;

use crate::settings::get_settings;

#[tokio::main]
async fn main() {
  Logger::setup();

  let db = match Database::setup().await {
    Ok(value) => value,
    Err(_) => panic!("Failed to setup database connection"),
  };

  let models = match Models::setup(db.clone()).await {
    Ok(value) => value,
    Err(err) => panic!("Failed to setup models {}", err),
  };

  let context = Context::new(models);

  let app = Router::new()
    .merge(routes::user::create_route())
    .merge(routes::cat::create_route())
    // High level logging of requests and responses
    .layer(
      trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().include_headers(true))
        .on_request(trace::DefaultOnRequest::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO)),
    )
    // Mark the `Authorization` request header as sensitive so it doesn't
    // show in logs.
    .layer(SetSensitiveHeadersLayer::new(std::iter::once(
      header::AUTHORIZATION,
    )))
    // Compress responses
    .layer(CompressionLayer::new())
    // Propagate `X-Request-Id`s from requests to responses
    .layer(PropagateHeaderLayer::new(header::HeaderName::from_static(
      "x-request-id",
    )))
    .layer(Extension(context));

  let settings = get_settings();
  let port = settings.server.port;
  let address = SocketAddr::from(([127, 0, 0, 1], port));

  info!("listening on {}", &address);

  axum::Server::bind(&address)
    .serve(app.into_make_service())
    .await
    .expect("Failed to start server");
}
