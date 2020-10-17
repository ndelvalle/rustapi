use log::LevelFilter;
use log::{ParseLevelError, SetLoggerError};
use pretty_env_logger::env_logger;
use std::str::FromStr;

use crate::settings::Settings;
pub struct Logger;
pub enum LoggerError {
    SetLoggerError(SetLoggerError),
    ParseLevelError(ParseLevelError),
}

impl Logger {
    pub fn new(settings: &Settings) -> Result<(), LoggerError> {
        let mut builder = env_logger::Builder::new();
        let level = LevelFilter::from_str(settings.logger.level.as_str())
            .map_err(|err| LoggerError::ParseLevelError(err))?;

        builder.default_format();
        builder.filter_level(level);
        builder
            .try_init()
            .map_err(|err| LoggerError::SetLoggerError(err))
    }
}
