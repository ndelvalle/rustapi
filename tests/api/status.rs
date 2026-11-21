use assert_json_diff::assert_json_eq;
use reqwest::StatusCode;
use serde_json::json;
use serde_json::Value as Json;
use sqlx::PgPool;

use crate::helpers::TestApp;

use pretty_assertions::assert_eq;

#[sqlx::test]
async fn get_status_route(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let res = reqwest::get(app.url("/status")).await.unwrap();
    let status_code = res.status();
    let body = res.json::<Json>().await.unwrap();

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
    let res = client.get(app.url("/status")).send().await.unwrap();
    let generated = res.headers().get("X-Request-Id").unwrap().to_str().unwrap();
    assert!(uuid::Uuid::parse_str(generated).is_ok());

    // A request with an ID gets the same one back.
    let res = client
        .get(app.url("/status"))
        .header("X-Request-Id", "my-request-id")
        .send()
        .await
        .unwrap();
    assert_eq!(res.headers().get("X-Request-Id").unwrap(), "my-request-id");
}

#[sqlx::test]
async fn request_body_over_the_limit_is_rejected(db: PgPool) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let app = TestApp::spawn(db).await;
    // Only the headers are sent, announcing a body over the limit. Uploading the body
    // would race with the server closing the connection after it answers.
    let request = "POST /users HTTP/1.1\r\n\
            Host: localhost\r\n\
            Content-Type: application/json\r\n\
            Content-Length: 2097152\r\n\
            \r\n";

    let mut stream = tokio::net::TcpStream::connect(&app.address).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();

    let mut buffer = [0; 64];
    let read = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..read]);

    assert!(response.starts_with("HTTP/1.1 413"), "{response}");
}
