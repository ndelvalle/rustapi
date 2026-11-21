use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::helpers::TestApp;
use rustapi::models::cat::Cat;
use rustapi::routes::cat::CatResponse;

use pretty_assertions::assert_eq;

#[sqlx::test]
async fn post_cat_route(db: PgPool) {
    #[derive(Debug, Serialize, Deserialize)]
    struct Body {
        name: String,
    }

    let body = Body {
        name: "Tigrin".to_owned(),
    };

    let app = TestApp::spawn(db).await;
    let user = app.create_user("nico@test.com").await;
    let token = app.token(&user);

    let client = reqwest::Client::new();
    let res = client
        .post(app.url("/v1/cats"))
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
    let body = res.json::<CatResponse>().await.unwrap();
    assert_eq!(body.name, "Tigrin");
    assert_eq!(body.user, user.user_id, "Cat should belong to user");
}

#[sqlx::test]
async fn get_cats_route(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let user = app.create_user("nico@test.com").await;
    let token = app.token(&user);

    Cat::create(&app.db, user.user_id, "Tigrin").await.unwrap();

    Cat::create(&app.db, user.user_id, "Cielito").await.unwrap();

    let client = reqwest::Client::new();
    let res = client
        .get(app.url("/v1/cats"))
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
    let body = res.json::<Vec<CatResponse>>().await.unwrap();
    assert_eq!(body.len(), 2, "Should return two cats");

    // First cat (Cielito):
    let cat = body.first().unwrap();
    assert_eq!(cat.name, "Cielito");
    assert_eq!(cat.user, user.user_id);

    // Second cat (Tigrin):
    let cat = body.get(1).unwrap();
    assert_eq!(cat.name, "Tigrin");
    assert_eq!(cat.user, user.user_id);
}

#[sqlx::test]
async fn get_cat_by_id_route(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let user = app.create_user("nico@test.com").await;
    let token = app.token(&user);

    let cholin = Cat::create(&app.db, user.user_id, "Cholin").await.unwrap();

    let client = reqwest::Client::new();
    let res = client
        .get(app.url(&format!("/v1/cats/{}", cholin.cat_id)))
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
    let body = res.json::<CatResponse>().await.unwrap();
    assert_eq!(body.name, "Cholin");
    assert_eq!(body.user, user.user_id);
}

#[sqlx::test]
async fn remove_cat_by_id_route(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let user = app.create_user("nico@test.com").await;
    let token = app.token(&user);

    let tigrin = Cat::create(&app.db, user.user_id, "Tigrin").await.unwrap();

    let client = reqwest::Client::new();
    let res = client
        .delete(app.url(&format!("/v1/cats/{}", tigrin.cat_id)))
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
    let cat = Cat::find_by_id(&app.db, user.user_id, tigrin.cat_id)
        .await
        .unwrap();
    assert!(cat.is_none(), "Cat should be removed from the database");
}

#[sqlx::test]
async fn get_cats_route_pagination_limits(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let user = app.create_user("nico@test.com").await;
    let token = app.token(&user);
    let client = reqwest::Client::new();

    // A limit above the maximum is capped.
    let res = client
        .get(app.url("/v1/cats?limit=100000&offset=5"))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers().get("X-Pagination-Limit").unwrap(), "100");
    assert_eq!(res.headers().get("X-Pagination-Offset").unwrap(), "5");

    // An offset that is out of range is rejected.
    let res = client
        .get(app.url("/v1/cats?offset=99999999999"))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // An invalid limit is rejected.
    let res = client
        .get(app.url("/v1/cats?limit=abc"))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn put_cat_route(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let user = app.create_user("nico@test.com").await;
    let token = app.token(&user);

    let tigrin = Cat::create(&app.db, user.user_id, "Tigrin").await.unwrap();

    let client = reqwest::Client::new();
    let res = client
        .put(app.url(&format!("/v1/cats/{}", tigrin.cat_id)))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "name": "Cielito" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // Body:
    let body = res.json::<CatResponse>().await.unwrap();
    assert_eq!(body.id, tigrin.cat_id);
    assert_eq!(body.name, "Cielito");
    assert!(body.updated_at > tigrin.updated_at);

    // Cat from the database
    let cat = Cat::find_by_id(&app.db, user.user_id, tigrin.cat_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(cat.name, "Cielito");
}

#[sqlx::test]
async fn cat_routes_require_authentication(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let client = reqwest::Client::new();
    let id = uuid::Uuid::now_v7();
    let body = serde_json::json!({ "name": "Tigrin" });

    let requests = [
        client.get(app.url("/v1/cats")),
        client.post(app.url("/v1/cats")).json(&body),
        client.get(app.url(&format!("/v1/cats/{id}"))),
        client.put(app.url(&format!("/v1/cats/{id}"))).json(&body),
        client.delete(app.url(&format!("/v1/cats/{id}"))),
        client
            .get(app.url("/v1/cats"))
            .header("Authorization", "Bearer not-a-token"),
    ];

    for request in requests {
        let res = request.send().await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }
}

#[sqlx::test]
async fn cat_routes_hide_cats_from_other_users(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let owner = app.create_user("owner@test.com").await;
    let other = app.create_user("other@test.com").await;
    let token = app.token(&other);

    let tigrin = Cat::create(&app.db, owner.user_id, "Tigrin").await.unwrap();

    let client = reqwest::Client::new();
    let url = app.url(&format!("/v1/cats/{}", tigrin.cat_id));
    let requests = [
        client.get(&url),
        client
            .put(&url)
            .json(&serde_json::json!({ "name": "Stolen" })),
        client.delete(&url),
    ];

    for request in requests {
        let res = request
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    let res = client
        .get(app.url("/v1/cats"))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    let body = res.json::<Vec<CatResponse>>().await.unwrap();
    assert!(body.is_empty(), "Should not list cats from other users");

    // Cat from the database
    let cat = Cat::find_by_id(&app.db, owner.user_id, tigrin.cat_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(cat.name, "Tigrin", "Cat should be untouched");
}

#[sqlx::test]
async fn cat_routes_reject_invalid_input(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let user = app.create_user("nico@test.com").await;
    let token = app.token(&user);
    let client = reqwest::Client::new();

    // An ID that is not a UUID.
    let res = client
        .get(app.url("/v1/cats/not-an-id"))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = res.json::<serde_json::Value>().await.unwrap();
    assert_eq!(body["code"], 40001);

    // An empty name.
    let res = client
        .post(app.url("/v1/cats"))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "name": "" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = res.json::<serde_json::Value>().await.unwrap();
    assert_eq!(body["errors"]["name"][0], "must not be empty");

    // A cat that does not exist.
    let res = client
        .get(app.url(&format!("/v1/cats/{}", uuid::Uuid::now_v7())))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn cat_routes_reject_tokens_of_locked_and_removed_users(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let locked = app.create_user("locked@test.com").await;
    let locked_token = app.token(&locked);
    let removed = app.create_user("removed@test.com").await;
    let removed_token = app.token(&removed);

    let client = reqwest::Client::new();

    // Both tokens work before the users are locked and removed.
    for token in [&locked_token, &removed_token] {
        let res = client
            .get(app.url("/v1/cats"))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    sqlx::query(r#"update "user" set locked_at = now() where user_id = $1"#)
        .bind(locked.user_id)
        .execute(&app.db)
        .await
        .unwrap();
    sqlx::query(r#"delete from "user" where user_id = $1"#)
        .bind(removed.user_id)
        .execute(&app.db)
        .await
        .unwrap();

    let res = client
        .get(app.url("/v1/cats"))
        .header("Authorization", format!("Bearer {}", locked_token))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::LOCKED);

    let res = client
        .get(app.url("/v1/cats"))
        .header("Authorization", format!("Bearer {}", removed_token))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
