use axum::{extract::Extension, routing::post, Json, Router};
use bson::doc;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::context::Context;
use crate::errors::BadRequest;
use crate::errors::NotFound;
use crate::errors::{AuthenticateError, Error};
use crate::lib::token;
use crate::models::user;
use crate::models::user::{PublicUser, User};
use crate::models::ModelExt;

pub fn create_route() -> Router {
  Router::new()
    .route("/users", post(create_user))
    .route("/users/authenticate", post(authenticate_user))
}

async fn create_user(
  Extension(context): Extension<Context>,
  Json(body): Json<CreateBody>,
) -> Result<Json<PublicUser>, Error> {
  let password_hash = user::hash_password(body.password).await?;
  let user = User::new(body.name, body.email, password_hash);
  let user = context.models.user.create(user).await?;
  let res = PublicUser::from(user);

  Ok(Json(res))
}

async fn authenticate_user(
  Extension(context): Extension<Context>,
  Json(body): Json<AuthorizeBody>,
) -> Result<Json<AuthenticateResponse>, Error> {
  let email = &body.email;
  let password = &body.password;

  if email.is_empty() {
    debug!("Missing email, returning 400 status code");
    return Err(Error::BadRequest(BadRequest::new(
      "email".to_owned(),
      "Missing email attribute".to_owned(),
    )));
  }

  if password.is_empty() {
    debug!("Missing password, returning 400 status code");
    return Err(Error::BadRequest(BadRequest::new(
      "password".to_owned(),
      "Missing password attribute".to_owned(),
    )));
  }

  let user = context
    .models
    .user
    .find_one(doc! { "email": email }, None)
    .await?;

  let user = match user {
    Some(user) => user,
    None => {
      debug!("User not found, returning 401");
      return Err(Error::NotFound(NotFound::new(String::from("user"))));
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

  let secret = context.settings.auth.secret.as_str();
  let token = token::create(user.clone(), secret)
    .map_err(|_| Error::Authenticate(AuthenticateError::TokenCreation))?;

  let res = AuthenticateResponse {
    access_token: token,
    user: PublicUser::from(user),
  };

  Ok(Json(res))
}

// TODO: Validate password length
#[derive(Deserialize)]
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
struct AuthenticateResponse {
  access_token: String,
  user: PublicUser,
}
