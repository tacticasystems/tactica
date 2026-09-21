use std::{path::PathBuf, time::{Duration, Instant, SystemTime}};

use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use tactica_uuid_kinds::UserId;

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("Error encoding JWT token: {0}")]
    Encode(jsonwebtoken::errors::Error),

    #[error("Error decoding JWT key: {0}")]
    KeyDecode(jsonwebtoken::errors::Error),

    #[error("Error reading JWT key: {0}")]
    KeyRead(std::io::Error),
}

#[derive(Debug, Clone)]
pub struct JwtContext {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,

    max_token_lifetime: Duration,
}


#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub tty: String,
    pub exp: i64,
    pub nbf: i64,
}

impl JwtContext {
    pub fn new(
        pubkey: &[u8],
        privkey: &[u8],
    ) -> Result<Self, JwtError> {
        let encoding_key = EncodingKey::from_ed_pem(privkey).map_err(JwtError::KeyDecode)?;
        let decoding_key = DecodingKey::from_ed_pem(pubkey).map_err(JwtError::KeyDecode)?;
        Ok(Self {
            encoding_key,
            decoding_key,

            max_token_lifetime: Duration::from_mins(5),
        })
    }

    pub fn from_files(
        pubkey_path: &PathBuf,
        privkey_path: &PathBuf,
    ) -> Result<Self, JwtError> {
        let pubkey = std::fs::read(pubkey_path).map_err(JwtError::KeyRead)?;
        let privkey = std::fs::read(privkey_path).map_err(JwtError::KeyRead)?;
        Self::new(&pubkey, &privkey)
    }

    pub fn encoding_key(&self) -> &EncodingKey {
        &self.encoding_key
    }

    pub fn decoding_key(&self) -> &DecodingKey {
        &self.decoding_key
    }

    pub fn generate_jwt_for_user(&self, user_id: UserId) -> Result<String, JwtError> {
        let now = Utc::now();
        let exp = now.clone() + self.max_token_lifetime;

        let claims = Claims {
            sub: format!("{}", user_id.to_string()),
            tty: "user".to_string(),
            exp: exp.timestamp(),
            nbf: now.timestamp(),
        };

        let mut header = Header::default();
        header.alg = jsonwebtoken::Algorithm::EdDSA;

        encode(&header, &claims, &self.encoding_key)
            .map_err(JwtError::Encode)
    }

    pub fn validate_jwt(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = jsonwebtoken::Validation::default();
        validation.validate_nbf = true;
        validation.validate_exp = true;
        validation.algorithms = vec![Algorithm::EdDSA];

        let token_data = jsonwebtoken::decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(JwtError::KeyDecode)?;
        Ok(token_data.claims)
    }
}
