use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use tokio::net::TcpStream;
use tokio::time::sleep;
use tokio::time::Duration;

use crate::create_app;
use crate::settings::get_settings;

static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub async fn setup() -> Result<(), &'static str> {
  let is_initialized = IS_INITIALIZED.load(Ordering::Relaxed);
  if is_initialized {
    return Ok(());
  }

  std::env::set_var("RUN_MODE", "test");
  tokio::spawn(start_app());
  wait_for_app_to_start().await?;
  IS_INITIALIZED.store(true, Ordering::Relaxed);

  Ok(())
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

async fn wait_for_app_to_start() -> Result<(), &'static str> {
  let settings = get_settings();
  let port = settings.server.port;
  let address = SocketAddr::from(([127, 0, 0, 1], port));

  for _ in 0..5 {
    let started = TcpStream::connect(address).await.is_ok();
    if started {
      return Ok(());
    }
    sleep(Duration::from_secs(1)).await;
  }

  Err("Could not connect to APP")
}
