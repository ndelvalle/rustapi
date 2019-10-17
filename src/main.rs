#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
extern crate config;
extern crate serde;
// #[macro_use(Model)] extern crate wither_derive;
// extern crate wither;

mod settings;

use settings::Settings;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn main() {
    let settings = Settings::new();

    println!("Settings: {:?}", settings);

    rocket::ignite().mount("/", routes![index]).launch();
}
