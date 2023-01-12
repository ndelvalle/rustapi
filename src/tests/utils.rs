use crate::errors::Error;
use crate::models::user::hash_password;
use crate::models::user::User;
use crate::settings::SETTINGS;
use crate::utils::models::ModelExt;
use crate::utils::token;

pub async fn create_user<T: AsRef<str>>(email: T) -> Result<User, Error> {
  let name = "Nahuel";
  let password = "Password1";

  let password_hash = hash_password(password).await?;
  let user = User::new(name, email.as_ref(), password_hash);
  let user = User::create(user).await?;

  Ok(user)
}

pub async fn create_user_token(user: User) -> Result<String, Error> {
  let secret = SETTINGS.auth.secret.as_str();
  let token = token::create(user, secret).unwrap();

  Ok(token)
}
