use crate::database::Database;

use crate::models::cat::Model as CatModel;

#[derive(Clone)]
pub struct Context {
  pub cat: CatModel,
}

impl Context {
  pub fn new(db: Database) -> Self {
    let cat = CatModel::new(db);

    Self { cat }
  }
}
