use std::net::SocketAddr;
use std::sync::LazyLock;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::sync::OnceCell;

use crate::app::create_app;
use crate::models::cat::Cat;
use crate::models::user::User;
use crate::settings::SETTINGS;

static API: OnceCell<()> = OnceCell::const_new();
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

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

        Cat::delete_all().await.unwrap();
        User::delete_all().await.unwrap();

        test.await;
    })
}
