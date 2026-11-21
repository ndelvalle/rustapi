use axum::extract::State;
use axum::http::StatusCode;
use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;
use validator::Validate;

use crate::app::Context;
use crate::common::custom_response::{CustomResponse, CustomResponseBuilder};
use crate::common::date::Date;
use crate::common::token;
use crate::errors::{AuthenticateError, Error, ResultExt};
use crate::models::user;
use crate::models::user::User;

pub fn create_route() -> Router<Context> {
    Router::new()
        .route("/users", post(create_user))
        .route("/users/authenticate", post(authenticate_user))
}

async fn create_user(
    State(ctx): State<Context>,
    Json(body): Json<CreateBody>,
) -> Result<CustomResponse<UserResponse>, Error> {
    body.validate()?;

    let password_hash = user::hash_password(body.password).await?;
    let user = User::create(&ctx.db, &body.name, &body.email, &password_hash)
        .await
        .on_constraint("user_email_key", |_| {
            Error::conflict("email", "is already in use")
        })?;
    let response = UserResponse::from(user);

    let response = CustomResponseBuilder::new()
        .body(response)
        .status_code(StatusCode::CREATED)
        .build();

    Ok(response)
}

async fn authenticate_user(
    State(ctx): State<Context>,
    Json(body): Json<AuthorizeBody>,
) -> Result<Json<AuthenticateResponse>, Error> {
    body.validate()?;

    let user = User::find_by_email(&ctx.db, &body.email).await?;

    let hash = user.as_ref().map(|user| user.password.clone());
    let is_match = user::verify_password(body.password, hash).await?;

    // An unknown email and a wrong password are answered alike.
    let user = match user {
        Some(user) if is_match => user,
        _ => {
            debug!("Wrong email or password, returning 401 status code");
            return Err(Error::Authenticate(AuthenticateError::WrongCredentials));
        }
    };

    if user.locked_at.is_some() {
        debug!("User is locked, returning 423 status code");
        return Err(Error::Authenticate(AuthenticateError::Locked));
    }

    let token = token::create(user.user_id, &ctx.settings.auth.secret)
        .map_err(|_| Error::Authenticate(AuthenticateError::TokenCreation))?;

    let response = AuthenticateResponse {
        access_token: token,
        user: UserResponse::from(user),
    };

    Ok(Json(response))
}

#[derive(Debug, Deserialize, Validate)]
struct CreateBody {
    #[validate(length(min = 1, message = "must not be empty"))]
    name: String,
    #[validate(email(message = "must be a valid email address"))]
    email: String,
    #[validate(length(min = 8, max = 128, message = "must have between 8 and 128 characters"))]
    password: String,
}

#[derive(Debug, Deserialize, Validate)]
struct AuthorizeBody {
    #[validate(length(min = 1, message = "must not be empty"))]
    email: String,
    #[validate(length(min = 1, message = "must not be empty"))]
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateResponse {
    pub access_token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub updated_at: Date,
    pub created_at: Date,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.user_id,
            name: user.name,
            email: user.email,
            updated_at: user.updated_at,
            created_at: user.created_at,
        }
    }
}
