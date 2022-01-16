use crate::settings::Settings;

pub struct Logger;

impl Logger {
  pub fn setup(settings: &Settings) {
    if std::env::var_os("RUST_LOG").is_none() {
      let log_level = settings.logger.level.as_str();
      let rust_log = format!("rustapi={}", log_level);
      std::env::set_var("RUST_LOG", rust_log);
    }

    tracing_subscriber::fmt::init();
  }
}
