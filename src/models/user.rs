use sqlx::FromRow;
use std::sync::LazyLock;
use tokio::task;
use uuid::Uuid;
use validator::Validate;

use crate::database;
use crate::errors::Error;
use crate::utils::date;
use crate::utils::date::Date;

#[cfg(not(test))]
const HASH_COST: u32 = bcrypt::DEFAULT_COST;
#[cfg(test)]
const HASH_COST: u32 = 4;

static DUMMY_HASH: LazyLock<String> =
    LazyLock::new(|| bcrypt::hash("", HASH_COST).expect("Failed to hash the dummy password"));

#[derive(Debug, Clone, FromRow, Validate)]
pub struct User {
    pub id: Uuid,
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    pub password: String,
    pub locked_at: Option<Date>,
    pub created_at: Date,
    pub updated_at: Date,
}

impl User {
    pub fn new<A, B, C>(name: A, email: B, password_hash: C) -> Self
    where
        A: Into<String>,
        B: Into<String>,
        C: Into<String>,
    {
        let now = date::now();
        Self {
            id: Uuid::now_v7(),
            name: name.into(),
            email: email.into(),
            password: password_hash.into(),
            locked_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub async fn create(user: Self) -> Result<Self, Error> {
        user.validate().map_err(|_| Error::bad_request())?;

        let user = sqlx::query_as(
            r#"insert into "user" (id, name, email, password, locked_at, created_at, updated_at)
               values ($1, $2, $3, $4, $5, $6, $7)
               returning *"#,
        )
        .bind(user.id)
        .bind(user.name)
        .bind(user.email)
        .bind(user.password)
        .bind(user.locked_at)
        .bind(user.created_at)
        .bind(user.updated_at)
        .fetch_one(database::connection().await)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(id: Uuid) -> Result<Option<Self>, Error> {
        let user = sqlx::query_as(r#"select * from "user" where id = $1"#)
            .bind(id)
            .fetch_optional(database::connection().await)
            .await?;

        Ok(user)
    }

    pub async fn find_by_email(email: &str) -> Result<Option<Self>, Error> {
        let user = sqlx::query_as(r#"select * from "user" where email = $1"#)
            .bind(email)
            .fetch_optional(database::connection().await)
            .await?;

        Ok(user)
    }

    #[cfg(test)]
    pub async fn delete_all() -> Result<(), Error> {
        sqlx::query(r#"delete from "user""#)
            .execute(database::connection().await)
            .await?;

        Ok(())
    }
}

// Checks the password against the hash. A missing hash never matches, but it is still
// checked against a dummy one so that the call takes the same time.
pub async fn verify_password(password: String, hash: Option<String>) -> Result<bool, Error> {
    task::spawn_blocking(move || {
        let is_known = hash.is_some();
        let hash = hash.as_deref().unwrap_or(DUMMY_HASH.as_str());
        let is_match = bcrypt::verify(password, hash).unwrap_or(false);

        is_known && is_match
    })
    .await
    .map_err(Error::RunSyncTask)
}

pub async fn hash_password<P>(password: P) -> Result<String, Error>
where
    P: AsRef<str> + Send + 'static,
{
    task::spawn_blocking(move || bcrypt::hash(password.as_ref(), HASH_COST))
        .await
        .map_err(Error::RunSyncTask)?
        .map_err(Error::HashPassword)
}
