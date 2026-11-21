use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use sqlx::error::DatabaseError;
use std::collections::BTreeMap;
use tokio::task::JoinError;
use tracing::error;
use validator::ValidationErrors;

// Maps the name of a field to the reasons why its value was rejected.
pub type FieldErrors = BTreeMap<String, Vec<String>>;

#[derive(thiserror::Error, Debug)]
#[error("...")]
pub enum Error {
    #[error("{0}")]
    Database(#[from] sqlx::Error),

    #[error("Error parsing ID {0}")]
    ParseId(String),

    #[error("Validation failed")]
    Validation(FieldErrors),

    #[error("Conflict")]
    Conflict(FieldErrors),

    #[error("{0}")]
    Authenticate(#[from] AuthenticateError),

    #[error("{0}")]
    BadRequest(#[from] BadRequest),

    #[error("{0}")]
    NotFound(#[from] NotFound),

    #[error("{0}")]
    RunSyncTask(#[from] JoinError),

    #[error("{0}")]
    HashPassword(#[from] argon2::password_hash::Error),
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
            Error::Conflict(_) => (StatusCode::CONFLICT, 40007),
            Error::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, 40008),

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

    pub fn conflict(field: &str, message: &str) -> Self {
        Error::Conflict(FieldErrors::from([(
            field.to_owned(),
            vec![message.to_owned()],
        )]))
    }
}

impl From<ValidationErrors> for Error {
    fn from(errors: ValidationErrors) -> Self {
        let errors = errors
            .field_errors()
            .into_iter()
            .map(|(field, errors)| {
                let messages = errors
                    .iter()
                    .map(|error| match &error.message {
                        Some(message) => message.to_string(),
                        None => error.code.to_string(),
                    })
                    .collect();

                (field.to_string(), messages)
            })
            .collect();

        Error::Validation(errors)
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

        let body = match self {
            Error::Validation(errors) | Error::Conflict(errors) => {
                json!({ "code": code, "message": message, "errors": errors })
            }
            _ => json!({ "code": code, "message": message }),
        };

        (status_code, Json(body)).into_response()
    }
}

// This is used for turning the violation of a named database constraint into an API error.
pub trait ResultExt<T> {
    fn on_constraint(
        self,
        name: &str,
        map_err: impl FnOnce(Box<dyn DatabaseError>) -> Error,
    ) -> Result<T, Error>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn on_constraint(
        self,
        name: &str,
        map_err: impl FnOnce(Box<dyn DatabaseError>) -> Error,
    ) -> Result<T, Error> {
        self.map_err(|error| match error.into() {
            Error::Database(sqlx::Error::Database(db_error))
                if db_error.constraint() == Some(name) =>
            {
                map_err(db_error)
            }
            error => error,
        })
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
