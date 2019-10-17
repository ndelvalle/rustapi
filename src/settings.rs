use std::env;
use std::collections::HashMap;
use config::{Config, File, Environment};

pub struct Settings {}

impl Settings {
    pub fn new() -> HashMap<String, String> {
        let mut settings = Config::new();

        match settings.merge(File::with_name("config/default")) {
            Err(e) => println!("Error trying to load configuration: {}", e.to_string()),
            _ => (),
        }

        // Add in the current environment file
        // Default to 'dev' env
        // Note that this file is _optional_
        let env = env::var("RUN_MODE").unwrap_or("dev".into());
        match settings.merge(File::with_name(&format!("config/{}", env)).required(false)) {
            Err(e) => println!("Error trying to load configuration: {}", e.to_string()),
            _ => (),
        }

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        match settings.merge(Environment::with_prefix("app")) {
            Err(e) => println!("Error trying to load configuration: {}", e.to_string()),
            _ => (),
        }

        // You can deserialize (and thus freeze) the entire configuration as
        settings.try_into::<HashMap<String, String>>().unwrap()
    }
}