use axum::{
  extract::Extension,
  routing::{get, post, MethodRouter},
  Json, Router,
};
use serde::Deserialize;

use crate::context::Context;
use crate::models::cat::{Cat, PublicCat};
use crate::models::ModelExt;

pub fn create_route() -> Router {
  Router::new()
    .merge(route("/cats", post(create_cat)))
    .merge(route("/cats", get(find_cats)))
}

fn route(path: &str, method_router: MethodRouter) -> Router {
  Router::new().route(path, method_router)
}

#[derive(Deserialize)]
struct CreateCat {
  name: String,
}

async fn create_cat(
  Extension(context): Extension<Context>,
  Json(payload): Json<CreateCat>,
) -> Json<PublicCat> {
  let cat = Cat::new(payload.name);
  let cat = context.cat.create(cat).await.unwrap();
  let res = PublicCat::from(cat);

  Json(res)
}

async fn find_cats(Extension(_context): Extension<Context>) -> &'static str {
  "Hello, World!"
}
