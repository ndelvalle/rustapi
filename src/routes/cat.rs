use axum::{
  routing::{get, MethodRouter},
  Router,
};

pub fn create_route() -> Router {
  route("/cats", get(find_cats))
}

fn route(path: &str, method_router: MethodRouter) -> Router {
  Router::new().route(path, method_router)
}

async fn find_cats() -> &'static str {
  "Hello, World!"
}
