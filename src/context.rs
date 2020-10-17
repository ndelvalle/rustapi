use crate::database::Database;

#[derive(Clone)]
pub struct Context {
    pub database: Database,
}
