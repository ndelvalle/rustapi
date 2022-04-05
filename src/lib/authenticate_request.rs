use axum::{
  async_trait,
  extract::{FromRequest, RequestParts, TypedHeader},
  headers::{authorization::Bearer, Authorization},
};

use crate::context::Context;
use crate::errors::AuthenticateError;
use crate::errors::Error;
use crate::lib::token;
use crate::lib::token::TokenUser;

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

    let extensions = req.extensions();
    let context = extensions.get::<Context>().ok_or(Error::ReadContext)?;
    let secret = context.settings.auth.secret.as_str();
    let token_data =
      token::decode(bearer.token(), secret).map_err(|_| AuthenticateError::InvalidToken)?;

    Ok(token_data.claims.user)
  }
}
