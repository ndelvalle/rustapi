use pretty_assertions::assert_eq;
use reqwest::{Method, StatusCode};
use sqlx::PgPool;

use crate::helpers::TestApp;

#[sqlx::test]
async fn cors_allows_only_the_configured_origins(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let client = reqwest::Client::new();

    for origin in ["https://app.example.com", "https://admin.example.com"] {
        let response = client
            .get(app.url("/status"))
            .header("Origin", origin)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("Access-Control-Allow-Origin")
                .unwrap(),
            origin
        );
    }

    // The request of another origin is answered, but without the header that lets the
    // browser hand the response to the page.
    let response = client
        .get(app.url("/status"))
        .header("Origin", "https://evil.example.com")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response
            .headers()
            .get("Access-Control-Allow-Origin")
            .is_none()
    );
}

#[sqlx::test]
async fn cors_answers_preflight_requests(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let client = reqwest::Client::new();

    let preflight = |origin: &'static str| {
        client
            .request(Method::OPTIONS, app.url("/v1/cats"))
            .header("Origin", origin)
            .header("Access-Control-Request-Method", "POST")
            .header(
                "Access-Control-Request-Headers",
                "authorization,content-type",
            )
            .send()
    };

    // The preflight needs no token, because browsers send it without credentials.
    let response = preflight("https://app.example.com").await.unwrap();
    let headers = response.headers();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        headers.get("Access-Control-Allow-Origin").unwrap(),
        "https://app.example.com"
    );
    assert!(headers.get("Access-Control-Allow-Methods").is_some());
    assert!(headers.get("Access-Control-Allow-Headers").is_some());

    let response = preflight("https://evil.example.com").await.unwrap();
    assert!(
        response
            .headers()
            .get("Access-Control-Allow-Origin")
            .is_none()
    );
}

#[sqlx::test]
async fn cors_allows_any_origin_with_a_wildcard(db: PgPool) {
    let mut settings = TestApp::settings();
    settings.cors.allowed_origins = vec!["*".to_owned()];
    let app = TestApp::spawn_with_settings(db, settings).await;

    let response = reqwest::Client::new()
        .get(app.url("/status"))
        .header("Origin", "https://anything.example.com")
        .send()
        .await
        .unwrap();

    assert_eq!(
        response
            .headers()
            .get("Access-Control-Allow-Origin")
            .unwrap(),
        "*"
    );
}

#[sqlx::test]
async fn cors_allows_no_origin_with_an_empty_list(db: PgPool) {
    let mut settings = TestApp::settings();
    settings.cors.allowed_origins = Vec::new();
    let app = TestApp::spawn_with_settings(db, settings).await;

    let response = reqwest::Client::new()
        .get(app.url("/status"))
        .header("Origin", "https://app.example.com")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response
            .headers()
            .get("Access-Control-Allow-Origin")
            .is_none()
    );
}

#[sqlx::test]
async fn cors_exposes_the_pagination_headers(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let user = app.create_user("nico@test.com").await;
    let token = app.token(&user);

    let response = reqwest::Client::new()
        .get(app.url("/v1/cats"))
        .header("Origin", "https://app.example.com")
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("Access-Control-Expose-Headers")
            .unwrap(),
        "*"
    );
}
