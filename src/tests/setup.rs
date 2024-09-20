use bson::doc;
use once_cell::sync::Lazy;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::sync::OnceCell;

use crate::app::create_app;
use crate::models::cat::Cat;
use crate::models::user::User;
use crate::settings::SETTINGS;
use crate::utils::models::ModelExt;

static API: OnceCell<()> = OnceCell::const_new();
static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());

pub async fn start_api_once() {
    API.get_or_init(|| async {
        std::env::set_var("RUN_MODE", "test");

        let app = create_app().await;
        let port = SETTINGS.server.port;
        let address = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(address)
            .await
            .expect("error listening on the assigner port");
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("Failed to start server");
        });
    })
    .await;
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
