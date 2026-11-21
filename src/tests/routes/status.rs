use assert_json_diff::assert_json_eq;
use reqwest::StatusCode;
use serde_json::json;
use serde_json::Value as Json;

use crate::tests::setup::use_app;

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn get_status_route() {
    use_app(async {
        let res = reqwest::get("http://localhost:8088/status").await.unwrap();
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
    });
}

#[test]
fn request_id_is_generated_or_propagated() {
    use_app(async {
        let client = reqwest::Client::new();

        // A request without an ID gets a generated one.
        let res = client
            .get("http://localhost:8088/status")
            .send()
            .await
            .unwrap();
        let generated = res.headers().get("X-Request-Id").unwrap().to_str().unwrap();
        assert!(uuid::Uuid::parse_str(generated).is_ok());

        // A request with an ID gets the same one back.
        let res = client
            .get("http://localhost:8088/status")
            .header("X-Request-Id", "my-request-id")
            .send()
            .await
            .unwrap();
        assert_eq!(res.headers().get("X-Request-Id").unwrap(), "my-request-id");
    });
}

#[test]
fn request_body_over_the_limit_is_rejected() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use_app(async {
        // Only the headers are sent, announcing a body over the limit. Uploading the body
        // would race with the server closing the connection after it answers.
        let request = "POST /users HTTP/1.1\r\n\
            Host: localhost\r\n\
            Content-Type: application/json\r\n\
            Content-Length: 2097152\r\n\
            \r\n";

        let mut stream = tokio::net::TcpStream::connect("localhost:8088")
            .await
            .unwrap();
        stream.write_all(request.as_bytes()).await.unwrap();

        let mut buffer = [0; 64];
        let read = stream.read(&mut buffer).await.unwrap();
        let response = String::from_utf8_lossy(&buffer[..read]);

        assert!(response.starts_with("HTTP/1.1 413"), "{response}");
    });
}
