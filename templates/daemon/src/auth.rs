//! Authentication module

use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Missing credentials")]
    MissingCredentials,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token expired")]
    TokenExpired,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Authentication result
pub type AuthResult<T> = Result<T, AuthError>;

/// Authentication mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMode {
    /// No authentication required (for local/stdio)
    None,
    /// Capability token from file
    CapabilityToken,
    /// Signed bearer token (JWT)
    SignedBearerToken,
}

/// Authenticator trait for WebSocket connections
pub trait Authenticator: Send + Sync {
    /// Authenticate credentials and return Ok if valid
    fn authenticate(&self, credentials: &Credentials) -> AuthResult<()>;

    /// Return the authentication mode
    fn mode(&self) -> AuthMode;
}

/// Credentials provided by client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Token string (for capability token)
    pub token: Option<String>,
    /// Bearer token (for JWT)
    pub bearer: Option<String>,
}

impl Credentials {
    /// Create empty credentials
    pub fn new() -> Self {
        Self {
            token: None,
            bearer: None,
        }
    }

    /// Set capability token
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Set bearer token
    pub fn with_bearer(mut self, bearer: impl Into<String>) -> Self {
        self.bearer = Some(bearer.into());
        self
    }
}

impl Default for Credentials {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple authenticator that accepts any credentials (for development/testing)
#[derive(Debug, Clone, Default)]
pub struct NoAuthenticator;

impl Authenticator for NoAuthenticator {
    fn authenticate(&self, _credentials: &Credentials) -> AuthResult<()> {
        Ok(())
    }

    fn mode(&self) -> AuthMode {
        AuthMode::None
    }
}

/// File-based capability token authenticator
#[derive(Debug, Clone)]
pub struct CapabilityTokenAuthenticator {
    token: String,
}

impl CapabilityTokenAuthenticator {
    /// Create from a token file
    pub fn from_file(path: &std::path::Path) -> AuthResult<Self> {
        let token = std::fs::read_to_string(path)?.trim().to_string();
        Ok(Self { token })
    }

    /// Create with a known token
    pub fn with_token(token: impl Into<String>) -> Self {
        Self { token: token.into() }
    }
}

impl Authenticator for CapabilityTokenAuthenticator {
    fn authenticate(&self, credentials: &Credentials) -> AuthResult<()> {
        match &credentials.token {
            Some(t) if t == &self.token => Ok(()),
            Some(_) => Err(AuthError::InvalidCredentials),
            None => Err(AuthError::MissingCredentials),
        }
    }

    fn mode(&self) -> AuthMode {
        AuthMode::CapabilityToken
    }
}

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user/client identifier)
    pub sub: String,
    /// Issuer
    pub iss: Option<String>,
    /// Audience
    pub aud: Option<String>,
    /// Expiration time
    pub exp: Option<u64>,
    /// Issued at
    pub iat: Option<u64>,
}

/// Configuration for JWT validation
#[derive(Clone)]
pub struct JwtValidator {
    decoding_key: DecodingKey,
    validation: Validation,
    expected_issuer: Option<String>,
    expected_audience: Option<String>,
}

impl JwtValidator {
    /// Create a new validator with shared secret
    pub fn with_shared_secret(secret: &str, issuer: Option<String>, audience: Option<String>) -> Self {
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        if let Some(iss) = &issuer {
            validation.set_issuer(&[iss]);
        }

        Self {
            decoding_key,
            validation,
            expected_issuer: issuer,
            expected_audience: audience,
        }
    }

    /// Validate a JWT token and return claims
    pub fn validate(&self, token: &str) -> Result<JwtClaims, AuthError> {
        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                    _ => AuthError::InvalidCredentials,
                }
            })?;

        // Validate issuer if configured
        if let Some(expected_iss) = &self.expected_issuer {
            let claims_iss = token_data.claims.iss.as_ref().ok_or(AuthError::InvalidCredentials)?;
            if claims_iss != expected_iss {
                return Err(AuthError::InvalidCredentials);
            }
        }

        // Validate audience if configured
        if let Some(expected_aud) = &self.expected_audience {
            let claims_aud = token_data.claims.aud.as_ref().ok_or(AuthError::InvalidCredentials)?;
            if claims_aud != expected_aud {
                return Err(AuthError::InvalidCredentials);
            }
        }

        Ok(token_data.claims)
    }
}

/// Signed bearer token (JWT) authenticator
#[derive(Clone)]
pub struct SignedBearerTokenAuthenticator {
    validator: JwtValidator,
}

impl SignedBearerTokenAuthenticator {
    /// Create from shared secret file
    pub fn from_secret_file(
        path: &std::path::Path,
        issuer: Option<String>,
        audience: Option<String>,
    ) -> AuthResult<Self> {
        let secret = std::fs::read_to_string(path)?.trim().to_string();
        Ok(Self::with_shared_secret(&secret, issuer, audience))
    }

    /// Create with known secret
    pub fn with_shared_secret(secret: &str, issuer: Option<String>, audience: Option<String>) -> Self {
        Self {
            validator: JwtValidator::with_shared_secret(secret, issuer, audience),
        }
    }
}

impl Authenticator for SignedBearerTokenAuthenticator {
    fn authenticate(&self, credentials: &Credentials) -> AuthResult<()> {
        let bearer = credentials.bearer.as_ref().ok_or(AuthError::MissingCredentials)?;
        self.validator.validate(bearer)?;
        Ok(())
    }

    fn mode(&self) -> AuthMode {
        AuthMode::SignedBearerToken
    }
}
