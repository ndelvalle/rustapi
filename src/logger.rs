use crate::settings::get_settings;

pub struct Logger;

impl Logger {
  pub fn setup() {
    if std::env::var_os("RUST_LOG").is_none() {
      let settings = get_settings();
      let level = settings.logger.level.as_str();
      let env = format!("rustapi={},tower_http=debug", level);

      std::env::set_var("RUST_LOG", env);
    }

    tracing_subscriber::fmt::init();
  }
}
