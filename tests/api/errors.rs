use pretty_assertions::assert_eq;
use reqwest::{Method, StatusCode};
use serde_json::Value as Json;
use sqlx::PgPool;

use crate::helpers::TestApp;

// Asserts that the response is an error in the standard shape and returns its message.
async fn assert_error(response: reqwest::Response, status: StatusCode, code: u64) -> String {
    assert_eq!(response.status(), status);

    let content_type = response.headers().get("Content-Type").unwrap();
    assert_eq!(content_type, "application/json");

    let body = response.json::<Json>().await.unwrap();
    assert_eq!(body["code"], code, "{body}");

    body["message"].as_str().unwrap().to_owned()
}

#[sqlx::test]
async fn unreadable_bodies_are_rejected_with_the_standard_error(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let client = reqwest::Client::new();
    let post = |body: &'static str| {
        client
            .post(app.url("/users"))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
    };

    // JSON with a syntax error.
    let response = post(r#"{"name":"#).await.unwrap();
    let message = assert_error(response, StatusCode::BAD_REQUEST, 40009).await;
    assert!(
        message.contains("Failed to parse the request body as JSON"),
        "{message}"
    );

    // An empty body.
    let response = post("").await.unwrap();
    assert_error(response, StatusCode::BAD_REQUEST, 40009).await;

    // Valid JSON with a missing field.
    let response = post(r#"{"name":"Nahuel"}"#).await.unwrap();
    let message = assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, 40009).await;
    assert!(message.contains("missing field `email`"), "{message}");

    // Valid JSON with a field of the wrong type.
    let response = post(r#"{"name":1,"email":"nahuel@gmail.com","password":"Password1"}"#)
        .await
        .unwrap();
    let message = assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, 40009).await;
    assert!(message.contains("expected a string"), "{message}");

    // Valid JSON that is not an object.
    let response = post("[]").await.unwrap();
    assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, 40009).await;

    // A body that is not declared as JSON.
    let response = client
        .post(app.url("/users"))
        .body(r#"{"name":"Nahuel","email":"nahuel@gmail.com","password":"Password1"}"#)
        .send()
        .await
        .unwrap();
    let message = assert_error(response, StatusCode::UNSUPPORTED_MEDIA_TYPE, 40009).await;
    assert!(
        message.contains("Content-Type: application/json"),
        "{message}"
    );

    // None of them created a user.
    let users: i64 = sqlx::query_scalar(r#"select count(*) from "user""#)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(users, 0);
}

#[sqlx::test]
async fn every_route_with_a_body_rejects_unreadable_and_invalid_bodies(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let user = app.create_user("nico@test.com").await;
    let token = app.token(&user);
    let client = reqwest::Client::new();
    let cat_url = format!("/v1/cats/{}", uuid::Uuid::now_v7());

    // Each route with a body that parses but fails validation.
    let routes = [
        (
            Method::POST,
            "/users",
            r#"{"name":"","email":"nahuel@gmail.com","password":"Password1"}"#,
        ),
        (
            Method::POST,
            "/users/authenticate",
            r#"{"email":"","password":"Password1"}"#,
        ),
        (Method::POST, "/v1/cats", r#"{"name":""}"#),
        (Method::PUT, cat_url.as_str(), r#"{"name":""}"#),
    ];

    for (method, path, invalid_body) in routes {
        let request = |body: &'static str| {
            client
                .request(method.clone(), app.url(path))
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(body)
                .send()
        };

        let response = request("{").await.unwrap();
        assert_error(response, StatusCode::BAD_REQUEST, 40009).await;

        let response = request("{}").await.unwrap();
        assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, 40009).await;

        let response = request(invalid_body).await.unwrap();
        let message = assert_error(response, StatusCode::UNPROCESSABLE_ENTITY, 40008).await;
        assert_eq!(message, "Validation failed", "{method} {path}");
    }
}

#[sqlx::test]
async fn unknown_routes_are_answered_with_the_standard_error(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let client = reqwest::Client::new();

    for path in ["/nope", "/v1/nope", "/users/nope", "/v1/cats/1/nope", "/"] {
        let response = client.get(app.url(path)).send().await.unwrap();
        let message = assert_error(response, StatusCode::NOT_FOUND, 40003).await;
        assert_eq!(message, "Not found", "{path}");
    }
}

#[sqlx::test]
async fn unsupported_methods_are_answered_with_the_standard_error(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let user = app.create_user("nico@test.com").await;
    let token = app.token(&user);
    let client = reqwest::Client::new();

    let cases = [
        (Method::DELETE, "/users", "POST"),
        (Method::GET, "/users/authenticate", "POST"),
        (Method::POST, "/status", "GET,HEAD"),
        (Method::PATCH, "/v1/cats", "POST,GET,HEAD"),
    ];

    for (method, path, allowed) in cases {
        let response = client
            .request(method.clone(), app.url(path))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .unwrap();

        // The response still says which methods the route supports.
        let allow = response.headers().get("Allow").unwrap().to_str().unwrap();
        let mut allow = allow.split(',').collect::<Vec<_>>();
        let mut expected = allowed.split(',').collect::<Vec<_>>();
        allow.sort();
        expected.sort();
        assert_eq!(allow, expected, "{method} {path}");

        let message = assert_error(response, StatusCode::METHOD_NOT_ALLOWED, 40010).await;
        assert_eq!(message, "Method not allowed");
    }
}

#[sqlx::test]
async fn request_body_over_the_limit_is_rejected(db: PgPool) {
    let mut settings = TestApp::settings();
    settings.server.request_body_limit_bytes = 1024;
    let app = TestApp::spawn_with_settings(db, settings).await;

    // A body that is announced as larger than the limit is rejected before it is read. Only
    // the headers are sent, because uploading the body would race with the server closing
    // the connection after it answers.
    let request = "POST /users HTTP/1.1\r\n\
        Host: localhost\r\n\
        Content-Type: application/json\r\n\
        Content-Length: 2048\r\n\
        \r\n";
    let (status, body) = app.send_raw(request).await;
    let body = serde_json::from_str::<Json>(&body).unwrap();
    assert_eq!(status, 413);
    assert_eq!(body["code"], 40012);
    assert_eq!(body["message"], "Request body is too large");

    // A body of unannounced length is rejected once it exceeds the limit while being read.
    let chunk = "a".repeat(1500);
    let request = format!(
        "POST /users HTTP/1.1\r\n\
        Host: localhost\r\n\
        Content-Type: application/json\r\n\
        Transfer-Encoding: chunked\r\n\
        \r\n\
        {:x}\r\n{chunk}\r\n0\r\n\r\n",
        chunk.len()
    );
    let (status, body) = app.send_raw(&request).await;
    let body = serde_json::from_str::<Json>(&body).unwrap();
    assert_eq!(status, 413);
    assert_eq!(body["code"], 40012);

    // A body within the limit is accepted.
    let response = reqwest::Client::new()
        .post(app.url("/users"))
        .json(&serde_json::json!({
            "name": "Nahuel",
            "email": "nahuel@gmail.com",
            "password": "Password1",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[sqlx::test]
async fn slow_requests_time_out_with_the_standard_error(db: PgPool) {
    let mut settings = TestApp::settings();
    settings.server.request_timeout_seconds = 1;
    let app = TestApp::spawn_with_settings(db, settings).await;

    // The body is announced but never completed, so the handler waits for it until the
    // request times out.
    let request = "POST /users HTTP/1.1\r\n\
        Host: localhost\r\n\
        Content-Type: application/json\r\n\
        Content-Length: 100\r\n\
        \r\n\
        {\"name\":";
    let started_at = std::time::Instant::now();
    let (status, body) = app.send_raw(request).await;
    let elapsed = started_at.elapsed();

    let body = serde_json::from_str::<Json>(&body).unwrap();
    assert_eq!(status, 408);
    assert_eq!(body["code"], 40011);
    assert_eq!(body["message"], "Request timed out");
    assert!(elapsed.as_millis() >= 900, "Timed out after {elapsed:?}");

    // A request that completes in time is not affected.
    let response = reqwest::get(app.url("/status")).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test]
async fn errors_of_the_handlers_keep_their_own_body(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let client = reqwest::Client::new();

    // The layer that rewrites the timeout and body limit errors leaves other errors alone.
    let response = client.get(app.url("/v1/cats")).send().await.unwrap();
    let message = assert_error(response, StatusCode::UNAUTHORIZED, 40005).await;
    assert_eq!(message, "Invalid authentication credentials");

    // And it leaves successful responses alone.
    let response = client.get(app.url("/status")).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.json::<Json>().await.unwrap(),
        serde_json::json!({ "status": "ok" })
    );
}
