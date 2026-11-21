use chrono::{Duration, Utc};
use pretty_assertions::assert_eq;
use rustapi::models::cat::Cat;
use sqlx::PgPool;

use crate::helpers::TestApp;

#[sqlx::test]
async fn database_generates_ids_and_timestamps(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let before = Utc::now() - Duration::seconds(5);

    let user = app.create_user("nico@test.com").await;
    let cat = Cat::create(&app.db, user.user_id, "Tigrin").await.unwrap();

    let after = Utc::now() + Duration::seconds(5);

    // IDs are UUIDv7.
    assert_eq!(user.user_id.get_version_num(), 7);
    assert_eq!(cat.cat_id.get_version_num(), 7);

    // Timestamps are set to the time of the insert.
    for created_at in [user.created_at, cat.created_at] {
        assert!(created_at > before && created_at < after, "{created_at}");
    }
    assert_eq!(user.updated_at, user.created_at);
    assert_eq!(cat.updated_at, cat.created_at);
    assert!(user.locked_at.is_none());
}

#[sqlx::test]
async fn ids_increase_with_time(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let user = app.create_user("nico@test.com").await;

    let mut ids = Vec::new();
    for name in ["Tigrin", "Cielito", "Cholin", "Mishi", "Garfield"] {
        let cat = Cat::create(&app.db, user.user_id, name).await.unwrap();
        ids.push(cat.cat_id);
    }

    // The IDs sort in creation order, both as UUIDs and as text.
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted);

    let texts = ids.iter().map(|id| id.to_string()).collect::<Vec<_>>();
    let mut sorted_texts = texts.clone();
    sorted_texts.sort();
    assert_eq!(texts, sorted_texts);

    // The database sorts them the same way.
    let from_database: Vec<uuid::Uuid> =
        sqlx::query_scalar("select cat_id from cat order by cat_id")
            .fetch_all(&app.db)
            .await
            .unwrap();
    assert_eq!(ids, from_database);
}

#[sqlx::test]
async fn database_sets_updated_at_on_update(db: PgPool) {
    let app = TestApp::spawn(db).await;
    let user = app.create_user("nico@test.com").await;
    let cat = Cat::create(&app.db, user.user_id, "Tigrin").await.unwrap();

    // An update that changes the row moves updated_at and leaves created_at alone.
    let updated = Cat::update_name(&app.db, user.user_id, cat.cat_id, "Cielito")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.name, "Cielito");
    assert_eq!(updated.created_at, cat.created_at);
    assert!(updated.updated_at > cat.updated_at);

    // An update that changes nothing leaves updated_at alone.
    let unchanged = Cat::update_name(&app.db, user.user_id, cat.cat_id, "Cielito")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(unchanged.updated_at, updated.updated_at);

    // The trigger also covers updates that do not go through the model.
    sqlx::query(r#"update "user" set locked_at = now() where user_id = $1"#)
        .bind(user.user_id)
        .execute(&app.db)
        .await
        .unwrap();
    let locked_user: (chrono::DateTime<Utc>,) =
        sqlx::query_as(r#"select updated_at from "user" where user_id = $1"#)
            .bind(user.user_id)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert!(locked_user.0 > user.updated_at);
}
