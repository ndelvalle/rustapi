use reqwest;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::tests::setup::with_app;

#[test]
fn post_user_route() {
  #[derive(Debug, Serialize, Deserialize)]
  struct Body {
    name: String,
    email: String,
    password: String,
  }

  let body = Body {
    name: "Nahuel".to_owned(),
    email: "nahuel@gmail.com".to_owned(),
    password: "Password1".to_owned(),
  };

  with_app(async move {
    let client = reqwest::Client::new();
    let res = client
      .post("http://localhost:8088/users")
      .json(&body)
      .send()
      .await
      .unwrap();

    let status_code = res.status();
    // TODO: Assert body response
    let _body = res.json::<Json>().await.unwrap();

    // Status code:
    let actual = status_code;
    let expected = StatusCode::OK;
    assert_eq!(actual, expected);
  });
}
