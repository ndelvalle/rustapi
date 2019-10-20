#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
extern crate config;
extern crate serde;

mod settings;

use settings::Settings;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn main() {
    let settings = match Settings::new() {
        Ok(value) => value,
        Err(err) => panic!("Error trying to load settings. Error: {}", err)
    };

    println!("Settings: {:?}", settings);

    rocket::ignite().mount("/", routes![index]).launch();
}
