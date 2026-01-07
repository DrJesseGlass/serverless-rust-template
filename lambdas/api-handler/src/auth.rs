#![allow(dead_code)]

use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::{error, info, warn};

use crate::{json_response, ApiResponse};

/// Cached JWKS (JSON Web Key Set) from Cognito
static JWKS_CACHE: RwLock<Option<JwksCache>> = RwLock::new(None);

#[derive(Clone)]
struct JwksCache {
    keys: HashMap<String, DecodingKey>,
    fetched_at: std::time::Instant,
}

/// JWKS response from Cognito
#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    n: String,
    e: String,
    alg: Option<String>,
}

/// Claims from Cognito JWT token
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub iss: String,
    pub aud: Option<String>,
    pub client_id: Option<String>,
    pub token_use: String,
    pub exp: usize,
    pub iat: usize,
}

/// Authenticated user info extracted from token
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
}

impl From<Claims> for AuthUser {
    fn from(claims: Claims) -> Self {
        Self {
            id: claims.sub,
            email: claims.email,
            name: claims.name,
        }
    }
}

fn unauthorized(message: &str) -> ApiGatewayV2httpResponse {
    warn!(message = message, "Authentication failed");
    json_response(401, &ApiResponse::<()>::error(message))
}

fn extract_token(request: &ApiGatewayV2httpRequest) -> Option<&str> {
    request
        .headers
        .get("authorization")
        .or_else(|| request.headers.get("Authorization"))
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
}

/// Fetch JWKS from Cognito and cache it
fn fetch_jwks(issuer: &str) -> Result<HashMap<String, DecodingKey>, &'static str> {
    let jwks_url = format!("{}/.well-known/jwks.json", issuer);

    // Use blocking HTTP client (ureq is lightweight and works in Lambda)
    let response = ureq::get(&jwks_url).call().map_err(|e| {
        error!(error = %e, url = %jwks_url, "Failed to fetch JWKS");
        "Failed to fetch JWKS"
    })?;

    let jwks: JwksResponse = response.into_json().map_err(|e| {
        error!(error = %e, "Failed to parse JWKS response");
        "Failed to parse JWKS"
    })?;

    let mut keys = HashMap::new();
    for jwk in jwks.keys {
        if jwk.kty == "RSA" {
            match DecodingKey::from_rsa_components(&jwk.n, &jwk.e) {
                Ok(key) => {
                    keys.insert(jwk.kid.clone(), key);
                }
                Err(e) => {
                    warn!(error = %e, kid = %jwk.kid, "Failed to parse JWK");
                }
            }
        }
    }

    if keys.is_empty() {
        return Err("No valid RSA keys in JWKS");
    }

    info!(key_count = keys.len(), "Fetched and cached JWKS");
    Ok(keys)
}

/// Get decoding key for the given key ID, fetching JWKS if needed
fn get_decoding_key(kid: &str, issuer: &str) -> Result<DecodingKey, &'static str> {
    // Check cache first
    {
        let cache = JWKS_CACHE.read().unwrap();
        if let Some(ref cached) = *cache {
            // Refresh cache if older than 1 hour
            if cached.fetched_at.elapsed() < std::time::Duration::from_secs(3600) {
                if let Some(key) = cached.keys.get(kid) {
                    return Ok(key.clone());
                }
            }
        }
    }

    // Fetch fresh JWKS
    let keys = fetch_jwks(issuer)?;
    let key = keys.get(kid).cloned().ok_or("Key ID not found in JWKS")?;

    // Update cache
    {
        let mut cache = JWKS_CACHE.write().unwrap();
        *cache = Some(JwksCache {
            keys,
            fetched_at: std::time::Instant::now(),
        });
    }

    Ok(key)
}

/// Validate JWT token and extract claims
pub fn validate_token(token: &str) -> Result<Claims, &'static str> {
    let cognito_issuer =
        std::env::var("COGNITO_ISSUER").map_err(|_| "COGNITO_ISSUER not configured")?;

    // Decode header to get the key ID
    let header = decode_header(token).map_err(|e| {
        error!(error = %e, "Failed to decode token header");
        "Invalid token format"
    })?;

    let kid = header.kid.ok_or("Token missing key ID")?;

    // Get the decoding key (fetches JWKS if needed)
    let decoding_key = get_decoding_key(&kid, &cognito_issuer)?;

    // Set up validation
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    validation.set_issuer(&[&cognito_issuer]);

    // Cognito access tokens don't have 'aud' claim
    validation.validate_aud = false;

    let token_data: TokenData<Claims> = decode(token, &decoding_key, &validation).map_err(|e| {
        error!(error = %e, "Failed to validate token");
        "Invalid token"
    })?;

    let claims = token_data.claims;

    // Verify issuer matches
    if claims.iss != cognito_issuer {
        return Err("Invalid token issuer");
    }

    // Verify token type
    if claims.token_use != "id" && claims.token_use != "access" {
        return Err("Invalid token type");
    }

    Ok(claims)
}

#[allow(clippy::result_large_err)]
pub fn require_auth(
    request: &ApiGatewayV2httpRequest,
) -> Result<AuthUser, ApiGatewayV2httpResponse> {
    let token =
        extract_token(request).ok_or_else(|| unauthorized("Missing authorization header"))?;

    let claims = validate_token(token).map_err(unauthorized)?;

    Ok(AuthUser::from(claims))
}

/// Optional authentication - returns Some(user) if valid token, None otherwise
pub fn optional_auth(request: &ApiGatewayV2httpRequest) -> Option<AuthUser> {
    let token = extract_token(request)?;
    let claims = validate_token(token).ok()?;
    Some(AuthUser::from(claims))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_token() {
        // Basic compile test
    }
}
