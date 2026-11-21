use argon2::password_hash::{self, phc::PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::Argon2;
use sqlx::{FromRow, PgPool};
use std::sync::OnceLock;
use tokio::task;
use uuid::Uuid;

use crate::common::date::Date;
use crate::errors::Error;

static DUMMY_HASH: OnceLock<String> = OnceLock::new();

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub locked_at: Option<Date>,
    pub created_at: Date,
    pub updated_at: Date,
}

impl User {
    pub async fn create(
        db: &PgPool,
        name: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<Self, Error> {
        let user = sqlx::query_as(
            r#"insert into "user" (name, email, password)
               values ($1, $2, $3)
               returning *"#,
        )
        .bind(name)
        .bind(email)
        .bind(password_hash)
        .fetch_one(db)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(db: &PgPool, user_id: Uuid) -> Result<Option<Self>, Error> {
        let user = sqlx::query_as(r#"select * from "user" where user_id = $1"#)
            .bind(user_id)
            .fetch_optional(db)
            .await?;

        Ok(user)
    }

    pub async fn find_by_email(db: &PgPool, email: &str) -> Result<Option<Self>, Error> {
        let user = sqlx::query_as(r#"select * from "user" where email = $1"#)
            .bind(email)
            .fetch_optional(db)
            .await?;

        Ok(user)
    }
}

// Checks the password against the hash. A missing hash never matches, but it is still
// checked against a dummy one so that the call takes the same time.
pub async fn verify_password(password: String, hash: Option<String>) -> Result<bool, Error> {
    task::spawn_blocking(move || {
        let is_known = hash.is_some();
        let hash = match &hash {
            Some(hash) => hash.as_str(),
            None => DUMMY_HASH
                .get_or_init(|| hash_with_argon2("").expect("Failed to hash the dummy password")),
        };
        let is_match = PasswordHash::new(hash)
            .map(|hash| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &hash)
                    .is_ok()
            })
            .unwrap_or(false);

        is_known && is_match
    })
    .await
    .map_err(Error::RunSyncTask)
}

pub async fn hash_password(password: String) -> Result<String, Error> {
    task::spawn_blocking(move || hash_with_argon2(&password))
        .await
        .map_err(Error::RunSyncTask)?
        .map_err(Error::HashPassword)
}

// Hashes with Argon2id and a random salt, using the parameters recommended by OWASP.
fn hash_with_argon2(password: &str) -> Result<String, password_hash::Error> {
    let hash = Argon2::default().hash_password(password.as_bytes())?;

    Ok(hash.to_string())
}
