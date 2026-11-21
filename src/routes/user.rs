use axum::http::StatusCode;
use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;
use validator::Validate;

use crate::errors::{AuthenticateError, Error};
use crate::models::user;
use crate::models::user::User;
use crate::settings::SETTINGS;
use crate::utils::custom_response::{CustomResponse, CustomResponseBuilder};
use crate::utils::date::Date;
use crate::utils::token;

pub fn create_route() -> Router {
    Router::new()
        .route("/users", post(create_user))
        .route("/users/authenticate", post(authenticate_user))
}

async fn create_user(Json(body): Json<CreateBody>) -> Result<CustomResponse<UserResponse>, Error> {
    body.validate().map_err(|_| Error::bad_request())?;

    let password_hash = user::hash_password(body.password).await?;
    let user = User::new(body.name, body.email, password_hash);
    let user = User::create(user).await?;
    let res = UserResponse::from(user);

    let res = CustomResponseBuilder::new()
        .body(res)
        .status_code(StatusCode::CREATED)
        .build();

    Ok(res)
}

async fn authenticate_user(
    Json(body): Json<AuthorizeBody>,
) -> Result<Json<AuthenticateResponse>, Error> {
    let email = &body.email;
    let password = &body.password;

    if email.is_empty() {
        debug!("Missing email, returning 400 status code");
        return Err(Error::bad_request());
    }

    if password.is_empty() {
        debug!("Missing password, returning 400 status code");
        return Err(Error::bad_request());
    }

    let user = User::find_by_email(email).await?;

    let hash = user.as_ref().map(|user| user.password.clone());
    let is_match = user::verify_password(password.clone(), hash).await?;

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

    let secret = SETTINGS.auth.secret.as_str();
    let token = token::create(user.clone(), secret)
        .map_err(|_| Error::Authenticate(AuthenticateError::TokenCreation))?;

    let res = AuthenticateResponse {
        access_token: token,
        user: UserResponse::from(user),
    };

    Ok(Json(res))
}

#[derive(Debug, Deserialize, Validate)]
struct CreateBody {
    #[validate(length(min = 1))]
    name: String,
    #[validate(email)]
    email: String,
    // The upper bound is the longest password bcrypt hashes without truncating.
    #[validate(length(min = 8, max = 72))]
    password: String,
}

#[derive(Debug, Deserialize)]
struct AuthorizeBody {
    email: String,
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
            id: user.id,
            name: user.name,
            email: user.email,
            updated_at: user.updated_at,
            created_at: user.created_at,
        }
    }
}
