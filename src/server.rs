use crate::failure::Error;
use crate::rocket::config::{Config, Environment, LoggingLevel};
use crate::rocket::response::content;
use crate::rocket::Rocket;
use crate::rocket::State;

use crate::logger::Logger;
use crate::settings::Settings;

pub struct Server {}

impl Server {
    pub fn new(settings: &Settings, logger: Logger) -> Result<Rocket, Error> {
        logger.debug("Setting up server environment");

        let environment = settings.environment.clone();
        let rocket_env = environment.parse::<Environment>().unwrap();

        logger.debug("Setting up server address and port");
        let config = Config::build(rocket_env)
            .address(&settings.server.address)
            .port(settings.server.port)
            .log_level(LoggingLevel::Off)
            .finalize()
            .unwrap();

        logger.debug("Setting up server routes and middlewares");
        let server = rocket::custom(config)
            .manage(logger.child("router".to_string()))
            .mount("/", routes![index]);

        Ok(server)
    }
}

#[get("/")]
fn index(logger: State<Logger>) -> content::Json<&'static str> {
    logger.debug("Retrieving hello world route status");

    content::Json(r#"{ "hello": "world" }"#)
}
