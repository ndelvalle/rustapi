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
    use_app(async {
        let body = format!(r#"{{"name":"{}"}}"#, "a".repeat(2 * 1024 * 1024));

        let res = reqwest::Client::new()
            .post("http://localhost:8088/users")
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
    });
}
