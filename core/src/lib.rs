use std::sync::RwLock;

uniffi::setup_scaffolding!();

/// Authentication state
#[derive(Debug, Clone, uniffi::Record)]
pub struct AuthTokens {
    pub access_token: String,
    pub id_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: u64,
}

/// User info from token
#[derive(Debug, Clone, uniffi::Record)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
}

/// API configuration
#[derive(Debug, Clone, uniffi::Record)]
pub struct ApiConfig {
    pub api_url: String,
    pub cognito_domain: String,
    pub cognito_client_id: String,
}

/// Errors that can occur
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum CoreError {
    #[error("Not authenticated")]
    NotAuthenticated,
    #[error("Token expired")]
    TokenExpired,
    #[error("Network error: {msg}")]
    Network { msg: String },
    #[error("Invalid response: {msg}")]
    InvalidResponse { msg: String },
}

/// Global auth state (simple for now)
static AUTH_STATE: RwLock<Option<AuthTokens>> = RwLock::new(None);
static CONFIG: RwLock<Option<ApiConfig>> = RwLock::new(None);

/// Initialize the SDK with configuration
#[uniffi::export]
pub fn initialize(config: ApiConfig) {
    let mut cfg = CONFIG.write().unwrap();
    *cfg = Some(config);
}

/// Store authentication tokens after login
#[uniffi::export]
pub fn set_auth_tokens(tokens: AuthTokens) {
    let mut state = AUTH_STATE.write().unwrap();
    *state = Some(tokens);
}

/// Clear authentication (logout)
#[uniffi::export]
pub fn clear_auth() {
    let mut state = AUTH_STATE.write().unwrap();
    *state = None;
}

/// Check if user is authenticated
#[uniffi::export]
pub fn is_authenticated() -> bool {
    let state = AUTH_STATE.read().unwrap();
    if let Some(tokens) = &*state {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        tokens.expires_at > now
    } else {
        false
    }
}

/// Get current user info (parsed from ID token)
#[uniffi::export]
pub fn get_current_user() -> Result<User, CoreError> {
    let state = AUTH_STATE.read().unwrap();
    let tokens = state.as_ref().ok_or(CoreError::NotAuthenticated)?;

    // Parse JWT payload (base64 decode middle section)
    let parts: Vec<&str> = tokens.id_token.split('.').collect();
    if parts.len() != 3 {
        return Err(CoreError::InvalidResponse {
            msg: "Invalid token format".into(),
        });
    }

    // Decode base64 (JWT uses base64url)
    let payload = parts[1].replace('-', "+").replace('_', "/");

    // Add padding if needed
    let padded = match payload.len() % 4 {
        2 => format!("{}==", payload),
        3 => format!("{}=", payload),
        _ => payload,
    };

    let decoded = base64_decode(&padded).map_err(|_| CoreError::InvalidResponse {
        msg: "Failed to decode token".into(),
    })?;

    let claims: serde_json::Value =
        serde_json::from_slice(&decoded).map_err(|_| CoreError::InvalidResponse {
            msg: "Failed to parse token".into(),
        })?;

    Ok(User {
        id: claims["sub"].as_str().unwrap_or("").to_string(),
        email: claims["email"].as_str().map(String::from),
        name: claims["name"].as_str().map(String::from),
    })
}

/// Get the OAuth authorization URL
#[uniffi::export]
pub fn get_auth_url(redirect_uri: String) -> Result<String, CoreError> {
    let cfg = CONFIG.read().unwrap();
    let config = cfg.as_ref().ok_or(CoreError::InvalidResponse {
        msg: "SDK not initialized".into(),
    })?;

    Ok(format!(
        "{}/oauth2/authorize?client_id={}&response_type=code&scope=openid+email+profile&redirect_uri={}",
        config.cognito_domain,
        config.cognito_client_id,
        redirect_uri
    ))
}

/// Get the token endpoint URL
#[uniffi::export]
pub fn get_token_endpoint() -> Result<String, CoreError> {
    let cfg = CONFIG.read().unwrap();
    let config = cfg.as_ref().ok_or(CoreError::InvalidResponse {
        msg: "SDK not initialized".into(),
    })?;

    Ok(format!("{}/oauth2/token", config.cognito_domain))
}

/// Get configured API URL
#[uniffi::export]
pub fn get_api_url() -> Result<String, CoreError> {
    let cfg = CONFIG.read().unwrap();
    let config = cfg.as_ref().ok_or(CoreError::InvalidResponse {
        msg: "SDK not initialized".into(),
    })?;

    Ok(config.api_url.clone())
}

/// Get current access token for API calls
#[uniffi::export]
pub fn get_access_token() -> Result<String, CoreError> {
    let state = AUTH_STATE.read().unwrap();
    let tokens = state.as_ref().ok_or(CoreError::NotAuthenticated)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if tokens.expires_at <= now {
        return Err(CoreError::TokenExpired);
    }

    Ok(tokens.access_token.clone())
}

// Simple base64 decode (no external dependency)
fn base64_decode(input: &str) -> Result<Vec<u8>, ()> {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut output = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0;

    for c in input.bytes() {
        if c == b'=' {
            break;
        }
        let val = CHARS.iter().position(|&x| x == c).ok_or(())? as u32;
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push((buf >> bits) as u8);
        }
    }

    Ok(output)
}
