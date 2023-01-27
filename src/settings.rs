use config::{Config, ConfigError, Environment, File};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::{env, fmt};

lazy_static! {
  pub static ref SETTINGS: Settings = Settings::new().expect("Failed to setup settings");
}

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
  pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Logger {
  pub level: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
  pub uri: String,
  pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Auth {
  pub secret: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
  pub environment: String,
  pub server: Server,
  pub logger: Logger,
  pub database: Database,
  pub auth: Auth,
}

impl Settings {
  pub fn new() -> Result<Self, ConfigError> {
    let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

    let mut builder = Config::builder()
      .add_source(File::with_name("config/default"))
      .add_source(File::with_name(&format!("config/{run_mode}")).required(false))
      .add_source(File::with_name("config/local").required(false))
      .add_source(Environment::default().separator("__"));

    // Some cloud services like Heroku exposes a randomly assigned port in
    // the PORT env var and there is no way to change the env var name.
    if let Ok(port) = env::var("PORT") {
      builder = builder.set_override("server.port", port)?;
    }

    builder
      .build()?
      // Deserialize (and thus freeze) the entire configuration.
      .try_deserialize()
  }
}

impl fmt::Display for Server {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "http://localhost:{}", &self.port)
  }
}
