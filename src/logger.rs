use crate::settings::Settings;

pub struct Logger;

impl Logger {
  pub fn setup(settings: &Settings) {
    if std::env::var_os("RUST_LOG").is_none() {
      let level = settings.logger.level.as_str();
      std::env::set_var("RUST_LOG", level);
    }

    tracing_subscriber::fmt::init();
  }
}
