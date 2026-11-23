use axum::Router;
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    sensitive_headers::SetSensitiveHeadersLayer,
    timeout::TimeoutLayer,
    trace,
};

use crate::errors::Error;
use crate::routes;
use crate::settings::Settings;

// This is the context shared with every request handler.
#[derive(Clone)]
pub struct Context {
    pub db: PgPool,
    pub settings: Arc<Settings>,
}

pub fn create_app(ctx: Context) -> Router {
    let server = &ctx.settings.server;
    let request_timeout = Duration::from_secs(server.request_timeout_seconds);
    let request_body_limit = server.request_body_limit_bytes;

    Router::new()
        .merge(routes::status::create_route())
        .merge(routes::user::create_route())
        .merge(Router::new().nest(
            "/v1",
            // All public v1 routes will be nested here.
            Router::new().merge(routes::cat::create_route(ctx.clone())),
        ))
        // Answer unknown routes and unsupported methods with the standard error body
        .fallback(async || Error::not_found())
        .method_not_allowed_fallback(async || Error::MethodNotAllowed)
        // Abort requests that take too long with a 408 status code
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            request_timeout,
        ))
        // Reject request bodies that are too large with a 413 status code
        .layer(RequestBodyLimitLayer::new(request_body_limit))
        // Give the errors of the two layers above the standard error body
        .layer(middleware::map_response(standardize_layer_errors))
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
        // Let browsers on the allowed origins call the API
        .layer(cors_layer(&ctx.settings.cors.allowed_origins))
        .with_state(ctx)
}

// The timeout and body limit layers answer on their own, without a JSON body.
async fn standardize_layer_errors(response: Response) -> Response {
    let is_json = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with(mime::APPLICATION_JSON.as_ref()));

    if is_json {
        return response;
    }

    match response.status() {
        StatusCode::REQUEST_TIMEOUT => Error::RequestTimeout.into_response(),
        StatusCode::PAYLOAD_TOO_LARGE => Error::PayloadTooLarge.into_response(),
        _ => response,
    }
}

fn cors_layer(allowed_origins: &[String]) -> CorsLayer {
    let allow_origin = if allowed_origins.iter().any(|origin| origin == "*") {
        AllowOrigin::any()
    } else {
        AllowOrigin::list(allowed_origins.iter().map(|origin| {
            origin
                .parse::<HeaderValue>()
                .unwrap_or_else(|_| panic!("Invalid CORS origin {origin}"))
        }))
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers(Any)
}
