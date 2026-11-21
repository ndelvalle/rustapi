use std::sync::Arc;

use rustapi::app::{create_app, Context};
use rustapi::common::token;
use rustapi::models::user::{hash_password, User};
use rustapi::settings::Settings;
use sqlx::PgPool;
use tokio::net::TcpListener;

// This is an instance of the API listening on its own port and backed by its own database.
pub struct TestApp {
    pub address: String,
    pub db: PgPool,
    pub settings: Arc<Settings>,
}

impl TestApp {
    pub async fn spawn(db: PgPool) -> Self {
        let settings = Arc::new(Settings::from_run_mode("test").expect("Failed to load settings"));
        let app = create_app(Context {
            db: db.clone(),
            settings: settings.clone(),
        });

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind a random port");
        let address = listener.local_addr().unwrap().to_string();

        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("Failed to start server");
        });

        Self {
            address,
            db,
            settings,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.address, path)
    }

    pub async fn create_user(&self, email: &str) -> User {
        let password_hash = hash_password("Password1".to_owned()).await.unwrap();

        User::create(&self.db, "Nahuel", email, &password_hash)
            .await
            .unwrap()
    }

    pub fn token(&self, user: &User) -> String {
        token::create(user.user_id, &self.settings.auth.secret).unwrap()
    }
}
