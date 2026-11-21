use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

mod app;
mod database;
mod errors;
mod logger;
mod models;
mod routes;
mod settings;
mod utils;

// There are a couple approaches to take when implementing E2E tests. This
// approach adds tests on /src/tests, this way tests can reference modules
// inside the src folder. Another approach would be to have the tests in a
// /tests folder on the root of the project, to do this and be able to import
// modules from the src folder, modules need to be exported as a lib.
#[cfg(test)]
mod tests;

use settings::SETTINGS;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let port = SETTINGS.server.port;
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));

    let app = app::create_app().await;

    let listener = TcpListener::bind(address).await?;
    info!("Server listening on {}", &address);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server stopped, closing database connections");
    database::connection().await.close().await;

    Ok(())
}

// Resolves on Ctrl+C or SIGTERM, letting in-flight requests finish before the server stops.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to listen for the Ctrl+C signal");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to listen for the SIGTERM signal")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received");
}
