pub mod cat;
pub mod user;

use crate::lib::models::ModelExt;
use crate::Error;
use cat::Cat;
use user::User;

pub async fn sync_indexes() -> Result<(), Error> {
  User::sync_indexes().await?;
  Cat::sync_indexes().await?;

  Ok(())
}
