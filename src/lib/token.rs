use jsonwebtoken::dangerous_insecure_decode_with_validation;
use jsonwebtoken::{errors::Error, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::models::user::User;

type TokenResult = Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error>;

#[derive(Debug, Serialize, Deserialize)]
struct TokenUser {
  id: String,
  name: String,
  email: String,
}

impl From<User> for TokenUser {
  fn from(cat: User) -> Self {
    Self {
      id: cat.id.unwrap().to_hex(),
      name: cat.name.clone(),
      email: cat.email.clone(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  exp: usize, // Expiration time (as UTC timestamp). validate_exp defaults to true in validation
  iat: usize, // Issued at (as UTC timestamp)
  user: TokenUser,
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

pub fn create_token(user: User, secret: &str) -> Result<String, Error> {
  let header = Header::default();
  let encoding_key = EncodingKey::from_secret(secret.as_ref());
  let claims = Claims::new(user);

  jsonwebtoken::encode(&header, &claims, &encoding_key)
}

pub fn decode_token(token: &str, private_key: &str) -> TokenResult {
  let validation = Validation::default();
  let decoding_key = DecodingKey::from_secret(private_key.as_ref());

  jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation)
}

pub fn get_token_payload(token: &str) -> TokenResult {
  let validation = Validation::default();

  dangerous_insecure_decode_with_validation::<Claims>(token, &validation)
}
