use crate::failure::Error;
use crate::rocket;
use crate::settings::Settings;

pub struct Server {}

impl Server {
    pub fn new(settings: &Settings) -> Result<rocket::Rocket, Error> {
        let env = settings.environment.clone();
        let rocket_env = env.parse::<rocket::config::Environment>().unwrap();

        let config = rocket::config::Config::build(rocket_env)
            .address(&settings.server.address)
            .port(settings.server.port)
            .finalize()?;

        Ok(rocket::custom(config).mount("/", routes![index]))
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello World"
}
