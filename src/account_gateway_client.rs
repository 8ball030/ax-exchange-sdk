use crate::{
    protocol::{self, account_gateway::*},
    types::*,
};
use anyhow::{bail, Result};
use arc_swap::ArcSwapOption;
use arcstr::ArcStr;
use chrono::{DateTime, Utc};
use log::debug;
use reqwest;
use rust_decimal::Decimal;
use serde_json::Value;
use std::{sync::Arc, time::Duration};
use url::Url;

pub struct AccountGatewayClient {
    base_url: Url,
    user_token: Arc<ArcSwapOption<(ArcStr, DateTime<Utc>)>>,
}

impl AccountGatewayClient {
    pub async fn connect(
        base_url: Url,
        user_token: Arc<ArcSwapOption<(ArcStr, DateTime<Utc>)>>,
    ) -> Result<Self> {
        Ok(Self {
            base_url,
            user_token,
        })
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
        body: Option<Value>,
    ) -> Result<reqwest::Response> {
        let url = self.base_url.join(path)?;
        debug!("{} {}", method, url);

        let token = self.get_token().await?;

        // Create a temporary client for requests
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

    /// Get account balances for a user
    pub async fn get_balances(&self, username: &str) -> Result<Vec<Balance>> {
        let path = format!("balances?username={}", username);
        let response = self.make_request(reqwest::Method::GET, &path, None).await?;

        if response.status().is_success() {
            let balances: Vec<Balance> = response.json().await?;
            Ok(balances)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get balances failed: {}", error_text)
        }
    }

    /// Get account positions for a user
    pub async fn get_positions(&self, username: &str) -> Result<Vec<Position>> {
        let path = format!("positions?username={}", username);
        let response = self.make_request(reqwest::Method::GET, &path, None).await?;

        if response.status().is_success() {
            let positions: Vec<Position> = response.json().await?;
            Ok(positions)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get positions failed: {}", error_text)
        }
    }

    /// Get user account status
    pub async fn get_user_status(&self, username: &str) -> Result<UserStatus> {
        let path = format!("user-status?username={}", username);
        let response = self.make_request(reqwest::Method::GET, &path, None).await?;

        if response.status().is_success() {
            let status: UserStatus = response.json().await?;
            Ok(status)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get user status failed: {}", error_text)
        }
    }

    /// Get open interest data
    pub async fn get_open_interest(&self) -> Result<Vec<OpenInterest>> {
        let response = self
            .make_request(reqwest::Method::GET, "open-interest", None)
            .await?;

        if response.status().is_success() {
            let data: Vec<OpenInterest> = response.json().await?;
            Ok(data)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get open interest failed: {}", error_text)
        }
    }

    /// Get user's fill history
    pub async fn get_fills(
        &self,
        username: &str,
        params: Option<protocol::common::HistoryParams>,
    ) -> Result<protocol::common::PaginatedResponse<Vec<Fill>>> {
        let mut path = format!("fills?username={}", username);

        if let Some(params) = params {
            let mut query_params = Vec::new();

            if let Some(pagination) = params.pagination {
                if let Some(limit) = pagination.limit {
                    query_params.push(format!("limit={}", limit));
                }
                if let Some(offset) = pagination.offset {
                    query_params.push(format!("offset={}", offset));
                }
            }

            if let Some(date_range) = params.date_range {
                if let Some(start) = date_range.start_time {
                    query_params.push(format!("start_time={}", start.to_rfc3339()));
                }
                if let Some(end) = date_range.end_time {
                    query_params.push(format!("end_time={}", end.to_rfc3339()));
                }
            }

            if let Some(filters) = params.filters {
                for (key, value) in filters {
                    query_params.push(format!("{}={}", key, value));
                }
            }

            if !query_params.is_empty() {
                path.push('&');
                path.push_str(&query_params.join("&"));
            }
        }

        let response = self.make_request(reqwest::Method::GET, &path, None).await?;

        if response.status().is_success() {
            let fills: Vec<Fill> = response.json().await?;
            Ok(protocol::common::PaginatedResponse {
                data: fills,
                metadata: None, // TODO: Extract from response headers if available
            })
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get fills failed: {}", error_text)
        }
    }

    /// Get recent fills for a specific symbol
    pub async fn get_last_fills(
        &self,
        username: &str,
        symbol: &str,
        count: u32,
    ) -> Result<Vec<Fill>> {
        let path = format!(
            "last-fills?username={}&symbol={}&count={}",
            username, symbol, count
        );
        let response = self.make_request(reqwest::Method::GET, &path, None).await?;

        if response.status().is_success() {
            let fills: Vec<Fill> = response.json().await?;
            Ok(fills)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get last fills failed: {}", error_text)
        }
    }

    /// Get funding payment history
    pub async fn get_funding_history(
        &self,
        username: &str,
        params: Option<protocol::common::HistoryParams>,
    ) -> Result<protocol::common::PaginatedResponse<Vec<FundingHistory>>> {
        let mut path = format!("funding-history?username={}", username);

        if let Some(params) = params {
            let mut query_params = Vec::new();

            if let Some(pagination) = params.pagination {
                if let Some(limit) = pagination.limit {
                    query_params.push(format!("limit={}", limit));
                }
                if let Some(offset) = pagination.offset {
                    query_params.push(format!("offset={}", offset));
                }
            }

            if let Some(date_range) = params.date_range {
                if let Some(start) = date_range.start_time {
                    query_params.push(format!("start_time={}", start.to_rfc3339()));
                }
                if let Some(end) = date_range.end_time {
                    query_params.push(format!("end_time={}", end.to_rfc3339()));
                }
            }

            if !query_params.is_empty() {
                path.push('&');
                path.push_str(&query_params.join("&"));
            }
        }

        let response = self.make_request(reqwest::Method::GET, &path, None).await?;

        if response.status().is_success() {
            let records: Vec<FundingHistory> = response.json().await?;
            Ok(protocol::common::PaginatedResponse {
                data: records,
                metadata: None,
            })
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get funding history failed: {}", error_text)
        }
    }

    /// Get deposit history
    pub async fn get_deposit_history(
        &self,
        username: &str,
        params: Option<protocol::common::HistoryParams>,
    ) -> Result<protocol::common::PaginatedResponse<Vec<DepositRecord>>> {
        let mut path = format!("deposit-history?username={}", username);

        if let Some(params) = params {
            let mut query_params = Vec::new();

            if let Some(pagination) = params.pagination {
                if let Some(limit) = pagination.limit {
                    query_params.push(format!("limit={}", limit));
                }
                if let Some(offset) = pagination.offset {
                    query_params.push(format!("offset={}", offset));
                }
            }

            if let Some(date_range) = params.date_range {
                if let Some(start) = date_range.start_time {
                    query_params.push(format!("start_time={}", start.to_rfc3339()));
                }
                if let Some(end) = date_range.end_time {
                    query_params.push(format!("end_time={}", end.to_rfc3339()));
                }
            }

            if !query_params.is_empty() {
                path.push('&');
                path.push_str(&query_params.join("&"));
            }
        }

        let response = self.make_request(reqwest::Method::GET, &path, None).await?;

        if response.status().is_success() {
            let records: Vec<DepositRecord> = response.json().await?;
            Ok(protocol::common::PaginatedResponse {
                data: records,
                metadata: None,
            })
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get deposit history failed: {}", error_text)
        }
    }

    /// Get withdrawal history
    pub async fn get_withdrawal_history(
        &self,
        username: &str,
        params: Option<protocol::common::HistoryParams>,
    ) -> Result<protocol::common::PaginatedResponse<Vec<WithdrawalRecord>>> {
        let mut path = format!("withdrawal-history?username={}", username);

        if let Some(params) = params {
            let mut query_params = Vec::new();

            if let Some(pagination) = params.pagination {
                if let Some(limit) = pagination.limit {
                    query_params.push(format!("limit={}", limit));
                }
                if let Some(offset) = pagination.offset {
                    query_params.push(format!("offset={}", offset));
                }
            }

            if let Some(date_range) = params.date_range {
                if let Some(start) = date_range.start_time {
                    query_params.push(format!("start_time={}", start.to_rfc3339()));
                }
                if let Some(end) = date_range.end_time {
                    query_params.push(format!("end_time={}", end.to_rfc3339()));
                }
            }

            if !query_params.is_empty() {
                path.push('&');
                path.push_str(&query_params.join("&"));
            }
        }

        let response = self.make_request(reqwest::Method::GET, &path, None).await?;

        if response.status().is_success() {
            let records: Vec<WithdrawalRecord> = response.json().await?;
            Ok(protocol::common::PaginatedResponse {
                data: records,
                metadata: None,
            })
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get withdrawal history failed: {}", error_text)
        }
    }

    /// Submit deposit request
    pub async fn deposit(&self, request: DepositRequest) -> Result<()> {
        let response = self
            .make_request(
                reqwest::Method::POST,
                "deposit",
                Some(serde_json::to_value(request)?),
            )
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Deposit failed: {}", error_text)
        }
    }

    /// Submit withdrawal request
    pub async fn withdraw(&self, request: WithdrawRequest) -> Result<()> {
        let response = self
            .make_request(
                reqwest::Method::POST,
                "withdraw",
                Some(serde_json::to_value(request)?),
            )
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Withdraw failed: {}", error_text)
        }
    }

    /// Liquidate account
    pub async fn liquidate(&self, request: LiquidateRequest) -> Result<LiquidateResponse> {
        let response = self
            .make_request(
                reqwest::Method::POST,
                "liquidate",
                Some(serde_json::to_value(request)?),
            )
            .await?;

        if response.status().is_success() {
            let liquidate_response: LiquidateResponse = response.json().await?;
            Ok(liquidate_response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Liquidate failed: {}", error_text)
        }
    }

    /// Get trading volume statistics
    pub async fn get_trading_volume(
        &self,
        username: &str,
        params: Option<protocol::common::DateRangeParams>,
    ) -> Result<Decimal> {
        let mut path = format!("trading-volume?username={}", username);

        if let Some(params) = params {
            let mut query_params = Vec::new();

            if let Some(start) = params.start_time {
                query_params.push(format!("start_time={}", start.to_rfc3339()));
            }
            if let Some(end) = params.end_time {
                query_params.push(format!("end_time={}", end.to_rfc3339()));
            }

            if !query_params.is_empty() {
                path.push('&');
                path.push_str(&query_params.join("&"));
            }
        }

        let response = self.make_request(reqwest::Method::GET, &path, None).await?;

        if response.status().is_success() {
            let volume: Decimal = response.json().await?;
            Ok(volume)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get trading volume failed: {}", error_text)
        }
    }

    /// Get deposit statistics
    pub async fn get_deposit_stats(
        &self,
        username: &str,
        params: Option<protocol::common::DateRangeParams>,
    ) -> Result<DepositStats> {
        let mut path = format!("deposit-stats?username={}", username);

        if let Some(params) = params {
            let mut query_params = Vec::new();

            if let Some(start) = params.start_time {
                query_params.push(format!("start_time={}", start.to_rfc3339()));
            }
            if let Some(end) = params.end_time {
                query_params.push(format!("end_time={}", end.to_rfc3339()));
            }

            if !query_params.is_empty() {
                path.push('&');
                path.push_str(&query_params.join("&"));
            }
        }

        let response = self.make_request(reqwest::Method::GET, &path, None).await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            match serde_json::from_str::<DepositStats>(&response_text) {
                Ok(stats) => Ok(stats),
                Err(e) => {
                    bail!(
                        "Failed to deserialize deposit stats from response '{}': {}",
                        response_text,
                        e
                    )
                }
            }
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get deposit stats failed: {}", error_text)
        }
    }

    /// Get withdrawal statistics
    pub async fn get_withdrawal_stats(
        &self,
        username: &str,
        params: Option<protocol::common::DateRangeParams>,
    ) -> Result<WithdrawalStats> {
        let mut path = format!("withdrawal-stats?username={}", username);

        if let Some(params) = params {
            let mut query_params = Vec::new();

            if let Some(start) = params.start_time {
                query_params.push(format!("start_time={}", start.format("%Y-%m-%d")));
            }
            if let Some(end) = params.end_time {
                query_params.push(format!("end_time={}", end.format("%Y-%m-%d")));
            }

            if !query_params.is_empty() {
                path.push('&');
                path.push_str(&query_params.join("&"));
            }
        }

        let response = self.make_request(reqwest::Method::GET, &path, None).await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            match serde_json::from_str::<WithdrawalStats>(&response_text) {
                Ok(stats) => Ok(stats),
                Err(e) => {
                    bail!(
                        "Failed to deserialize withdrawal stats from response '{}': {}",
                        response_text,
                        e
                    )
                }
            }
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get withdrawal stats failed: {}", error_text)
        }
    }

    /// Get admin statistics (requires admin privileges)
    pub async fn get_admin_stats(
        &self,
        params: protocol::common::DateRangeParams,
    ) -> Result<AdminResponse> {
        let mut query_params = Vec::new();

        if let Some(start) = params.start_time {
            query_params.push(format!("start_time={}", start.format("%Y-%m-%d")));
        }
        if let Some(end) = params.end_time {
            query_params.push(format!("end_time={}", end.format("%Y-%m-%d")));
        }

        let path = if query_params.is_empty() {
            "admin-stats".to_string()
        } else {
            format!("admin-stats?{}", query_params.join("&"))
        };

        let response = self.make_request(reqwest::Method::GET, &path, None).await?;

        if response.status().is_success() {
            let stats: AdminResponse = response.json().await?;
            Ok(stats)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get admin stats failed: {}", error_text)
        }
    }
}
