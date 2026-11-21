use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::models::cat::Cat;
use crate::routes::cat::CatResponse;
use crate::tests::setup::use_app;
use crate::tests::utils::create_user;
use crate::tests::utils::create_user_token;

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
        let body = res.json::<CatResponse>().await.unwrap();
        assert_eq!(body.name, "Tigrin");
        assert_eq!(body.user, user.id, "Cat should belong to user");
    });
}

#[test]
fn get_cats_route() {
    use_app(async move {
        let user = create_user("nico@test.com").await.unwrap();
        let token = create_user_token(user.clone()).await.unwrap();

        let tigrin = Cat::new(user.id, "Tigrin".to_owned());
        Cat::create(tigrin).await.unwrap();

        let cielito = Cat::new(user.id, "Cielito".to_owned());
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
        let body = res.json::<Vec<CatResponse>>().await.unwrap();
        assert_eq!(body.len(), 2, "Should return two cats");

        // First cat (Cielito):
        let cat = body.first().unwrap();
        assert_eq!(cat.name, "Cielito");
        assert_eq!(cat.user, user.id);

        // Second cat (Tigrin):
        let cat = body.get(1).unwrap();
        assert_eq!(cat.name, "Tigrin");
        assert_eq!(cat.user, user.id);
    });
}

#[test]
fn get_cat_by_id_route() {
    use_app(async move {
        let user = create_user("nico@test.com").await.unwrap();
        let token = create_user_token(user.clone()).await.unwrap();

        let cholin = Cat::new(user.id, "Cholin".to_owned());
        let cholin = Cat::create(cholin).await.unwrap();

        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://localhost:8088/v1/cats/{}", cholin.id))
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
        assert_eq!(body.user, user.id);
    });
}

#[test]
fn remove_cat_by_id_route() {
    use_app(async move {
        let user = create_user("nico@test.com").await.unwrap();
        let token = create_user_token(user.clone()).await.unwrap();

        let tigrin = Cat::new(user.id, "Tigrin".to_owned());
        let tigrin = Cat::create(tigrin).await.unwrap();

        let client = reqwest::Client::new();
        let res = client
            .delete(format!("http://localhost:8088/v1/cats/{}", tigrin.id))
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
        let cat = Cat::find_by_id(tigrin.id, user.id).await.unwrap();
        assert!(cat.is_none(), "Cat should be removed from the database");
    });
}

#[test]
fn get_cats_route_pagination_limits() {
    use_app(async move {
        let user = create_user("nico@test.com").await.unwrap();
        let token = create_user_token(user.clone()).await.unwrap();
        let client = reqwest::Client::new();

        // A limit above the maximum is capped.
        let res = client
            .get("http://localhost:8088/v1/cats?limit=100000&offset=5")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get("X-Pagination-Limit").unwrap(), "100");
        assert_eq!(res.headers().get("X-Pagination-Offset").unwrap(), "5");

        // An invalid limit is rejected.
        let res = client
            .get("http://localhost:8088/v1/cats?limit=abc")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    });
}

#[test]
fn put_cat_route() {
    use_app(async move {
        let user = create_user("nico@test.com").await.unwrap();
        let token = create_user_token(user.clone()).await.unwrap();

        let tigrin = Cat::new(user.id, "Tigrin".to_owned());
        let tigrin = Cat::create(tigrin).await.unwrap();

        let client = reqwest::Client::new();
        let res = client
            .put(format!("http://localhost:8088/v1/cats/{}", tigrin.id))
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({ "name": "Cielito" }))
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        // Body:
        let body = res.json::<CatResponse>().await.unwrap();
        assert_eq!(body.id, tigrin.id);
        assert_eq!(body.name, "Cielito");
        assert!(body.updated_at > tigrin.updated_at);

        // Cat from the database
        let cat = Cat::find_by_id(tigrin.id, user.id).await.unwrap().unwrap();
        assert_eq!(cat.name, "Cielito");
    });
}

#[test]
fn cat_routes_require_authentication() {
    use_app(async move {
        let client = reqwest::Client::new();
        let id = uuid::Uuid::now_v7();
        let body = serde_json::json!({ "name": "Tigrin" });

        let requests = [
            client.get("http://localhost:8088/v1/cats"),
            client.post("http://localhost:8088/v1/cats").json(&body),
            client.get(format!("http://localhost:8088/v1/cats/{id}")),
            client
                .put(format!("http://localhost:8088/v1/cats/{id}"))
                .json(&body),
            client.delete(format!("http://localhost:8088/v1/cats/{id}")),
            client
                .get("http://localhost:8088/v1/cats")
                .header("Authorization", "Bearer not-a-token"),
        ];

        for request in requests {
            let res = request.send().await.unwrap();
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        }
    });
}

#[test]
fn cat_routes_hide_cats_from_other_users() {
    use_app(async move {
        let owner = create_user("owner@test.com").await.unwrap();
        let other = create_user("other@test.com").await.unwrap();
        let token = create_user_token(other.clone()).await.unwrap();

        let tigrin = Cat::new(owner.id, "Tigrin".to_owned());
        let tigrin = Cat::create(tigrin).await.unwrap();

        let client = reqwest::Client::new();
        let url = format!("http://localhost:8088/v1/cats/{}", tigrin.id);
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
            .get("http://localhost:8088/v1/cats")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();
        let body = res.json::<Vec<CatResponse>>().await.unwrap();
        assert!(body.is_empty(), "Should not list cats from other users");

        // Cat from the database
        let cat = Cat::find_by_id(tigrin.id, owner.id).await.unwrap().unwrap();
        assert_eq!(cat.name, "Tigrin", "Cat should be untouched");
    });
}

#[test]
fn cat_routes_reject_invalid_input() {
    use_app(async move {
        let user = create_user("nico@test.com").await.unwrap();
        let token = create_user_token(user.clone()).await.unwrap();
        let client = reqwest::Client::new();

        // An ID that is not a UUID.
        let res = client
            .get("http://localhost:8088/v1/cats/not-an-id")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let body = res.json::<serde_json::Value>().await.unwrap();
        assert_eq!(body["code"], 40001);

        // An empty name.
        let res = client
            .post("http://localhost:8088/v1/cats")
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({ "name": "" }))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        // A cat that does not exist.
        let res = client
            .get(format!(
                "http://localhost:8088/v1/cats/{}",
                uuid::Uuid::now_v7()
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn cat_routes_reject_tokens_of_locked_and_removed_users() {
    use_app(async move {
        let locked = create_user("locked@test.com").await.unwrap();
        let locked_token = create_user_token(locked.clone()).await.unwrap();
        let removed = create_user("removed@test.com").await.unwrap();
        let removed_token = create_user_token(removed.clone()).await.unwrap();

        let client = reqwest::Client::new();

        // Both tokens work before the users are locked and removed.
        for token in [&locked_token, &removed_token] {
            let res = client
                .get("http://localhost:8088/v1/cats")
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        let connection = crate::database::connection().await;
        sqlx::query(r#"update "user" set locked_at = now() where id = $1"#)
            .bind(locked.id)
            .execute(connection)
            .await
            .unwrap();
        sqlx::query(r#"delete from "user" where id = $1"#)
            .bind(removed.id)
            .execute(connection)
            .await
            .unwrap();

        let res = client
            .get("http://localhost:8088/v1/cats")
            .header("Authorization", format!("Bearer {}", locked_token))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::LOCKED);

        let res = client
            .get("http://localhost:8088/v1/cats")
            .header("Authorization", format!("Bearer {}", removed_token))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    });
}
