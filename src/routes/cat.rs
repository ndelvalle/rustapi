use axum::{
  routing::{get, MethodRouter},
  extract::Extension,
  Router,
};

use crate::context::Context;

pub fn create_route() -> Router {
  route("/cats", get(find_cats))
}

fn route(path: &str, method_router: MethodRouter) -> Router {
  Router::new().route(path, method_router)
}

async fn find_cats(Extension(_context): Extension<Context>,) -> &'static str {
  "Hello, World!"
}
