use crate::database::Database;

use crate::models::cat::Cat;

#[derive(Clone)]
pub struct Context {
  pub cat: Cat,
}

impl Context {
  pub fn new(db: Database) -> Self {
    let cat = Cat::new(db.clone());

    Self { cat }
  }
}
