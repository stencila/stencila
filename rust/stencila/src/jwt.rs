use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// The expiry time of the permissions
    pub exp: i64,

    /// The scope of the permissions. A filesystem path.
    pub scope: Option<String>,
}

/// Custom server errors
///
/// Implements `warp::reject::Reject` so we can provide custom response messages
/// for example, when authorization fails.
#[derive(Debug, Error)]
pub enum JwtError {
    #[error("no token supplied")]
    NoTokenSupplied,

    #[error("authorization scheme is not 'Bearer'")]
    IncorrectAuthorizationScheme,

    #[error("JSON Web Token error {message}")]
    TokenError { message: String },
}

impl warp::reject::Reject for JwtError {}

impl From<jsonwebtoken::errors::Error> for JwtError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        JwtError::TokenError {
            message: error.to_string(),
        }
    }
}

/// Extract a JSON Web Token from an authorization header
pub fn from_auth_header(header: String) -> Result<String, JwtError> {
    if header.len() <= 7 || !header[..7].eq_ignore_ascii_case("bearer ") {
        return Err(JwtError::IncorrectAuthorizationScheme);
    }
    Ok(header[7..].trim().to_string())
}

/// Dump a JSON Web Token to an authorization header
pub fn to_auth_header(jwt: String) -> String {
    format!("Bearer {}", jwt)
}

/// Encode a JSON Web Token
pub fn encode(
    key: &str,
    scope: Option<String>,
    expiry_seconds: Option<i64>,
) -> Result<String, JwtError> {
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(expiry_seconds.unwrap_or(60)))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims { exp, scope };

    match jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS512),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(key.as_bytes()),
    ) {
        Ok(token) => Ok(token),
        Err(error) => Err(error.into()),
    }
}

/// Decode a JSON Web Token
pub fn decode(jwt: &str, key: &str) -> Result<Claims, JwtError> {
    match jsonwebtoken::decode::<Claims>(
        jwt,
        &jsonwebtoken::DecodingKey::from_secret(key.as_bytes()),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512),
    ) {
        Ok(data) => Ok(data.claims),
        Err(error) => Err(error.into()),
    }
}
