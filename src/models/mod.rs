pub mod cat;
pub mod user;

use crate::lib::models::ModelExt;
use crate::Database;
use crate::Error;

#[derive(Clone)]
pub struct Models {
  pub user: user::Model,
  pub cat: cat::Model,
}

impl Models {
  pub async fn setup(db: Database) -> Result<Self, Error> {
    let user = user::Model::new(db.clone());
    let cat = cat::Model::new(db.clone());

    let this = Self { user, cat };

    this.sync_indexes().await?;
    Ok(this)
  }

  pub async fn sync_indexes(&self) -> Result<(), Error> {
    self.user.sync_indexes().await?;
    self.cat.sync_indexes().await?;

    Ok(())
  }
}
