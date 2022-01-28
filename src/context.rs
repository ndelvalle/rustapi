use crate::database::Database;

use crate::models::cat::Model as CatModel;
use crate::models::user::Model as UserModel;

#[derive(Clone)]
pub struct Context {
  pub user: UserModel,
  pub cat: CatModel,
}

impl Context {
  pub fn new(db: Database) -> Self {
    let user = UserModel::new(db.clone());
    let cat = CatModel::new(db);

    Self { cat, user }
  }
}
