use chrono::{NaiveDateTime, Utc};
use diesel::associations::Identifiable;
use diesel::prelude::{Insertable, Queryable, Selectable};
use diesel::query_dsl::methods::FilterDsl;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use tokio::task;
use validator::Validate;

use crate::database;
use crate::errors::Error;
use crate::schema::users;

#[derive(Debug, Clone, Serialize, Deserialize, Identifiable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub locked_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable, Validate)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser<'a> {
    #[validate(length(min = 5))]
    pub name: &'a str,
    #[validate(email)]
    pub email: &'a str,
    pub password: &'a str,
}

impl User {
    pub async fn create(name: &str, email: &str, password_hash: &str) -> User {
        let now = Utc::now().naive_utc();
        let new_user = NewUser {
            name,
            email,
            password: password_hash,
        };

        let mut conn = database::get_connection().await;
        let user = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<Self>(&mut conn)
            .await
            .expect("Error saving new post");

        user
    }

    pub async fn find_by_email(value: &str) -> Result<Option<Self>, Error> {
        use crate::schema::users::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::OptionalExtension;

        let mut conn = database::get_connection().await;
        users
            .filter(email.eq(value))
            .first::<User>(&mut conn)
            .await
            .optional()
            .map_err(Error::Diesel)
    }

    pub fn is_password_match(&self, password: &str) -> bool {
        bcrypt::verify(password, self.password.as_ref()).unwrap_or(false)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub updated_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

impl From<User> for PublicUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name.clone(),
            email: user.email.clone(),
            updated_at: user.updated_at,
            created_at: user.created_at,
        }
    }
}

pub async fn hash_password<P>(password: P) -> Result<String, Error>
where
    P: AsRef<str> + Send + 'static,
{
    // TODO: Hash password with salt.
    // https://docs.rs/bcrypt/latest/bcrypt/fn.hash_with_salt.html
    #[cfg(not(test))]
    let cost = bcrypt::DEFAULT_COST;
    #[cfg(test)]
    let cost = 4;
    task::spawn_blocking(move || bcrypt::hash(password.as_ref(), cost))
        .await
        .map_err(Error::RunSyncTask)?
        .map_err(Error::HashPassword)
}
