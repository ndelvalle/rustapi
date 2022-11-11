use serde::Serialize;

use crate::utils::request_query::RequestQuery;

const LIMIT: u64 = 100;
const OFFSET: u64 = 0;

#[derive(Debug, Serialize)]
pub struct Pagination {
  pub count: u64,
  pub offset: u64,
  pub limit: u64,
}

impl Pagination {
  pub fn build_from_request_query(query: RequestQuery) -> PaginationBuilder {
    let limit = query
      .limit
      // Make sure the requested limit is not greater than the maximum allowed
      // limit.
      .map(|limit| std::cmp::min(limit, LIMIT))
      .unwrap_or(LIMIT);

    let offset = query.offset.unwrap_or(OFFSET);

    PaginationBuilder {
      count: None,
      offset,
      limit,
    }
  }
}

pub struct PaginationBuilder {
  pub count: Option<u64>,
  pub offset: u64,
  pub limit: u64,
}

impl Default for PaginationBuilder {
  fn default() -> Self {
    Self {
      count: None,
      offset: OFFSET,
      limit: LIMIT,
    }
  }
}

impl PaginationBuilder {
  pub fn count(mut self, count: u64) -> Self {
    self.count = Some(count);
    self
  }

  pub fn build(self) -> Pagination {
    Pagination {
      count: self.count.expect("Pagination count to be set"),
      offset: self.offset,
      limit: self.limit,
    }
  }
}
