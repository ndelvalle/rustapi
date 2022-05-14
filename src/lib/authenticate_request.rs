use axum::{
  async_trait,
  extract::{FromRequest, RequestParts, TypedHeader},
  headers::{authorization::Bearer, Authorization},
};

use crate::errors::AuthenticateError;
use crate::errors::Error;
use crate::lib::token;
use crate::lib::token::TokenUser;
use crate::settings::get_settings;

#[async_trait]
impl<B> FromRequest<B> for TokenUser
where
  B: Send,
{
  type Rejection = Error;

  async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
    let TypedHeader(Authorization(bearer)) =
      TypedHeader::<Authorization<Bearer>>::from_request(req)
        .await
        .map_err(|_| AuthenticateError::InvalidToken)?;

    let settings = get_settings();
    let secret = settings.auth.secret.as_str();
    let token_data =
      token::decode(bearer.token(), secret).map_err(|_| AuthenticateError::InvalidToken)?;

    Ok(token_data.claims.user)
  }
}
