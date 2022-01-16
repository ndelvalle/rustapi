use axum::{
  routing::{get, MethodRouter},
  Router,
};

pub fn create_route() -> Router {
  route("/cats", get(handler))
}

fn route(path: &str, method_router: MethodRouter) -> Router {
  Router::new().route(path, method_router)
}

async fn handler() -> &'static str {
  "Hello, World!"
}
