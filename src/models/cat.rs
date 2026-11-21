use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::common::date::Date;
use crate::errors::Error;

#[derive(Debug, Clone, FromRow)]
pub struct Cat {
    pub cat_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub created_at: Date,
    pub updated_at: Date,
}

impl Cat {
    pub async fn create(db: &PgPool, user_id: Uuid, name: &str) -> Result<Self, Error> {
        let cat = sqlx::query_as(
            "insert into cat (user_id, name)
             values ($1, $2)
             returning *",
        )
        .bind(user_id)
        .bind(name)
        .fetch_one(db)
        .await?;

        Ok(cat)
    }

    pub async fn find_by_id(
        db: &PgPool,
        user_id: Uuid,
        cat_id: Uuid,
    ) -> Result<Option<Self>, Error> {
        let cat = sqlx::query_as("select * from cat where user_id = $1 and cat_id = $2")
            .bind(user_id)
            .bind(cat_id)
            .fetch_optional(db)
            .await?;

        Ok(cat)
    }

    pub async fn find_by_user(
        db: &PgPool,
        user_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Self>, Error> {
        let cats = sqlx::query_as(
            "select * from cat where user_id = $1
             order by created_at desc, cat_id desc
             limit $2 offset $3",
        )
        .bind(user_id)
        .bind(i64::from(limit))
        .bind(i64::from(offset))
        .fetch_all(db)
        .await?;

        Ok(cats)
    }

    pub async fn count_by_user(db: &PgPool, user_id: Uuid) -> Result<u64, Error> {
        let count: i64 = sqlx::query_scalar("select count(*) from cat where user_id = $1")
            .bind(user_id)
            .fetch_one(db)
            .await?;

        Ok(count as u64)
    }

    pub async fn update_name(
        db: &PgPool,
        user_id: Uuid,
        cat_id: Uuid,
        name: &str,
    ) -> Result<Option<Self>, Error> {
        let cat = sqlx::query_as(
            "update cat set name = $3
             where user_id = $1 and cat_id = $2
             returning *",
        )
        .bind(user_id)
        .bind(cat_id)
        .bind(name)
        .fetch_optional(db)
        .await?;

        Ok(cat)
    }

    pub async fn delete(db: &PgPool, user_id: Uuid, cat_id: Uuid) -> Result<bool, Error> {
        let result = sqlx::query("delete from cat where user_id = $1 and cat_id = $2")
            .bind(user_id)
            .bind(cat_id)
            .execute(db)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
