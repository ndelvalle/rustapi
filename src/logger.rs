use std::env;

pub fn setup(level: &str) {
    if env::var_os("RUST_LOG").is_none() {
        let app_name = env!("CARGO_PKG_NAME");
        let env = format!("{app_name}={level},tower_http={level}");

        env::set_var("RUST_LOG", env);
    }

    tracing_subscriber::fmt::init();
}
