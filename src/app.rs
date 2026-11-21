use axum::http::{header, StatusCode};
use axum::Router;
use std::time::Duration;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    sensitive_headers::SetSensitiveHeadersLayer,
    timeout::TimeoutLayer,
    trace,
};

use crate::database;
use crate::logger;
use crate::routes;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const REQUEST_BODY_LIMIT: usize = 1024 * 1024;

pub async fn create_app() -> Router {
    logger::setup();

    database::migrate()
        .await
        .expect("Failed to run database migrations");

    Router::new()
        .merge(routes::status::create_route())
        .merge(routes::user::create_route())
        .merge(Router::new().nest(
            "/v1",
            // All public v1 routes will be nested here.
            Router::new().merge(routes::cat::create_route()),
        ))
        // Abort requests that take too long with a 408 status code
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
        // Reject request bodies that are too large with a 413 status code
        .layer(RequestBodyLimitLayer::new(REQUEST_BODY_LIMIT))
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
        .layer(PropagateRequestIdLayer::x_request_id())
        // Generate an `X-Request-Id` for requests that come without one
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        // CORS configuration. This should probably be more restrictive in
        // production.
        .layer(CorsLayer::permissive())
}
