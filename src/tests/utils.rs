use crate::errors::Error;
use crate::lib::models::ModelExt;
use crate::models::user::hash_password;
use crate::models::user::User;

pub async fn create_user<T: AsRef<str>>(email: T) -> Result<User, Error> {
  let name = "Nahuel";
  let password = "Password1";

  let password_hash = hash_password(password).await?;
  let user = User::new(name, email.as_ref(), password_hash);
  let user = User::create(user).await?;

  Ok(user)
}
