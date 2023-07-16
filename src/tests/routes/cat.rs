use reqwest;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::models::cat::Cat;
use crate::models::cat::PublicCat;
use crate::tests::setup::use_app;
use crate::tests::utils::create_user;
use crate::tests::utils::create_user_token;
use crate::utils::models::ModelExt;

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn post_cat_route() {
  #[derive(Debug, Serialize, Deserialize)]
  struct Body {
    name: String,
  }

  let body = Body {
    name: "Tigrin".to_owned(),
  };

  use_app(async move {
    let user = create_user("nico@test.com").await.unwrap();
    let token = create_user_token(user.clone()).await.unwrap();

    let client = reqwest::Client::new();
    let res = client
      .post("http://localhost:8088/v1/cats")
      .header("Authorization", format!("Bearer {}", token))
      .json(&body)
      .send()
      .await
      .unwrap();

    // Status code:
    let status_code = res.status();
    let actual = status_code;
    let expected = StatusCode::CREATED;
    assert_eq!(actual, expected);

    // Body:
    let body = res.json::<PublicCat>().await.unwrap();
    assert_eq!(body.name, "Tigrin");
    assert_eq!(body.user, user.id.unwrap(), "Cat should belong to user");
  });
}

#[test]
fn get_cats_route() {
  use_app(async move {
    let user = create_user("nico@test.com").await.unwrap();
    let token = create_user_token(user.clone()).await.unwrap();

    let tigrin = Cat::new(user.id.unwrap(), "Tigrin".to_owned());
    Cat::create(tigrin).await.unwrap();

    let cielito = Cat::new(user.id.unwrap(), "Cielito".to_owned());
    Cat::create(cielito).await.unwrap();

    let client = reqwest::Client::new();
    let res = client
      .get("http://localhost:8088/v1/cats")
      .header("Authorization", format!("Bearer {}", token))
      .send()
      .await
      .unwrap();

    // Status code:
    let status_code = res.status();
    let actual = status_code;
    let expected = StatusCode::OK;
    assert_eq!(actual, expected);

    // Response headers:
    let headers = res.headers();
    assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
    // Response pagination headers:
    assert_eq!(headers.get("X-Pagination-Count").unwrap(), "2");
    assert_eq!(headers.get("X-Pagination-Offset").unwrap(), "0");
    assert_eq!(headers.get("X-Pagination-Limit").unwrap(), "100");

    // Body:
    let body = res.json::<Vec<PublicCat>>().await.unwrap();
    assert_eq!(body.len(), 2, "Should return two cats");

    // First cat (Cielito):
    let cat = body.get(0).unwrap();
    assert_eq!(cat.name, "Cielito");
    assert_eq!(cat.user, user.id.unwrap());

    // Second cat (Tigrin):
    let cat = body.get(1).unwrap();
    assert_eq!(cat.name, "Tigrin");
    assert_eq!(cat.user, user.id.unwrap());
  });
}

#[test]
fn get_cat_by_id_route() {
  use_app(async move {
    let user = create_user("nico@test.com").await.unwrap();
    let token = create_user_token(user.clone()).await.unwrap();

    let cholin = Cat::new(user.id.unwrap(), "Cholin".to_owned());
    let cholin = Cat::create(cholin).await.unwrap();

    let client = reqwest::Client::new();
    let res = client
      .get(format!(
        "http://localhost:8088/v1/cats/{}",
        cholin.id.unwrap()
      ))
      .header("Authorization", format!("Bearer {}", token))
      .send()
      .await
      .unwrap();

    // Status code:
    let status_code = res.status();
    let actual = status_code;
    let expected = StatusCode::OK;
    assert_eq!(actual, expected);

    // Body:
    let body = res.json::<PublicCat>().await.unwrap();
    assert_eq!(body.name, "Cholin");
    assert_eq!(body.user, user.id.unwrap());
  });
}

#[test]
fn remove_cat_by_id_route() {
  use_app(async move {
    let user = create_user("nico@test.com").await.unwrap();
    let token = create_user_token(user.clone()).await.unwrap();

    let tigrin = Cat::new(user.id.unwrap(), "Tigrin".to_owned());
    let tigrin = Cat::create(tigrin).await.unwrap();

    let client = reqwest::Client::new();
    let res = client
      .delete(format!(
        "http://localhost:8088/v1/cats/{}",
        tigrin.id.unwrap()
      ))
      .header("Authorization", format!("Bearer {}", token))
      .send()
      .await
      .unwrap();

    // Status code:
    let status_code = res.status();
    let actual = status_code;
    let expected = StatusCode::NO_CONTENT;
    assert_eq!(actual, expected);

    // Cat from the database
    let cat = Cat::find_by_id(&tigrin.id.unwrap()).await.unwrap();
    assert!(cat.is_none(), "Cat should be removed from the database");
  });
}
