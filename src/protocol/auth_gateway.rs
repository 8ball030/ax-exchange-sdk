use crate::{
    types::{Ep3Account, Ep3Username, Password, Token, Username},
    InstrumentV0,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserTokenRequest {
    pub username: Username,
    pub password: Password,
    pub expiration_seconds: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserTokenResponse {
    pub token: Token,
}

/// Request to create an API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub username: String,
    pub password: String,
}

/// Response containing API key credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyResponse {
    pub api_key: String,
    pub secret: String,
}

/// Request to revoke an API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeApiKeyRequest {
    pub api_key: String,
}

/// Response from revoking an API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeApiKeyResponse {
    pub message: String,
}

/// Request to get API keys for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetApiKeysRequest {
    pub username: String,
}

/// Response containing user's API keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetApiKeysResponse {
    pub api_keys: Vec<String>,
}

/// Token validation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidationResponse {
    pub valid: bool,
    pub username: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Response from whoami endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoamiResponse {
    pub username: Username,
}

/// Request to decode a token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodeTokenRequest {
    pub username: Username,
    pub token: Token,
}

/// Response from decode_token endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodeTokenResponse {
    pub username: Username,
    pub ep3_username: Option<Ep3Username>,
    pub ep3_account: Option<Ep3Account>,
    pub is_admin_token: bool,
    #[serde(default)]
    pub can_place_orders: bool,
    #[serde(default)]
    pub enabled_2fa: bool,
}

// TODO: drop this struct on the migration to api-gateway
/// User credentials returned from authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    pub username: Username,
    pub ep3_username: Ep3Username,
    pub ep3_account: Ep3Account,
    pub is_admin_token: bool,
    pub firm: crate::types::Ep3Firm,
}

// TODO: drop this struct on the migration to api-gateway
/// User information for admin operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: Username,
    pub ep3_username: Ep3Username,
    pub ep3_account: Ep3Account,
    pub is_valid: bool,
}

/// Response from instruments endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetInstrumentsResponse {
    pub instruments: Vec<InstrumentV0>,
}

/// Request to create a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: Username,
    pub password: Password,
    pub ep3_username: Option<Ep3Username>,
    pub ep3_account: Option<Ep3Account>,
}

/// Response from creating a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserResponse {
    pub message: String,
}

/// Request to delete a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteUserRequest {
    pub username: Username,
}

/// Response from get_user endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserResponse {
    pub username: Username,
    pub ep3_username: Ep3Username,
    pub ep3_account: Ep3Account,
    pub is_valid: bool,
}
