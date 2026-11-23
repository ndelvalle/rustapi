use std::sync::Arc;

use rustapi::app::{Context, create_app};
use rustapi::common::token;
use rustapi::models::user::{User, hash_password};
use rustapi::settings::Settings;
use sqlx::PgPool;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

// This is an instance of the API listening on its own port and backed by its own database.
pub struct TestApp {
    pub address: String,
    pub db: PgPool,
    pub settings: Arc<Settings>,
}

impl TestApp {
    pub async fn spawn(db: PgPool) -> Self {
        Self::spawn_with_settings(db, Self::settings()).await
    }

    pub fn settings() -> Settings {
        Settings::from_run_mode("test").expect("Failed to load settings")
    }

    pub async fn spawn_with_settings(db: PgPool, settings: Settings) -> Self {
        let settings = Arc::new(settings);
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

    // Sends raw bytes and returns the status code and the body of the response. This is used
    // for requests that an HTTP client refuses to send, like one with an incomplete body.
    pub async fn send_raw(&self, request: &str) -> (u16, String) {
        let mut stream = TcpStream::connect(&self.address).await.unwrap();
        stream.write_all(request.as_bytes()).await.unwrap();

        timeout(Duration::from_secs(10), read_raw_response(&mut stream))
            .await
            .expect("The server did not answer in time")
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

// Reads until the whole body announced by the Content-Length header has arrived, because the
// server may keep the connection open after it answers.
async fn read_raw_response(stream: &mut TcpStream) -> (u16, String) {
    let mut response = Vec::new();
    let mut buffer = [0; 4096];

    loop {
        let read = stream.read(&mut buffer).await.unwrap();
        response.extend_from_slice(&buffer[..read]);

        let text = String::from_utf8_lossy(&response);
        if let Some((head, body)) = text.split_once("\r\n\r\n") {
            let content_length = head
                .lines()
                .filter_map(|line| line.split_once(':'))
                .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                .map(|(_, value)| value.trim().parse::<usize>().unwrap())
                .unwrap_or(0);

            if body.len() >= content_length {
                let status = head.split(' ').nth(1).unwrap().parse().unwrap();
                return (status, body.to_owned());
            }
        }

        assert!(
            read > 0,
            "The connection closed before the response was complete"
        );
    }
}
