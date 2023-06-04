use assert_json_diff::assert_json_eq;
use reqwest;
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
