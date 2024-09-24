use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::TcpListener;
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

use errors::Error;
use settings::SETTINGS;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let port = SETTINGS.server.port;
    let address = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);

    let app = app::create_app().await;

    let listener = TcpListener::bind(address).await?;
    info!("Server listening on {}", &address);

    axum::serve(listener, app).await
}
