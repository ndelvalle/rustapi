use axum::{
  http::header::{self, HeaderValue},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::Serialize;

pub struct CustomResponse<T: Serialize> {
  pub body: Option<T>,
  pub status_code: StatusCode,
}

pub struct CustomResponseBuilder<T: Serialize> {
  pub body: Option<T>,
  pub status_code: StatusCode,
}

impl<T> Default for CustomResponseBuilder<T>
where
  T: Serialize,
{
  fn default() -> Self {
    Self {
      body: None,
      status_code: StatusCode::OK,
    }
  }
}

impl<T> CustomResponseBuilder<T>
where
  T: Serialize,
{
  pub fn new() -> Self {
    Self::default()
  }

  pub fn body(mut self, body: T) -> Self {
    self.body = Some(body);
    self
  }

  pub fn status_code(mut self, status_code: StatusCode) -> Self {
    self.status_code = status_code;
    self
  }

  pub fn build(self) -> CustomResponse<T> {
    CustomResponse {
      body: self.body,
      status_code: self.status_code,
    }
  }
}

impl<T> IntoResponse for CustomResponse<T>
where
  T: Serialize,
{
  fn into_response(self) -> Response {
    let body = match self.body {
      Some(body) => serde_json::to_vec(&body),
      None => return (self.status_code).into_response(),
    };

    let bytes = match body {
      Ok(bytes) => bytes,
      Err(_) => {
        return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
      }
    };

    // If there is a body, we assume that the content type is application/json.
    (
      self.status_code,
      [(
        header::CONTENT_TYPE,
        HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
      )],
      bytes,
    )
      .into_response()
  }
}
