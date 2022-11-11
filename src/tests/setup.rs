use bson::doc;
use lazy_static::lazy_static;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::time::sleep;
use tokio::time::Duration;

use crate::app::create_app;
use crate::models::cat::Cat;
use crate::models::user::User;
use crate::settings::get_settings;
use crate::utils::models::ModelExt;

lazy_static! {
  static ref RUNTIME: Runtime = Runtime::new().expect("Failed to create Tokio runtime");
}

/// Use the static tokio runtime to start the APP, wait for the server to listen
/// on the specified port, run tests and then clean up the database. Tokio test
/// runtime (tokio::test) is not used to avoid starting the app again on each
/// test. Arguably this would be a better approach, but this API is statless
/// (Without considering the database), so there is no point on starting a new
/// instance on each test. Make sure to run tests sequentially (cargo test -- --test-threads=1)
/// to avoid inconsistency with the Database.
/// Read more: https://github.com/tokio-rs/tokio/issues/2374
pub fn with_app<F>(test: F) -> F::Output
where
  F: std::future::Future,
{
  std::env::set_var("RUN_MODE", "test");
  RUNTIME.block_on(async move {
    let is_app_running = is_app_running().await;

    if !is_app_running {
      tokio::spawn(start_app());
      wait_for_app_to_start().await.unwrap();
    }

    cleanup_database().await;
    test.await
  })
}

async fn start_app() {
  let app = create_app().await;
  let settings = get_settings();
  let port = settings.server.port;
  let address = SocketAddr::from(([127, 0, 0, 1], port));

  axum::Server::bind(&address)
    .serve(app.into_make_service())
    .await
    .expect("Failed to start server");
}

// TODO: Move to utils
async fn wait_for_app_to_start() -> Result<(), &'static str> {
  for _ in 0..2000 {
    let is_running = is_app_running().await;
    if is_running {
      return Ok(());
    }
    sleep(Duration::from_millis(10)).await;
  }

  Err("Could not connect to APP")
}

// TODO: Move to utils
async fn is_app_running() -> bool {
  let settings = get_settings();
  let port = settings.server.port;
  let address = SocketAddr::from(([127, 0, 0, 1], port));
  let is_running = TcpStream::connect(address).await.is_ok();

  is_running
}

// TODO: Move to utils
async fn cleanup_database() {
  Cat::delete_many(doc! {}).await.unwrap();
  User::delete_many(doc! {}).await.unwrap();
}
