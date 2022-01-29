use crate::database::Database;

use crate::models::cat::Model as CatModel;
use crate::models::user::Model as UserModel;
use crate::settings::Settings;

#[derive(Clone)]
pub struct Context {
  // Models
  pub user: UserModel,
  pub cat: CatModel,
  // Extras
  pub settings: Settings,
}

impl Context {
  pub fn new(db: Database, settings: Settings) -> Self {
    let user = UserModel::new(db.clone());
    let cat = CatModel::new(db);

    Self {
      cat,
      user,
      settings,
    }
  }
}
