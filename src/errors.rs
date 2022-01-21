use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use wither::bson;
use wither::mongodb::error::CommandError as MongoCommandError;
use wither::mongodb::error::Error as MongoError;
use wither::mongodb::error::ErrorKind as MongoErrorKind;
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
}

impl Error {
  fn get_codes(&self) -> (StatusCode, u16) {
    match *self {
      // 4XX Errors
      Error::ParseObjectID(_) => (StatusCode::BAD_REQUEST, 40001),
      Error::Wither(WitherError::Mongo(MongoError { ref kind, .. })) => {
        let mongo_error = kind.as_ref();
        match mongo_error {
          // MongoDB E11000 error code represent a duplicate key error
          MongoErrorKind::CommandError(MongoCommandError { code: 11000, .. }) => {
            (StatusCode::BAD_REQUEST, 40002)
          }
          _ => (StatusCode::INTERNAL_SERVER_ERROR, 5003),
        }
      }

      // 5XX Errors
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
