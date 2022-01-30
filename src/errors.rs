use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use wither::bson;
use wither::mongodb::error::Error as MongoError;
use wither::WitherError;

#[derive(thiserror::Error, Debug)]
#[error("...")]
pub enum Error {
  #[error("{0}")]
  Wither(#[from] WitherError),

  #[error("{0}")]
  Mongo(#[from] MongoError),

  #[error("{0}")]
  ParseObjectID(#[from] bson::oid::Error),

  #[error("{0}")]
  SerializeMongoResponse(#[from] bson::de::Error),

  #[error("{0}")]
  AuthenticateError(#[from] AuthenticateError),

  #[error("{0}")]
  BadRequest(#[from] BadRequest),

  #[error("{0}")]
  NotFound(#[from] NotFound),
}

impl Error {
  fn get_codes(&self) -> (StatusCode, u16) {
    match *self {
      // 4XX Errors
      Error::ParseObjectID(_) => (StatusCode::BAD_REQUEST, 40001),
      // Error::Wither(WitherError::Mongo(MongoError { ref kind, .. })) => {
      //   let mongo_error = kind.as_ref();
      //   match mongo_error {
      //     // MongoDB E11000 error code represent a duplicate key error
      //     MongoErrorKind::CommandError(MongoCommandError { code: 11000, .. }) => {
      //       (StatusCode::BAD_REQUEST, 40002)
      //     }
      //     _ => (StatusCode::INTERNAL_SERVER_ERROR, 5003),
      //   }
      // }
      Error::BadRequest(_) => (StatusCode::BAD_REQUEST, 40003),
      Error::NotFound(_) => (StatusCode::NOT_FOUND, 40003),

      Error::AuthenticateError(AuthenticateError::MissingCredentials) => {
        (StatusCode::BAD_REQUEST, 40003)
      }
      Error::AuthenticateError(AuthenticateError::WrongCredentials) => {
        (StatusCode::UNAUTHORIZED, 40003)
      }
      Error::AuthenticateError(AuthenticateError::InvalidToken) => {
        (StatusCode::UNAUTHORIZED, 40003)
      }
      Error::AuthenticateError(AuthenticateError::Locked) => (StatusCode::LOCKED, 40003),

      // 5XX Errors
      Error::AuthenticateError(AuthenticateError::TokenCreation) => {
        (StatusCode::INTERNAL_SERVER_ERROR, 40003)
      }
      Error::Wither(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5001),
      Error::Mongo(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5003),
      Error::SerializeMongoResponse(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5009),
    }
  }
}

impl IntoResponse for Error {
  fn into_response(self) -> Response {
    let (status_code, code) = self.get_codes();
    let message = self.to_string();
    let body = Json(json!({ "code": code, "message": message }));

    (status_code, body).into_response()
  }
}

#[derive(thiserror::Error, Debug)]
#[error("...")]
pub enum AuthenticateError {
  #[error("Missing authentication credentials")]
  MissingCredentials,
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
#[error("Bad request. Field: {field}, message: {message}")]
pub struct BadRequest {
  pub field: String,
  pub message: String,
}

impl BadRequest {
  pub fn new(field: String, message: String) -> Self {
    BadRequest { field, message }
  }
}

#[derive(thiserror::Error, Debug)]
#[error("Not found")]
pub struct NotFound {
  resource: String,
  message: String,
}

impl NotFound {
  pub fn new(resource: String) -> Self {
    NotFound {
      resource: resource.clone(),
      message: format!("{} not found", resource),
    }
  }
}
