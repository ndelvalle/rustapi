use axum::extract::{FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::{Authorization, HeaderMapExt};
use uuid::Uuid;

use crate::app::Context;
use crate::common::token;
use crate::errors::AuthenticateError;
use crate::errors::Error;
use crate::models::user::User;

// This is the user that made the request, as loaded by the authentication middleware.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
}

impl From<User> for AuthenticatedUser {
    fn from(user: User) -> Self {
        Self {
            user_id: user.user_id,
            name: user.name,
            email: user.email,
        }
    }
}

// This middleware authenticates the request. Besides validating the token, it loads the
// user so that removed and locked users are rejected even when their token is still valid.
pub async fn authenticate_request(
    State(ctx): State<Context>,
    mut request: Request,
    next: Next,
) -> Result<Response, Error> {
    let bearer = request
        .headers()
        .typed_get::<Authorization<Bearer>>()
        .ok_or(AuthenticateError::InvalidToken)?;

    let token_data = token::decode(bearer.token(), &ctx.settings.auth.secret)
        .map_err(|_| AuthenticateError::InvalidToken)?;

    let user = User::find_by_id(&ctx.db, token_data.claims.sub)
        .await?
        .ok_or(AuthenticateError::InvalidToken)?;

    if user.locked_at.is_some() {
        return Err(AuthenticateError::Locked.into());
    }

    request
        .extensions_mut()
        .insert(AuthenticatedUser::from(user));

    Ok(next.run(request).await)
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or_else(|| AuthenticateError::InvalidToken.into())
    }
}
