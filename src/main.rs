#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate dotenv;

use dotenv::dotenv;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn main() {
    dotenv().ok();
    rocket::ignite().mount("/", routes![index]).launch();
}
