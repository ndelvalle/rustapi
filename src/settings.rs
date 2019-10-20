use config::{Config, ConfigError, Environment, File};
use std::{env, fmt};

#[derive(Debug, Deserialize)]
pub struct Server {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Logger {
    pub level: String,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub environment: String,
    pub server: Server,
    pub logger: Logger,
    pub database: Database,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::new();

        settings.merge(File::with_name("config/default"))?;

        let env = env::var("RUN_MODE").unwrap_or("development".into());
        settings.merge(File::with_name(&format!("config/{}", env)).required(false))?;
        settings.merge(File::with_name("config/local").required(false))?;

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        settings.merge(Environment::with_prefix("app"))?;

        settings.try_into()
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", &self.address, &self.port)
    }
}
