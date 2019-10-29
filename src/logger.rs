use crate::failure::Error;
use slog::{Drain, Level, LevelFilter};
use std::str::FromStr;

pub struct Logger {
    pub instance: slog::Logger,
}

impl Logger {
    pub fn new(level: &str) -> Result<Self, Error> {
        let logger_level = Level::from_str(level).unwrap();
        let decorator = slog_term::TermDecorator::new().build();

        let drain = slog_term::CompactFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let drain = LevelFilter::new(drain, logger_level).fuse();

        let app_name = env!("CARGO_PKG_NAME");
        let app_version = env!("CARGO_PKG_VERSION");

        let logger = Self {
            instance: slog::Logger::root(drain, o!(app_name => app_version)),
        };

        Ok(logger)
    }

    pub fn child(&self, context: String) -> Self {
        let instance = self.instance.new(o!("context" => context));
        Self { instance }
    }

    pub fn critical(&self, message: &str) {
        crit!(self.instance, "{}", message);
    }

    pub fn error(&self, message: &str) {
        error!(self.instance, "{}", message);
    }

    pub fn warn(&self, message: &str) {
        warn!(self.instance, "{}", message);
    }

    pub fn info(&self, message: &str) {
        info!(self.instance, "{}", message);
    }

    pub fn debug(&self, message: &str) {
        debug!(self.instance, "{}", message);
    }
}
