use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use sqlx::PgPool;

use crate::helpers::{Logs, TestApp};

#[sqlx::test]
async fn request_logs_carry_the_request_id(db: PgPool) {
    let (logs, _guard) = Logs::capture("info");
    let app = TestApp::spawn(db).await;
    let client = reqwest::Client::new();

    // The client sends its own request ID.
    let response = client
        .get(app.url("/status"))
        .header("X-Request-Id", "client-request-id")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // The server generates one and returns it to the client.
    let response = client.get(app.url("/v1/cats")).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let generated = response.headers()["X-Request-Id"]
        .to_str()
        .unwrap()
        .to_owned();

    let lines = logs.lines();
    assert_eq!(lines.len(), 4, "{lines:#?}");

    for (request_id, path) in [("client-request-id", "/status"), (&generated, "/v1/cats")] {
        let context = format!("method=GET uri={path} request_id={request_id}");
        let request_lines = lines
            .iter()
            .filter(|line| line.contains(&context))
            .collect::<Vec<_>>();

        assert_eq!(request_lines.len(), 2, "{lines:#?}");
        assert!(request_lines[0].contains("started processing request"));
        assert!(request_lines[1].contains("finished processing request"));
    }
}

#[sqlx::test]
async fn handler_logs_carry_the_request_id(db: PgPool) {
    let (logs, _guard) = Logs::capture("debug");
    let app = TestApp::spawn(db).await;

    let response = reqwest::Client::new()
        .get(app.url("/status"))
        .header("X-Request-Id", "handler-request-id")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let lines = logs.lines();
    assert!(
        lines.iter().any(|line| line.contains("Returning status")),
        "{lines:#?}"
    );
    assert!(
        lines
            .iter()
            .all(|line| line.contains("request_id=handler-request-id")),
        "{lines:#?}"
    );
}

#[sqlx::test]
async fn error_logs_carry_the_request_id_when_only_errors_are_logged(db: PgPool) {
    let (logs, _guard) = Logs::capture("error");
    let app = TestApp::spawn(db).await;

    // Closing the pool makes the status route fail to reach the database.
    app.db.close().await;

    let response = reqwest::Client::new()
        .get(app.url("/status"))
        .header("X-Request-Id", "failing-request-id")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let lines = logs.lines();
    assert_eq!(lines.len(), 2, "{lines:#?}");
    assert!(lines[0].contains("ERROR"), "{lines:#?}");
    assert!(lines[0].contains("Database is unreachable"), "{lines:#?}");
    assert!(lines[1].contains("response failed"), "{lines:#?}");
    assert!(
        lines
            .iter()
            .all(|line| line.contains("request_id=failing-request-id")),
        "{lines:#?}"
    );
}
