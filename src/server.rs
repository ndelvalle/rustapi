use failure::Error;
use rocket::config::{Config, Environment, LoggingLevel};
use rocket::response::content;
use rocket::Rocket;
use rocket::State;
use rocket_contrib::json::Json;
use serde_json::Value;

use crate::components::posts::route;
use crate::database::Database;
use crate::settings::Settings;

pub struct Server;

impl Server {
    pub fn new(
        settings: &Settings,
        database: Database,
        logger: slog::Logger,
    ) -> Result<Rocket, Error> {
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
            .manage(database.pool)
            .manage(logger.new(o!("context" => "router")))
            .mount("/", routes![index, status])
            .mount(
                "/v1",
                routes![
                    route::find,
                    route::create,
                    route::find_by_id,
                    route::update,
                    route::delete,
                    route::find_by_title
                ],
            );

        Ok(server)
    }
}

#[get("/")]
fn index(logger: State<slog::Logger>) -> content::Json<&'static str> {
    debug!(logger, "Retrieving hello world route status");

    content::Json(r#"{ "hello": "world" }"#)
}

#[get("/status")]
fn status(logger: State<slog::Logger>) -> Json<Value> {
    debug!(logger, "Retrieving API status");

    Json(json!({
        "app": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
