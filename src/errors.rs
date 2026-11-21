use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bcrypt::BcryptError;
use serde_json::json;
use tokio::task::JoinError;
use tracing::error;

#[derive(thiserror::Error, Debug)]
#[error("...")]
pub enum Error {
    #[error("{0}")]
    Database(sqlx::Error),

    #[error("Error parsing ID {0}")]
    ParseId(String),

    #[error("Conflict")]
    Conflict,

    #[error("{0}")]
    Authenticate(#[from] AuthenticateError),

    #[error("{0}")]
    BadRequest(#[from] BadRequest),

    #[error("{0}")]
    NotFound(#[from] NotFound),

    #[error("{0}")]
    RunSyncTask(#[from] JoinError),

    #[error("{0}")]
    HashPassword(#[from] BcryptError),
}

impl Error {
    fn get_codes(&self) -> (StatusCode, u16) {
        match *self {
            // 4XX Errors
            Error::ParseId(_) => (StatusCode::BAD_REQUEST, 40001),
            Error::BadRequest(_) => (StatusCode::BAD_REQUEST, 40002),
            Error::NotFound(_) => (StatusCode::NOT_FOUND, 40003),
            Error::Authenticate(AuthenticateError::WrongCredentials) => {
                (StatusCode::UNAUTHORIZED, 40004)
            }
            Error::Authenticate(AuthenticateError::InvalidToken) => {
                (StatusCode::UNAUTHORIZED, 40005)
            }
            Error::Authenticate(AuthenticateError::Locked) => (StatusCode::LOCKED, 40006),
            Error::Conflict => (StatusCode::CONFLICT, 40007),

            // 5XX Errors
            Error::Authenticate(AuthenticateError::TokenCreation) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 5001)
            }
            Error::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5002),
            Error::RunSyncTask(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5005),
            Error::HashPassword(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5006),
        }
    }

    pub fn bad_request() -> Self {
        Error::BadRequest(BadRequest {})
    }

    pub fn not_found() -> Self {
        Error::NotFound(NotFound {})
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        match error.as_database_error() {
            Some(db_error) if db_error.is_unique_violation() => Error::Conflict,
            _ => Error::Database(error),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status_code, code) = self.get_codes();
        let message = if status_code.is_server_error() {
            error!("Internal server error: {self:?}");
            "Internal server error".to_owned()
        } else {
            self.to_string()
        };
        let body = Json(json!({ "code": code, "message": message }));

        (status_code, body).into_response()
    }
}

#[derive(thiserror::Error, Debug)]
#[error("...")]
pub enum AuthenticateError {
    #[error("Wrong authentication credentials")]
    WrongCredentials,
    #[error("Failed to create authentication token")]
    TokenCreation,
    #[error("Invalid authentication credentials")]
    InvalidToken,
    #[error("User is locked")]
    Locked,
}

#[derive(thiserror::Error, Debug)]
#[error("Bad Request")]
pub struct BadRequest {}

#[derive(thiserror::Error, Debug)]
#[error("Not found")]
pub struct NotFound {}
