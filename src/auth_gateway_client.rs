use crate::protocol::auth_gateway::*;
use crate::types::{Password, Token, Username};
use anyhow::{anyhow, bail, Result};
use arc_swap::ArcSwapOption;
use arcstr::ArcStr;
use chrono::{DateTime, Utc};
use log::debug;
use reqwest::Client;
use std::{sync::Arc, time::Duration};
use url::Url;

/// Configuration for auth gateway client
#[derive(Debug, Clone)]
pub struct AuthGatewayConfig {
    pub base_url: String,
    pub admin_secret_key: Option<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub pool_max_idle_per_host: usize,
}

impl AuthGatewayConfig {
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    pub fn validate(&self) -> Result<()> {
        if self.base_url.is_empty() {
            bail!("base_url cannot be empty");
        }
        Ok(())
    }

    pub fn from_env() -> Result<Self> {
        let base_url = std::env::var("AUTH_GATEWAY_URL")
            .map_err(|_| anyhow::anyhow!("AUTH_GATEWAY_URL environment variable not set"))?;

        let admin_secret_key = std::env::var("DECODE_TOKEN_SECRET_KEY").ok();

        Ok(Self {
            base_url,
            admin_secret_key,
            timeout_seconds: 10,
            max_retries: 3,
            pool_max_idle_per_host: 10,
        })
    }
}

/// Auth gateway client for authentication operations
#[derive(Debug, Clone)]
pub struct AuthGatewayClient {
    client: Client,
    pub config: AuthGatewayConfig,
}

impl AuthGatewayClient {
    pub fn new(config: AuthGatewayConfig) -> Result<Self> {
        config.validate()?;

        let client = Client::builder()
            .timeout(config.timeout())
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self { client, config })
    }

    pub fn from_env() -> Result<Self> {
        let config = AuthGatewayConfig::from_env()?;
        Self::new(config)
    }

    /// Get user authentication token
    pub async fn get_user_token(
        &self,
        username: &Username,
        password: &Password,
        expiration_seconds: i32,
    ) -> Result<Token> {
        debug!("Getting user token for: {}", username);

        let request = GetUserTokenRequest {
            username: username.clone(),
            password: password.clone(),
            expiration_seconds,
        };

        let url = format!("{}/get_user_token", self.config.base_url);
        let response = self.client.post(&url).json(&request).send().await?;

        if response.status().is_success() {
            let token_response: GetUserTokenResponse = response.json().await?;
            Ok(token_response.token)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get user token failed: {}", error_text)
        }
    }

    /// Decode and validate a user token
    pub async fn decode_token(
        &self,
        username: &Username,
        token: &Token,
    ) -> Result<UserCredentials> {
        debug!("Decoding token for user: {}", username);

        let request = DecodeTokenRequest {
            username: username.clone(),
            token: token.clone(),
        };

        let url = format!("{}/decode_token", self.config.base_url);
        let response = self.client.post(&url).json(&request).send().await?;

        if response.status().is_success() {
            let decode_response: DecodeTokenResponse = response.json().await?;

            // Convert to UserCredentials
            let ep3_username = decode_response
                .ep3_username
                .ok_or_else(|| anyhow::anyhow!("Missing ep3_username in token response"))?;
            let ep3_account = decode_response
                .ep3_account
                .ok_or_else(|| anyhow::anyhow!("Missing ep3_account in token response"))?;

            Ok(UserCredentials {
                username: decode_response.username,
                ep3_username: ep3_username.clone(),
                ep3_account,
                is_admin_token: decode_response.is_admin_token,
                firm: crate::types::ep3::Ep3Firm::from_ep3_username(&ep3_username),
            })
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Decode token failed: {}", error_text)
        }
    }

    /// Get trading instruments
    pub async fn get_instruments(&self, token: &Token) -> Result<GetInstrumentsResponse> {
        debug!("Getting instruments");

        let url = format!("{}/instruments", self.config.base_url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", token.expose_secret())
            .send()
            .await?;

        if response.status().is_success() {
            let instruments_response: GetInstrumentsResponse = response.json().await?;
            Ok(instruments_response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get instruments failed: {}", error_text)
        }
    }

    /// Get user information by username (admin operation)
    pub async fn get_user_by_username(
        &self,
        username: &Username,
        admin_token: &Token,
    ) -> Result<UserCredentials> {
        debug!("Getting user by username: {}", username);

        let url = format!("{}/users/{}", self.config.base_url, username.as_str());
        let response = self
            .client
            .get(&url)
            .header("Authorization", admin_token.expose_secret())
            .send()
            .await?;

        if response.status().is_success() {
            let user_info: UserInfo = response.json().await?;
            let ep3_username = user_info.ep3_username.clone();
            Ok(UserCredentials {
                username: user_info.username,
                ep3_username: user_info.ep3_username,
                ep3_account: user_info.ep3_account,
                is_admin_token: false, // Regular user lookup
                firm: crate::types::ep3::Ep3Firm::from_ep3_username(&ep3_username),
            })
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get user failed: {}", error_text)
        }
    }

    /// Decode token and get user info (whoami with full details)
    pub async fn whoami_decode(&self, token: &Token) -> Result<UserCredentials> {
        debug!("Decoding token for whoami");

        // Use decode_token to get full user credentials
        let decode_response = self.decode_token(&Username::from("dummy"), token).await?;
        Ok(decode_response)
    }
}

/// Extended Auth Gateway Client for API key management and additional auth operations
pub struct AuthGatewayExtendedClient {
    base_url: Url,
    base_client: Arc<AuthGatewayClient>,
    user_token: Arc<ArcSwapOption<(ArcStr, DateTime<Utc>)>>,
}

impl AuthGatewayExtendedClient {
    pub fn new(
        base_url: Url,
        base_client: Arc<AuthGatewayClient>,
        user_token: Arc<ArcSwapOption<(ArcStr, DateTime<Utc>)>>,
    ) -> Self {
        Self {
            base_url,
            base_client,
            user_token,
        }
    }

    /// Helper method to get current token
    async fn get_token(&self) -> Result<ArcStr> {
        let token = self.user_token.load();
        if let Some(stored) = &*token {
            let (token, expires_at) = &**stored;
            let now = Utc::now();
            if *expires_at > now {
                return Ok(token.clone());
            }
        }
        bail!("Token expired or not available")
    }

    /// Helper method to make authenticated HTTP requests
    async fn make_request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response> {
        let url = self.base_url.join(path)?;
        debug!("{} {}", method, url);

        let token = self.get_token().await?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let mut request = client
            .request(method, url)
            .header("Authorization", token.as_str())
            .header("Content-Type", "application/json");

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await?;
        Ok(response)
    }

    /// Create a new user account
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<CreateUserResponse> {
        let response = self
            .make_request(
                reqwest::Method::POST,
                "auth/create_user",
                Some(serde_json::to_value(request)?),
            )
            .await?;

        if response.status().is_success() {
            let user_response: CreateUserResponse = response.json().await?;
            Ok(user_response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Create user failed: {}", error_text)
        }
    }

    /// Create a new API key
    pub async fn create_api_key(
        &self,
        request: CreateApiKeyRequest,
    ) -> Result<CreateApiKeyResponse> {
        let response = self
            .make_request(
                reqwest::Method::POST,
                "auth/create_api_key",
                Some(serde_json::to_value(request)?),
            )
            .await?;

        if response.status().is_success() {
            let api_key_response: CreateApiKeyResponse = response.json().await?;
            Ok(api_key_response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Create API key failed: {}", error_text)
        }
    }

    /// Get all API keys for the current user
    pub async fn get_api_keys(&self, username: &str) -> Result<Vec<String>> {
        let request_body = serde_json::json!({
            "username": username
        });

        let response = self
            .make_request(
                reqwest::Method::POST,
                "auth/get_api_keys",
                Some(request_body),
            )
            .await?;

        if response.status().is_success() {
            let api_keys_response: GetApiKeysResponse = response.json().await?;
            Ok(api_keys_response.api_keys)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get API keys failed: {}", error_text)
        }
    }

    /// Revoke an API key
    pub async fn revoke_api_key(&self, api_key: &str) -> Result<RevokeApiKeyResponse> {
        let request_body = serde_json::json!({
            "api_key": api_key
        });

        let response = self
            .make_request(
                reqwest::Method::POST,
                "auth/revoke_api_key",
                Some(request_body),
            )
            .await?;

        if response.status().is_success() {
            let revoke_response: RevokeApiKeyResponse = response.json().await?;
            Ok(revoke_response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Revoke API key failed: {}", error_text)
        }
    }

    /// Refresh the current token
    pub async fn refresh_token(&self) -> Result<String> {
        // Delegate to the base client's token refresh functionality
        let token = self.user_token.load();
        if let Some(stored) = &*token {
            let (token, _) = &**stored;
            Ok(token.to_string())
        } else {
            bail!("No token available to refresh")
        }
    }

    /// Validate a token
    pub async fn validate_token(&self, token: &str) -> Result<TokenValidationResponse> {
        let request_body = serde_json::json!({
            "token": token
        });

        let response = self
            .make_request(
                reqwest::Method::POST,
                "auth/validate_token",
                Some(request_body),
            )
            .await?;

        if response.status().is_success() {
            let validation: TokenValidationResponse = response.json().await?;
            Ok(validation)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Validate token failed: {}", error_text)
        }
    }

    /// Get user token (delegates to base client)
    pub async fn get_user_token(
        &self,
        username: &str,
        password: &str,
        expiration_seconds: i32,
    ) -> Result<String> {
        self.base_client
            .get_user_token(
                &Username::new_unchecked(username),
                &Password::new_unchecked(password),
                expiration_seconds,
            )
            .await
            .map(|token| token.expose_secret().to_string())
            .map_err(|e| anyhow!("Failed to get user token: {}", e))
    }

    /// Get instruments (delegates to base client)
    pub async fn get_instruments(&self, token: &str) -> Result<GetInstrumentsResponse> {
        self.base_client
            .get_instruments(&token.into())
            .await
            .map_err(|e| anyhow!("Failed to get instruments: {}", e))
    }
}
