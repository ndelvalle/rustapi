use diesel::pg::PgConnection;
use failure::Error;
use r2d2;
use r2d2_diesel::ConnectionManager;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};
use rocket_contrib::databases::diesel;
use std::ops::Deref;

use crate::settings::Settings;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct Database {
    pub pool: Pool,
}

impl Database {
    pub fn new(settings: &Settings, logger: slog::Logger) -> Result<Self, Error> {
        let url = &settings.database.url;
        debug!(logger, "Setting up database using {} url", url);

        let manager = ConnectionManager::<PgConnection>::new(url);
        let this = Self {
            pool: r2d2::Pool::new(manager)?,
        };

        Ok(this)
    }
}

pub struct Connection(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

impl<'a, 'r> FromRequest<'a, 'r> for Connection {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Connection, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(Connection(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl Deref for Connection {
    type Target = PgConnection;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
