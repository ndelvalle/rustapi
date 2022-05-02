use crate::Settings;

#[derive(Clone)]
pub struct Context {
  pub settings: Settings,
}

impl Context {
  pub fn new(settings: Settings) -> Self {
    Self { settings }
  }
}
