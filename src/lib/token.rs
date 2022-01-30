use bson::oid::ObjectId;
use jsonwebtoken::{errors::Error, DecodingKey, EncodingKey, Header, TokenData, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::models::user::User;

type TokenResult = Result<TokenData<Claims>, Error>;

static VALIDATION: Lazy<Validation> = Lazy::new(Validation::default);
static HEADER: Lazy<Header> = Lazy::new(Header::default);

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUser {
  pub id: ObjectId,
  pub name: String,
  pub email: String,
}

impl From<User> for TokenUser {
  fn from(user: User) -> Self {
    Self {
      id: user.id.unwrap(),
      name: user.name.clone(),
      email: user.email,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub exp: usize, // Expiration time (as UTC timestamp). validate_exp defaults to true in validation
  pub iat: usize, // Issued at (as UTC timestamp)
  pub user: TokenUser,
}

impl Claims {
  pub fn new(user: User) -> Self {
    Self {
      exp: (chrono::Local::now() + chrono::Duration::days(30)).timestamp() as usize,
      iat: chrono::Local::now().timestamp() as usize,
      user: TokenUser::from(user),
    }
  }
}

pub fn create(user: User, secret: &str) -> Result<String, Error> {
  let encoding_key = EncodingKey::from_secret(secret.as_ref());
  let claims = Claims::new(user);

  jsonwebtoken::encode(&HEADER, &claims, &encoding_key)
}

pub fn decode(token: &str, secret: &str) -> TokenResult {
  let decoding_key = DecodingKey::from_secret(secret.as_ref());

  jsonwebtoken::decode::<Claims>(token, &decoding_key, &VALIDATION)
}
