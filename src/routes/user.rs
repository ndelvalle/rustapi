use axum::{
  extract::Extension,
  routing::{post, MethodRouter},
  Json, Router,
};
use serde::Deserialize;

use crate::context::Context;
use crate::models::user::{User, PublicUser};
use crate::models::ModelExt;

pub fn create_route() -> Router {
  Router::new()
    .merge(route("/users", post(create_user)))
}

fn route(path: &str, method_router: MethodRouter) -> Router {
  Router::new().route(path, method_router)
}

#[derive(Deserialize)]
struct CreateUser {
  name: String,
  email: String,
}

async fn create_user(
  Extension(context): Extension<Context>,
  Json(payload): Json<CreateUser>,
) -> Json<PublicUser> {
  let user = User::new(payload.name, payload.email);
  let user = context.user.create(user).await.unwrap();
  let res = PublicUser::from(user);

  Json(res)
}
