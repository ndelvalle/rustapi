use std::env;
use config::{Config, File, Environment, ConfigError};

#[derive(Debug, Deserialize)]
struct Database {
    url: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    database: Database,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::new();

        settings.merge(File::with_name("config/default"))?;

        // Add in the current environment file
        // Default to 'dev' env
        // Note that this file is _optional_
        let env = env::var("RUN_MODE").unwrap_or("dev".into());
        settings.merge(File::with_name(&format!("config/{}", env)).required(false))?;
        settings.merge(File::with_name("config/local").required(false))?;

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        settings.merge(Environment::with_prefix("app"))?;
        settings.merge(Environment::new())?;

        // Now that we're done, let's access our configuration
        // println!("debug: {:?}", s.get_bool("debug"));
        // println!("database: {:?}", s.get::<String>("database.url"));

        // You can deserialize (and thus freeze) the entire configuration as
        settings.try_into()
    }
}