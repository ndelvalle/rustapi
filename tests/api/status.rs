use assert_json_diff::assert_json_eq;
use reqwest::StatusCode;
use serde_json::Value as Json;
use serde_json::json;
use sqlx::PgPool;

use crate::helpers::TestApp;

use pretty_assertions::assert_eq;

#[sqlx::test]
async fn get_status_route(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let response = reqwest::get(app.url("/status")).await.unwrap();
    let status_code = response.status();
    let body = response.json::<Json>().await.unwrap();

    // Status code:
    let actual = status_code;
    let expected = StatusCode::OK;
    assert_eq!(actual, expected);

    // Body:
    let actual = body;
    let expected = json!({ "status": "ok" });
    assert_json_eq!(actual, expected);
}

#[sqlx::test]
async fn request_id_is_generated_or_propagated(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let client = reqwest::Client::new();

    // A request without an ID gets a generated one.
    let response = client.get(app.url("/status")).send().await.unwrap();
    let generated = response
        .headers()
        .get("X-Request-Id")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(uuid::Uuid::parse_str(generated).is_ok());

    // A request with an ID gets the same one back.
    let response = client
        .get(app.url("/status"))
        .header("X-Request-Id", "my-request-id")
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.headers().get("X-Request-Id").unwrap(),
        "my-request-id"
    );
}
