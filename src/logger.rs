use std::env;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// Logs the application and its HTTP layer at the given level, unless RUST_LOG overrides it.
pub fn setup(level: &str) {
    let app_name = env!("CARGO_PKG_NAME");
    let filter =
        env::var("RUST_LOG").unwrap_or_else(|_| format!("{app_name}={level},tower_http={level}"));
    let targets = filter.parse::<Targets>().expect("Invalid log filter");

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(targets)
        .init();
}
