use crate::failure::Error;
use crate::rocket::config::{Config, Environment, LoggingLevel};
use crate::rocket::response::content;
use crate::rocket::Rocket;
use crate::rocket::State;

use crate::settings::Settings;

pub struct Server {}

impl Server {
    pub fn new(settings: &Settings, logger: slog::Logger) -> Result<Rocket, Error> {
        debug!(logger, "Setting up server environment");

        let environment = settings.environment.clone();
        let rocket_env = environment.parse::<Environment>().unwrap();

        debug!(logger, "Setting up server address and port");
        let config = Config::build(rocket_env)
            .address(&settings.server.address)
            .port(settings.server.port)
            .log_level(LoggingLevel::Off)
            .finalize()
            .unwrap();

        debug!(logger, "Setting up server routes and middlewares");
        let server = rocket::custom(config)
            .manage(logger.new(o!("context" => "router")))
            .mount("/", routes![index]);

        Ok(server)
    }
}

#[get("/")]
fn index(logger: State<slog::Logger>) -> content::Json<&'static str> {
    debug!(logger, "Retrieving hello world route status");

    content::Json(r#"{ "hello": "world" }"#)
}
