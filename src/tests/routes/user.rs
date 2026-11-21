use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::routes::user::AuthenticateResponse;
use crate::routes::user::UserResponse;
use crate::tests::setup::use_app;
use crate::tests::utils::create_user;

#[cfg(test)]
use pretty_assertions::assert_eq;

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

    use_app(async move {
        let client = reqwest::Client::new();
        let res = client
            .post("http://localhost:8088/users")
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
    });
}

#[test]
fn authenticate_user_route() {
    #[derive(Debug, Serialize, Deserialize)]
    struct RequestBody {
        email: String,
        password: String,
    }

    let request_body = RequestBody {
        email: "nahuel@gmail.com".to_owned(),
        password: "Password1".to_owned(),
    };

    use_app(async move {
        create_user("nahuel@gmail.com").await.unwrap();

        let client = reqwest::Client::new();
        let res = client
            .post("http://localhost:8088/users/authenticate")
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
    });
}

#[test]
fn authenticate_user_route_rejects_unknown_email_and_wrong_password_alike() {
    use_app(async move {
        create_user("nahuel@gmail.com").await.unwrap();

        let client = reqwest::Client::new();
        let attempts = [
            ("unknown@gmail.com", "Password1"),
            ("nahuel@gmail.com", "WrongPassword"),
        ];

        for (email, password) in attempts {
            let res = client
                .post("http://localhost:8088/users/authenticate")
                .json(&serde_json::json!({ "email": email, "password": password }))
                .send()
                .await
                .unwrap();

            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

            let body = res.json::<serde_json::Value>().await.unwrap();
            assert_eq!(body["code"], 40004);
        }
    });
}

#[test]
fn post_user_route_rejects_duplicate_email() {
    use_app(async move {
        create_user("nahuel@gmail.com").await.unwrap();

        let client = reqwest::Client::new();
        let res = client
            .post("http://localhost:8088/users")
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
    });
}

#[test]
fn post_user_route_rejects_invalid_input() {
    use_app(async move {
        let client = reqwest::Client::new();
        let bodies = [
            serde_json::json!({ "name": "Nahuel", "email": "not-an-email", "password": "Password1" }),
            serde_json::json!({ "name": "", "email": "nahuel@gmail.com", "password": "Password1" }),
            serde_json::json!({ "name": "Nahuel", "email": "nahuel@gmail.com", "password": "short" }),
        ];

        for body in bodies {
            let res = client
                .post("http://localhost:8088/users")
                .json(&body)
                .send()
                .await
                .unwrap();

            assert_eq!(res.status(), StatusCode::BAD_REQUEST, "{body}");
        }
    });
}

#[test]
fn authenticate_user_route_rejects_locked_user() {
    use_app(async move {
        let user = create_user("nahuel@gmail.com").await.unwrap();

        sqlx::query(r#"update "user" set locked_at = now() where id = $1"#)
            .bind(user.id)
            .execute(crate::database::connection().await)
            .await
            .unwrap();

        let client = reqwest::Client::new();
        let res = client
            .post("http://localhost:8088/users/authenticate")
            .json(&serde_json::json!({ "email": "nahuel@gmail.com", "password": "Password1" }))
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::LOCKED);
    });
}

#[test]
fn authenticated_token_grants_access() {
    use_app(async move {
        create_user("nahuel@gmail.com").await.unwrap();

        let client = reqwest::Client::new();
        let res = client
            .post("http://localhost:8088/users/authenticate")
            .json(&serde_json::json!({ "email": "nahuel@gmail.com", "password": "Password1" }))
            .send()
            .await
            .unwrap();
        let body = res.json::<AuthenticateResponse>().await.unwrap();

        let res = client
            .get("http://localhost:8088/v1/cats")
            .header("Authorization", format!("Bearer {}", body.access_token))
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    });
}
