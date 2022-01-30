use axum::{
  extract::Extension,
  routing::{post, MethodRouter},
  Json, Router,
};
use bson::doc;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::context::Context;
use crate::errors::NotFound;
use crate::errors::{AuthenticateError, Error};
use crate::lib::token;
use crate::models::user::{PublicUser, User};
use crate::models::ModelExt;

pub fn create_route() -> Router {
  Router::new()
    .merge(route("/users", post(create_user)))
    .merge(route("/users/authenticate", post(authenticate_user)))
}

fn route(path: &str, method_router: MethodRouter) -> Router {
  Router::new().route(path, method_router)
}

async fn create_user(
  Extension(context): Extension<Context>,
  Json(body): Json<CreateBody>,
) -> Result<Json<PublicUser>, Error> {
  let user = User::new(body.name, body.email);
  let user = context.user.create(user).await?;
  let res = PublicUser::from(user);

  Ok(Json(res))
}

async fn authenticate_user(
  Extension(context): Extension<Context>,
  Json(body): Json<AuthorizeBody>,
) -> Result<Json<AuthenticateResponse>, Error> {
  let email = &body.email;
  let password = &body.password;

  if email.is_empty() || password.is_empty() {
    debug!("Missing credentials, returning 401");
    return Err(Error::AuthenticateError(
      AuthenticateError::MissingCredentials,
    ));
  }

  let user = context.user.find_one(doc! { "email": email }, None).await?;
  let user = match user {
    Some(user) => user,
    None => {
      debug!("User not found, returning 401");
      return Err(Error::NotFound(NotFound::new(String::from("user"))));
    }
  };

  if user.locked_at.is_some() {
    debug!("User is locked, returning 401");
    return Err(Error::AuthenticateError(AuthenticateError::Locked));
  }

  let secret = context.settings.auth.secret.as_str();
  let token = token::create(user.clone(), secret)
    .map_err(|_| Error::AuthenticateError(AuthenticateError::TokenCreation))?;

  let res = AuthenticateResponse {
    access_token: token,
    user: PublicUser::from(user),
  };

  Ok(Json(res))
}

#[derive(Deserialize)]
struct CreateBody {
  name: String,
  email: String,
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
