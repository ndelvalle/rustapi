use axum::Router;
use http::header;
use std::net::SocketAddr;
use tower_http::{
  compression::CompressionLayer, propagate_header::PropagateHeaderLayer,
  sensitive_headers::SetSensitiveRequestHeadersLayer, trace::TraceLayer,
};
use tracing::info;

mod database;
mod lib;
mod logger;
mod models;
mod routes;
mod settings;

// use context::Context;
use database::Database;
use logger::Logger;
use settings::Settings;

#[tokio::main]
async fn main() {
  let settings = match Settings::new() {
    Ok(value) => value,
    Err(err) => panic!("Failed to setup configuration. Error: {}", err),
  };

  Logger::setup(&settings);

  let database = match Database::setup(&settings).await {
    Ok(value) => value,
    Err(_) => panic!("Failed to setup database connection"),
  };

  // let context = web::Data::new(Context {
  //     database: database.clone(),
  // });

  // build our application with a route
  let app = Router::new()
    .merge(routes::cat::create_route())
    // Mark the `Authorization` request header as sensitive so it doesn't show in logs
    .layer(SetSensitiveRequestHeadersLayer::new(std::iter::once(
      header::AUTHORIZATION,
    )))
    // High level logging of requests and responses
    .layer(TraceLayer::new_for_http())
    // Compress responses
    .layer(CompressionLayer::new())
    // Propagate `X-Request-Id`s from requests to responses
    .layer(PropagateHeaderLayer::new(header::HeaderName::from_static(
      "x-request-id",
    )));

  let port = settings.server.port;
  let address = SocketAddr::from(([127, 0, 0, 1], port));

  info!("listening on {}", &address);

  axum::Server::bind(&address)
    .serve(app.into_make_service())
    .await
    .expect("Failed to start server");
}
