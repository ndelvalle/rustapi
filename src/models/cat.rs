use sqlx::FromRow;
use uuid::Uuid;

use crate::database;
use crate::errors::Error;
use crate::utils::date;
use crate::utils::date::Date;

#[derive(Debug, Clone, FromRow)]
pub struct Cat {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub created_at: Date,
    pub updated_at: Date,
}

impl Cat {
    pub fn new(user_id: Uuid, name: String) -> Self {
        let now = date::now();
        Self {
            id: Uuid::now_v7(),
            user_id,
            name,
            created_at: now,
            updated_at: now,
        }
    }

    pub async fn create(cat: Self) -> Result<Self, Error> {
        let cat = sqlx::query_as(
            "insert into cat (id, user_id, name, created_at, updated_at)
             values ($1, $2, $3, $4, $5)
             returning *",
        )
        .bind(cat.id)
        .bind(cat.user_id)
        .bind(cat.name)
        .bind(cat.created_at)
        .bind(cat.updated_at)
        .fetch_one(database::connection().await)
        .await?;

        Ok(cat)
    }

    pub async fn find_by_id(user_id: Uuid, id: Uuid) -> Result<Option<Self>, Error> {
        let cat = sqlx::query_as("select * from cat where user_id = $1 and id = $2")
            .bind(user_id)
            .bind(id)
            .fetch_optional(database::connection().await)
            .await?;

        Ok(cat)
    }

    pub async fn find_by_user(user_id: Uuid, limit: u32, offset: u32) -> Result<Vec<Self>, Error> {
        let cats = sqlx::query_as(
            "select * from cat where user_id = $1
             order by created_at desc, id desc
             limit $2 offset $3",
        )
        .bind(user_id)
        .bind(i64::from(limit))
        .bind(i64::from(offset))
        .fetch_all(database::connection().await)
        .await?;

        Ok(cats)
    }

    pub async fn count_by_user(user_id: Uuid) -> Result<u64, Error> {
        let count: i64 = sqlx::query_scalar("select count(*) from cat where user_id = $1")
            .bind(user_id)
            .fetch_one(database::connection().await)
            .await?;

        Ok(count as u64)
    }

    pub async fn update_name(user_id: Uuid, id: Uuid, name: &str) -> Result<Option<Self>, Error> {
        let cat = sqlx::query_as(
            "update cat set name = $3, updated_at = $4
             where user_id = $1 and id = $2
             returning *",
        )
        .bind(user_id)
        .bind(id)
        .bind(name)
        .bind(date::now())
        .fetch_optional(database::connection().await)
        .await?;

        Ok(cat)
    }

    pub async fn delete(user_id: Uuid, id: Uuid) -> Result<bool, Error> {
        let result = sqlx::query("delete from cat where user_id = $1 and id = $2")
            .bind(user_id)
            .bind(id)
            .execute(database::connection().await)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    #[cfg(test)]
    pub async fn delete_all() -> Result<(), Error> {
        sqlx::query("delete from cat")
            .execute(database::connection().await)
            .await?;

        Ok(())
    }
}
