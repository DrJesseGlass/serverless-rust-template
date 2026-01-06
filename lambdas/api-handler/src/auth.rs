#![allow(dead_code)]
#![allow(unused_imports)]

use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use jsonwebtoken::{decode, decode_header, DecodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use tracing::{error, warn};

use crate::{json_response, ApiResponse};

/// Cached JWKS (JSON Web Key Set) from Cognito
static JWKS_CACHE: OnceLock<HashMap<String, DecodingKey>> = OnceLock::new();

/// Claims from Cognito JWT token
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,               // User ID (unique identifier)
    pub email: Option<String>,     // User email
    pub name: Option<String>,      // User name
    pub iss: String,               // Issuer (Cognito URL)
    pub aud: Option<String>,       // Audience (client ID) - in access tokens this is missing
    pub client_id: Option<String>, // Client ID (in access tokens)
    pub token_use: String,         // "id" or "access"
    pub exp: usize,                // Expiration timestamp
    pub iat: usize,                // Issued at timestamp
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

/// Error response for auth failures
fn unauthorized(message: &str) -> ApiGatewayV2httpResponse {
    warn!(message = message, "Authentication failed");
    json_response(401, &ApiResponse::<()>::error(message))
}

/// Extract Bearer token from Authorization header
fn extract_token(request: &ApiGatewayV2httpRequest) -> Option<&str> {
    request
        .headers
        .get("authorization")
        .or_else(|| request.headers.get("Authorization"))
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
}

/// Validate JWT token and extract claims
///
/// Note: This is a simplified validation that checks:
/// - Token structure and signature format
/// - Expiration time
/// - Issuer matches Cognito
///
/// For production, you should fetch and cache the JWKS from:
/// https://cognito-idp.{region}.amazonaws.com/{userPoolId}/.well-known/jwks.json
/// and validate the signature against the public keys.
pub fn validate_token(token: &str) -> Result<Claims, &'static str> {
    let cognito_issuer =
        std::env::var("COGNITO_ISSUER").map_err(|_| "COGNITO_ISSUER not configured")?;

    let client_id =
        std::env::var("COGNITO_CLIENT_ID").map_err(|_| "COGNITO_CLIENT_ID not configured")?;

    // Decode header to get the key ID
    let header = decode_header(token).map_err(|e| {
        error!(error = %e, "Failed to decode token header");
        "Invalid token format"
    })?;

    let _kid = header.kid.ok_or("Token missing key ID")?;

    // For now, we'll do a simple decode without signature verification
    // In production, fetch JWKS and verify signature
    // See: https://docs.aws.amazon.com/cognito/latest/developerguide/amazon-cognito-user-pools-using-tokens-verifying-a-jwt.html

    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.set_issuer(&[&cognito_issuer]);

    // For ID tokens, audience is the client ID
    // For access tokens, we check client_id claim instead
    validation.set_audience(&[&client_id]);

    // IMPORTANT: In production, use the actual JWKS public key
    // This insecure_disable_signature_validation is for development only
    validation.insecure_disable_signature_validation();

    let token_data: TokenData<Claims> = decode(token, &DecodingKey::from_secret(&[]), &validation)
        .map_err(|e| {
            error!(error = %e, "Failed to validate token");
            "Invalid token"
        })?;

    // Additional validation
    let claims = token_data.claims;

    // Verify issuer
    if claims.iss != cognito_issuer {
        return Err("Invalid token issuer");
    }

    // Verify token type (accept both id and access tokens)
    if claims.token_use != "id" && claims.token_use != "access" {
        return Err("Invalid token type");
    }

    Ok(claims)
}

/// Middleware to require authentication on a route
///
/// Returns Ok(AuthUser) if authenticated, or an error response if not.
///
/// Usage:
/// ```rust
/// pub async fn my_protected_route(
///     state: &AppState,
///     request: &ApiGatewayV2httpRequest,
/// ) -> ApiGatewayV2httpResponse {
///     let user = match require_auth(request) {
///         Ok(user) => user,
///         Err(response) => return response,
///     };
///     
///     // user.id, user.email, user.name are available
///     // ... handle request
/// }
/// ```
pub fn require_auth(
    request: &ApiGatewayV2httpRequest,
) -> Result<AuthUser, ApiGatewayV2httpResponse> {
    let token =
        extract_token(request).ok_or_else(|| unauthorized("Missing authorization header"))?;

    let claims = validate_token(token).map_err(unauthorized)?;

    Ok(AuthUser::from(claims))
}

/// Optional authentication - returns Some(user) if valid token, None otherwise
///
/// Use this for routes that work with or without auth (e.g., public content with extra features for logged-in users)
pub fn optional_auth(request: &ApiGatewayV2httpRequest) -> Option<AuthUser> {
    let token = extract_token(request)?;
    let claims = validate_token(token).ok()?;
    Some(AuthUser::from(claims))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_token() {
        // This would require mocking the request
        // For now, just verify the module compiles
    }
}
