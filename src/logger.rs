use std::env;
use std::io::{self, IsTerminal};
use tracing::Dispatch;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// Logs to stdout at the given level, unless RUST_LOG overrides it. Colors are only used in a
// terminal, so that log collectors receive plain text.
pub fn setup(level: &str) {
    let filter = env::var("RUST_LOG").unwrap_or_else(|_| default_filter(level));
    let ansi = io::stdout().is_terminal();

    subscriber(&filter, io::stdout, ansi).init();
}

pub fn default_filter(level: &str) -> String {
    let app_name = env!("CARGO_PKG_NAME");

    format!("{app_name}={level},tower_http={level}")
}

pub fn subscriber<W>(filter: &str, writer: W, ansi: bool) -> Dispatch
where
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    let targets = filter.parse::<Targets>().expect("Invalid log filter");
    let format = tracing_subscriber::fmt::layer()
        .with_writer(writer)
        .with_ansi(ansi);

    Dispatch::new(tracing_subscriber::registry().with(format).with(targets))
}
