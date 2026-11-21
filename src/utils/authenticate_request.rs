use axum::extract::{FromRequestParts, Request};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::{Authorization, HeaderMapExt};

use crate::errors::AuthenticateError;
use crate::errors::Error;
use crate::models::user::User;
use crate::settings::SETTINGS;
use crate::utils::token;
use crate::utils::token::TokenUser;

// This middleware authenticates the request. Besides validating the token, it loads the
// user so that removed and locked users are rejected even when their token is still valid.
pub async fn authenticate_request(mut request: Request, next: Next) -> Result<Response, Error> {
    let bearer = request
        .headers()
        .typed_get::<Authorization<Bearer>>()
        .ok_or(AuthenticateError::InvalidToken)?;

    let secret = SETTINGS.auth.secret.as_str();
    let token_data =
        token::decode(bearer.token(), secret).map_err(|_| AuthenticateError::InvalidToken)?;

    let user = User::find_by_id(token_data.claims.user.id)
        .await?
        .ok_or(AuthenticateError::InvalidToken)?;

    if user.locked_at.is_some() {
        return Err(AuthenticateError::Locked.into());
    }

    request.extensions_mut().insert(TokenUser::from(user));

    Ok(next.run(request).await)
}

impl<S> FromRequestParts<S> for TokenUser
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<TokenUser>()
            .cloned()
            .ok_or_else(|| AuthenticateError::InvalidToken.into())
    }
}
