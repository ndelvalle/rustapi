use std::env;

use crate::settings::SETTINGS;

pub fn setup() {
  if env::var_os("RUST_LOG").is_none() {
    let level = SETTINGS.logger.level.as_str();
    let env = format!("rustapi={},tower_http={}", level, level);

    env::set_var("RUST_LOG", env);
  }

  tracing_subscriber::fmt::init();
}
