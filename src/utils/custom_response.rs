use axum::{
  http::header::{self, HeaderValue},
  http::StatusCode,
  response::{IntoResponse, IntoResponseParts, Response, ResponseParts},
};
use bytes::{BufMut, BytesMut};
use serde::Serialize;
use tracing::error;

use crate::errors::Error;

pub type CustomResponseResult<T> = Result<CustomResponse<T>, Error>;

#[derive(Debug)]
pub struct CustomResponse<T: Serialize> {
  pub body: Option<T>,
  pub status_code: StatusCode,
  pub pagination: Option<ResponsePagination>,
}

pub struct CustomResponseBuilder<T: Serialize> {
  pub body: Option<T>,
  pub status_code: StatusCode,
  pub pagination: Option<ResponsePagination>,
}

#[derive(Debug)]
pub struct ResponsePagination {
  pub count: u64,
  pub offset: u64,
  pub limit: u32,
}

impl<T> Default for CustomResponseBuilder<T>
where
  T: Serialize,
{
  fn default() -> Self {
    Self {
      body: None,
      status_code: StatusCode::OK,
      pagination: None,
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

  pub fn pagination(mut self, pagination: ResponsePagination) -> Self {
    self.pagination = Some(pagination);
    self
  }

  pub fn build(self) -> CustomResponse<T> {
    CustomResponse {
      body: self.body,
      status_code: self.status_code,
      pagination: self.pagination,
    }
  }
}

impl<T> IntoResponse for CustomResponse<T>
where
  T: Serialize,
{
  fn into_response(self) -> Response {
    let body = match self.body {
      Some(body) => body,
      None => return (self.status_code).into_response(),
    };

    let mut bytes = BytesMut::new().writer();
    if let Err(err) = serde_json::to_writer(&mut bytes, &body) {
      error!("Error serializing response body as JSON: {:?}", err);
      return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
    }

    let bytes = bytes.into_inner().freeze();
    let headers = [(
      header::CONTENT_TYPE,
      HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
    )];

    match self.pagination {
      Some(pagination) => (self.status_code, pagination, headers, bytes).into_response(),
      None => (self.status_code, headers, bytes).into_response(),
    }
  }
}

impl IntoResponseParts for ResponsePagination {
  type Error = (StatusCode, String);

  fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
    res
      .headers_mut()
      .insert("x-pagination-count", self.count.into());

    res
      .headers_mut()
      .insert("x-pagination-offset", self.offset.into());

    res
      .headers_mut()
      .insert("x-pagination-limit", self.limit.into());

    Ok(res)
  }
}
