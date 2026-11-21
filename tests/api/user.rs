use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::helpers::TestApp;
use rustapi::routes::user::AuthenticateResponse;
use rustapi::routes::user::UserResponse;

use pretty_assertions::assert_eq;

#[sqlx::test]
async fn post_user_route(db: PgPool) {
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

    let app = TestApp::spawn(db).await;
    let client = reqwest::Client::new();
    let res = client
        .post(app.url("/users"))
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
    let body = res.json::<UserResponse>().await.unwrap();
    assert_eq!(body.name, "Nahuel");
    assert_eq!(body.email, "nahuel@gmail.com");
}

#[sqlx::test]
async fn authenticate_user_route(db: PgPool) {
    #[derive(Debug, Serialize, Deserialize)]
    struct RequestBody {
        email: String,
        password: String,
    }

    let request_body = RequestBody {
        email: "nahuel@gmail.com".to_owned(),
        password: "Password1".to_owned(),
    };

    let app = TestApp::spawn(db).await;
    app.create_user("nahuel@gmail.com").await;

    let client = reqwest::Client::new();
    let res = client
        .post(app.url("/users/authenticate"))
        .json(&request_body)
        .send()
        .await
        .unwrap();

    // Status code:
    let status_code = res.status();
    let actual = status_code;
    let expected = StatusCode::OK;
    assert_eq!(actual, expected);

    // Body:
    let body = res.json::<AuthenticateResponse>().await.unwrap();
    assert_eq!(body.user.email, "nahuel@gmail.com");
}

#[sqlx::test]
async fn authenticate_user_route_rejects_unknown_email_and_wrong_password_alike(db: PgPool) {
    let app = TestApp::spawn(db).await;
    app.create_user("nahuel@gmail.com").await;

    let client = reqwest::Client::new();
    let attempts = [
        ("unknown@gmail.com", "Password1"),
        ("nahuel@gmail.com", "WrongPassword"),
    ];

    for (email, password) in attempts {
        let res = client
            .post(app.url("/users/authenticate"))
            .json(&serde_json::json!({ "email": email, "password": password }))
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        let body = res.json::<serde_json::Value>().await.unwrap();
        assert_eq!(body["code"], 40004);
    }
}

#[sqlx::test]
async fn post_user_route_rejects_duplicate_email(db: PgPool) {
    let app = TestApp::spawn(db).await;
    app.create_user("nahuel@gmail.com").await;

    let client = reqwest::Client::new();
    let res = client
        .post(app.url("/users"))
        .json(&serde_json::json!({
            "name": "Nahuel",
            "email": "nahuel@gmail.com",
            "password": "Password1",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CONFLICT);

    let body = res.json::<serde_json::Value>().await.unwrap();
    assert_eq!(body["code"], 40007);
    assert_eq!(body["errors"]["email"][0], "is already in use");
}

#[sqlx::test]
async fn emails_are_case_insensitive(db: PgPool) {
    let app = TestApp::spawn(db).await;
    app.create_user("nahuel@gmail.com").await;

    let client = reqwest::Client::new();

    // The same email in a different case is a duplicate.
    let res = client
        .post(app.url("/users"))
        .json(&serde_json::json!({
            "name": "Nahuel",
            "email": "Nahuel@Gmail.com",
            "password": "Password1",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CONFLICT);

    // The user can authenticate with the email in a different case.
    let res = client
        .post(app.url("/users/authenticate"))
        .json(&serde_json::json!({ "email": "NAHUEL@GMAIL.COM", "password": "Password1" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.json::<AuthenticateResponse>().await.unwrap();
    assert_eq!(body.user.email, "nahuel@gmail.com");
}

#[sqlx::test]
async fn access_token_only_carries_the_user_id(db: PgPool) {
    use base64::Engine;

    let app = TestApp::spawn(db).await;
    let user = app.create_user("nahuel@gmail.com").await;

    let res = reqwest::Client::new()
        .post(app.url("/users/authenticate"))
        .json(&serde_json::json!({ "email": "nahuel@gmail.com", "password": "Password1" }))
        .send()
        .await
        .unwrap();
    let body = res.json::<AuthenticateResponse>().await.unwrap();

    let payload = body.access_token.split('.').nth(1).unwrap();
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .unwrap();
    let claims = serde_json::from_slice::<serde_json::Value>(&payload).unwrap();
    let mut keys = claims.as_object().unwrap().keys().collect::<Vec<_>>();
    keys.sort();

    assert_eq!(keys, ["exp", "iat", "sub"]);
    assert_eq!(claims["sub"], user.user_id.to_string());
}

#[sqlx::test]
async fn post_user_route_rejects_invalid_input(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let client = reqwest::Client::new();
    let cases = [
        (
            serde_json::json!({ "name": "Nahuel", "email": "not-an-email", "password": "Password1" }),
            "email",
            "must be a valid email address",
        ),
        (
            serde_json::json!({ "name": "", "email": "nahuel@gmail.com", "password": "Password1" }),
            "name",
            "must not be empty",
        ),
        (
            serde_json::json!({ "name": "Nahuel", "email": "nahuel@gmail.com", "password": "short" }),
            "password",
            "must have between 8 and 128 characters",
        ),
    ];

    for (body, field, message) in cases {
        let res = client
            .post(app.url("/users"))
            .json(&body)
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY, "{body}");

        let errors = res.json::<serde_json::Value>().await.unwrap();
        assert_eq!(errors["code"], 40008);
        assert_eq!(errors["errors"][field][0], message);
        assert_eq!(errors["errors"].as_object().unwrap().len(), 1);
    }

    // Every invalid field is reported at once.
    let res = client
        .post(app.url("/users"))
        .json(&serde_json::json!({ "name": "", "email": "not-an-email", "password": "short" }))
        .send()
        .await
        .unwrap();
    let errors = res.json::<serde_json::Value>().await.unwrap();
    assert_eq!(errors["errors"].as_object().unwrap().len(), 3);
}

#[sqlx::test]
async fn authenticate_user_route_rejects_locked_user(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let user = app.create_user("nahuel@gmail.com").await;

    sqlx::query(r#"update "user" set locked_at = now() where user_id = $1"#)
        .bind(user.user_id)
        .execute(&app.db)
        .await
        .unwrap();

    let client = reqwest::Client::new();
    let res = client
        .post(app.url("/users/authenticate"))
        .json(&serde_json::json!({ "email": "nahuel@gmail.com", "password": "Password1" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::LOCKED);
}

#[sqlx::test]
async fn authenticated_token_grants_access(db: PgPool) {
    let app = TestApp::spawn(db).await;
    app.create_user("nahuel@gmail.com").await;

    let client = reqwest::Client::new();
    let res = client
        .post(app.url("/users/authenticate"))
        .json(&serde_json::json!({ "email": "nahuel@gmail.com", "password": "Password1" }))
        .send()
        .await
        .unwrap();
    let body = res.json::<AuthenticateResponse>().await.unwrap();

    let res = client
        .get(app.url("/v1/cats"))
        .header("Authorization", format!("Bearer {}", body.access_token))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

#[sqlx::test]
async fn passwords_are_hashed_with_argon2_and_not_truncated(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let client = reqwest::Client::new();
    let password = "a".repeat(100);

    let res = client
        .post(app.url("/users"))
        .json(&serde_json::json!({
            "name": "Nahuel",
            "email": "nahuel@gmail.com",
            "password": password,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // The password is stored as an Argon2id hash.
    let hash: String = sqlx::query_scalar(r#"select password from "user""#)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert!(hash.starts_with("$argon2id$"), "{hash}");

    // The whole password counts, so one that only differs at the end is rejected.
    let almost = format!("{}b", "a".repeat(99));
    for (password, expected) in [
        (password, StatusCode::OK),
        (almost, StatusCode::UNAUTHORIZED),
    ] {
        let res = client
            .post(app.url("/users/authenticate"))
            .json(&serde_json::json!({ "email": "nahuel@gmail.com", "password": password }))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), expected);
    }
}
