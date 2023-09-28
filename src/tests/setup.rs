use bson::doc;
use lazy_static::lazy_static;
use std::net::SocketAddr;
use tokio::runtime::Runtime;
use tokio::sync::OnceCell;

use crate::app::create_app;
use crate::models::cat::Cat;
use crate::models::user::User;
use crate::settings::SETTINGS;
use crate::utils::models::ModelExt;

static API: OnceCell<()> = OnceCell::const_new();

pub async fn start_api_once() {
  API
    .get_or_init(|| async {
      std::env::set_var("RUN_MODE", "test");

      let app = create_app().await;
      let port = SETTINGS.server.port;
      let address = SocketAddr::from(([127, 0, 0, 1], port));

      tokio::spawn(async move {
        axum::Server::bind(&address)
          .serve(app.into_make_service())
          .await
          .expect("Failed to start server");
      });
    })
    .await;
}

lazy_static! {
  static ref RUNTIME: Runtime = Runtime::new().unwrap();
}

pub fn use_app<F>(test: F)
where
  F: std::future::Future,
{
  RUNTIME.block_on(async move {
    start_api_once().await;

    Cat::delete_many(doc! {}).await.unwrap();
    User::delete_many(doc! {}).await.unwrap();

    test.await;
  })
}
