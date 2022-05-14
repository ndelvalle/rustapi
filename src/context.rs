use crate::Models;

#[derive(Clone)]
pub struct Context {
  pub models: Models,
}

impl Context {
  pub fn new(models: Models) -> Self {
    Self { models }
  }
}
