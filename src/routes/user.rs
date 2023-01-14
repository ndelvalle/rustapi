use axum::http::StatusCode;
use axum::{routing::post, Json, Router};
use bson::doc;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::errors::{AuthenticateError, Error};
use crate::models::user;
use crate::models::user::{PublicUser, User};
use crate::settings::SETTINGS;
use crate::utils::custom_response::{CustomResponse, CustomResponseBuilder};
use crate::utils::models::ModelExt;
use crate::utils::token;

pub fn create_route() -> Router {
  Router::new()
    .route("/users", post(create_user))
    .route("/users/authenticate", post(authenticate_user))
}

async fn create_user(Json(body): Json<CreateBody>) -> Result<CustomResponse<PublicUser>, Error> {
  let password_hash = user::hash_password(body.password).await?;
  let user = User::new(body.name, body.email, password_hash);
  let user = User::create(user).await?;
  let res = PublicUser::from(user);

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

  let user = User::find_one(doc! { "email": email }, None).await?;

  let user = match user {
    Some(user) => user,
    None => {
      debug!("User not found, returning 401");
      return Err(Error::not_found());
    }
  };

  if !user.is_password_match(password) {
    debug!("User password is incorrect, returning 401 status code");
    return Err(Error::Authenticate(AuthenticateError::WrongCredentials));
  }

  if user.locked_at.is_some() {
    debug!("User is locked, returning 401");
    return Err(Error::Authenticate(AuthenticateError::Locked));
  }

  let secret = SETTINGS.auth.secret.as_str();
  let token = token::create(user.clone(), secret)
    .map_err(|_| Error::Authenticate(AuthenticateError::TokenCreation))?;

  let res = AuthenticateResponse {
    access_token: token,
    user: PublicUser::from(user),
  };

  Ok(Json(res))
}

// TODO: Validate password length
#[derive(Debug, Deserialize)]
struct CreateBody {
  name: String,
  email: String,
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
  pub user: PublicUser,
}
