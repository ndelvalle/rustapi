use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

use rustapi::app::{Context, create_app};
use rustapi::database;
use rustapi::logger;
use rustapi::settings::Settings;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let settings = Settings::new().expect("Failed to load settings");

    logger::setup(&settings.logger.level);

    let db = database::connect(&settings.database.uri)
        .await
        .expect("Failed to connect to the database");

    database::migrate(&db)
        .await
        .expect("Failed to run database migrations");

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, settings.server.port));
    let app = create_app(Context {
        db: db.clone(),
        settings: Arc::new(settings),
    });

    let listener = TcpListener::bind(address).await?;
    info!("Server listening on {}", &address);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server stopped, closing database connections");
    db.close().await;

    Ok(())
}

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
