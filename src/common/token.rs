use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, errors::Error};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use uuid::Uuid;

type TokenResult = Result<TokenData<Claims>, Error>;

static VALIDATION: LazyLock<Validation> = LazyLock::new(Validation::default);
static HEADER: LazyLock<Header> = LazyLock::new(Header::default);

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,  // Subject, the ID of the user the token belongs to
    pub exp: usize, // Expiration time (as UTC timestamp). validate_exp defaults to true in validation
    pub iat: usize, // Issued at (as UTC timestamp)
}

impl Claims {
    pub fn new(user_id: Uuid) -> Self {
        let now = chrono::Utc::now();

        Self {
            sub: user_id,
            exp: (now + chrono::Duration::days(30)).timestamp() as usize,
            iat: now.timestamp() as usize,
        }
    }
}

pub fn create(user_id: Uuid, secret: &str) -> Result<String, Error> {
    let encoding_key = EncodingKey::from_secret(secret.as_ref())?;
    let claims = Claims::new(user_id);

    jsonwebtoken::encode(&HEADER, &claims, &encoding_key)
}

pub fn decode(token: &str, secret: &str) -> TokenResult {
    let decoding_key = DecodingKey::from_secret(secret.as_ref())?;

    jsonwebtoken::decode::<Claims>(token, &decoding_key, &VALIDATION)
}
