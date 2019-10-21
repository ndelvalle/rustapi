use sloggers::terminal::{Destination, TerminalLoggerBuilder};
use sloggers::types::Severity;
use sloggers::Build;

use std::str::FromStr;

pub struct Logger {
    instance: slog::Logger,
}

impl Logger {
    pub fn new(level: &str) -> Result<Self, sloggers::Error> {
        let mut builder = TerminalLoggerBuilder::new();

        builder.level(Severity::from_str(level)?);
        builder.destination(Destination::Stderr);

        Ok(Self {
            instance: builder.build()?,
        })
    }

    pub fn info(&self, message: &str) {
        info!(self.instance, "{}", message);
    }
}
