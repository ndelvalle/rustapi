// Original author: Christian Gill (@gillchristian)
// From: https://gist.github.com/gillchristian/db76e712cc02bff620b86f0cd2bfb691

use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use serde::Deserialize;

use crate::errors::Error;

const DEFAULT_LIMIT: u32 = 100;
const MAX_LIMIT: u32 = 100;

#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    /// The number of rows to skip before counting.
    #[serde(default)]
    pub offset: u32,
    /// The maximum number of rows to query.
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    DEFAULT_LIMIT
}

impl<S> FromRequestParts<S> for Pagination
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(pagination) = Query::<Pagination>::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::bad_request())?;

        Ok(Self {
            limit: pagination.limit.min(MAX_LIMIT),
            ..pagination
        })
    }
}
