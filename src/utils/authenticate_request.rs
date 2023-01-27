use axum::{
  async_trait,
  extract::{FromRequestParts, TypedHeader},
  headers::{authorization::Bearer, Authorization},
  http::request::Parts,
  RequestPartsExt,
};

use crate::errors::AuthenticateError;
use crate::errors::Error;
use crate::settings::SETTINGS;
use crate::utils::token;
use crate::utils::token::TokenUser;

#[async_trait]
impl<S> FromRequestParts<S> for TokenUser
where
  S: Send + Sync,
{
  type Rejection = Error;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let TypedHeader(Authorization(bearer)) = parts
      .extract::<TypedHeader<Authorization<Bearer>>>()
      .await
      .map_err(|_| AuthenticateError::InvalidToken)?;

    let secret = SETTINGS.auth.secret.as_str();
    let token_data =
      token::decode(bearer.token(), secret).map_err(|_| AuthenticateError::InvalidToken)?;

    Ok(token_data.claims.user)
  }
}
