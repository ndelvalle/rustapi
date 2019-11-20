use crate::failure::Error;
use slog::{Drain, Level, LevelFilter};
use std::str::FromStr;

pub struct Logger;

impl Logger {
    pub fn new(level: &str) -> Result<slog::Logger, Error> {
        let logger_level = Level::from_str(level).unwrap();
        let decorator = slog_term::TermDecorator::new().build();

        let drain = slog_term::CompactFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let drain = LevelFilter::new(drain, logger_level).fuse();

        let app_name = env!("CARGO_PKG_NAME");
        let app_version = env!("CARGO_PKG_VERSION");

        Ok(slog::Logger::root(drain, o!(app_name => app_version)))
    }
}
