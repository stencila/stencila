//! PKCE (Proof Key for Code Exchange) challenge generation.
//!
//! Implements RFC 7636 for OAuth 2.0 public clients. Generates a random
//! code verifier and derives a SHA-256 challenge for the authorization request.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngExt;
use sha2::{Digest, Sha256};

/// A PKCE code verifier and its corresponding challenge.
#[derive(Debug, Clone)]
pub struct PkceChallenge {
    /// The code verifier sent to the token endpoint.
    pub verifier: String,
    /// The S256 challenge sent to the authorization endpoint.
    pub challenge: String,
}

/// Generate a new PKCE challenge pair.
///
/// Creates a 32-byte random verifier, base64url-encodes it, then
/// derives the S256 challenge as `BASE64URL(SHA256(verifier))`.
#[must_use]
pub fn generate() -> PkceChallenge {
    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);

    let digest = Sha256::digest(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(digest);

    PkceChallenge {
        verifier,
        challenge,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_pkce_pair() {
        let pkce = generate();

        // Verifier should be 43 chars (32 bytes base64url-encoded without padding)
        assert_eq!(pkce.verifier.len(), 43);
        // Challenge should be 43 chars (32 bytes SHA256 base64url-encoded without padding)
        assert_eq!(pkce.challenge.len(), 43);

        // Verify the challenge matches the verifier
        let digest = Sha256::digest(pkce.verifier.as_bytes());
        let expected = URL_SAFE_NO_PAD.encode(digest);
        assert_eq!(pkce.challenge, expected);
    }

    #[test]
    fn generates_unique_pairs() {
        let a = generate();
        let b = generate();
        assert_ne!(a.verifier, b.verifier);
        assert_ne!(a.challenge, b.challenge);
    }

    #[test]
    fn verifier_is_url_safe() {
        let pkce = generate();
        // Should only contain URL-safe base64 characters (no +, /, or =)
        assert!(!pkce.verifier.contains('+'));
        assert!(!pkce.verifier.contains('/'));
        assert!(!pkce.verifier.contains('='));
    }
}
