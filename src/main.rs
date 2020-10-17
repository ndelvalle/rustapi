use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
#[macro_use]
extern crate log;

mod components;
mod context;
mod database;
mod errors;
mod logger;
mod settings;

use components::posts;
use context::Context;
use database::Database;
use logger::Logger;
use settings::Settings;

#[actix_web::main]
async fn main() {
    let settings = match Settings::new() {
        Ok(value) => value,
        Err(err) => panic!("Failed to setup configuration. Error: {}", err),
    };

    match Logger::new(&settings) {
        Ok(value) => value,
        Err(_) => panic!("Failed to setup logger"),
    };

    let database = match Database::new(&settings).await {
        Ok(value) => value,
        Err(_) => panic!("Failed to setup database connection"),
    };

    let context = web::Data::new(Context {
        database: database.clone(),
    });

    let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let socket_address = SocketAddr::new(localhost, settings.server.port);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .app_data(settings.clone())
            .app_data(context.clone())
            .wrap(Cors::new().supports_credentials().finish())
            .service(web::scope("/posts").configure(posts::route::create_router))
    })
    .bind(socket_address)
    .expect("Failed to bind server to specified port")
    .run()
    .await
    .expect("Failed to start server");
}
