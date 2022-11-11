use serde::Deserialize;

/// This struct is used to represent the query parameters that are sent to the
/// server endpoints for pagination.
#[derive(Debug, Deserialize)]
pub struct RequestQuery {
  pub from: Option<String>,
  pub offset: Option<u64>,
  pub limit: Option<u64>,
}
