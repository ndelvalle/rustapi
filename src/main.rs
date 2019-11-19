#![feature(proc_macro_hygiene, decl_macro)]

extern crate config;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
extern crate serde;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate diesel;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_json;
extern crate slog_async;
extern crate slog_term;

mod components;
mod database;
mod logger;
mod server;
mod settings;

use database::Database;
use logger::Logger;
use server::Server;
use settings::Settings;

fn main() {
    let settings = match Settings::new() {
        Ok(value) => value,
        Err(err) => panic!("Error trying to setup configuration. Error: {}", err),
    };

    let logger = match Logger::new(&settings.logger.level) {
        Ok(value) => value,
        Err(err) => panic!("Error trying to setup logger. Error: {}", err),
    };

    let database = match Database::new(&settings, logger.new(o!("context" => "database"))) {
        Ok(value) => value,
        Err(err) => panic!("Error trying to setup database. Error: {}", err),
    };

    let server = match Server::new(&settings, database, logger.new(o!("context" => "server"))) {
        Ok(value) => value,
        Err(err) => panic!("Error trying to setup server. Error: {}", err),
    };

    info!(logger, "Starting sever at {}", &settings.server);
    server.launch();
}
