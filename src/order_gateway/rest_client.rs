use crate::types::*;
use anyhow::{bail, Result};
use arc_swap::ArcSwapOption;
use arcstr::ArcStr;
use chrono::{DateTime, Utc};
use log::debug;
use reqwest;
use serde_json::Value;
use std::{sync::Arc, time::Duration};
use url::Url;

pub struct OrderGatewayRestClient {
    client: reqwest::Client,
    base_url: Url,
    username: String,
    token: Arc<ArcSwapOption<(ArcStr, DateTime<Utc>)>>,
}

impl OrderGatewayRestClient {
    pub fn new(
        base_url: Url,
        username: impl AsRef<str>,
        token: Arc<ArcSwapOption<(ArcStr, DateTime<Utc>)>>,
    ) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            client,
            base_url,
            username: username.as_ref().to_string(),
            token,
        })
    }

    /// Helper method to get current token
    fn token(&self) -> Result<ArcStr> {
        let token = self.token.load();
        if let Some(stored) = &*token {
            let (token, expires_at) = &**stored;
            let now = Utc::now();
            if *expires_at > now {
                return Ok(token.clone());
            }
        }
        bail!("token expired or not available")
    }

    /// Helper method to make authenticated HTTP requests
    async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<reqwest::Response> {
        let url = self.base_url.join(path)?;
        debug!("{} {}", method, url);

        let token = self.token()?;
        let mut request = self
            .client
            .request(method, url)
            .header("Authorization", format!("Bearer {}", token.as_str()))
            .header("Content-Type", "application/json");

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await?;
        Ok(response)
    }

    /// Check order gateway health
    pub async fn health(&self) -> Result<HealthResponse> {
        let url = self.base_url.join("orders/health")?;
        debug!("GET {}", url);

        // Create a temporary client for health checks
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        let response = client.get(url).send().await?;

        if response.status().is_success() {
            let health: HealthResponse = response.json().await?;
            Ok(health)
        } else {
            bail!("Orders health check failed: {}", response.status())
        }
    }

    /// Insert order via REST API
    pub async fn insert_order(
        &self,
        symbol: &str,
        side: &str,
        quantity: i64,
        price: &str,
        time_in_force: &str,
        post_only: Option<bool>,
    ) -> Result<String> {
        self.insert_order_with_tag(
            symbol,
            side,
            quantity,
            price,
            time_in_force,
            post_only,
            None,
        )
        .await
    }

    /// Insert order with tag via REST API
    pub async fn insert_order_with_tag(
        &self,
        symbol: &str,
        side: &str,
        quantity: i64,
        price: &str,
        time_in_force: &str,
        post_only: Option<bool>,
        tag: Option<&str>,
    ) -> Result<String> {
        let order_request = InsertOrderRequest {
            username: self.username.clone(),
            symbol: symbol.to_string(),
            side: side.to_string(),
            quantity,
            price: price.to_string(),
            time_in_force: time_in_force.to_string(),
            post_only,
            tag: tag.map(|t| t.to_string()),
        };

        let response = self
            .request(
                reqwest::Method::POST,
                "orders/insert_order",
                Some(serde_json::to_value(order_request)?),
            )
            .await?;

        if response.status().is_success() {
            let insert_response: InsertOrderResponse = response.json().await?;
            Ok(insert_response.order_id)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Insert order failed: {}", error_text)
        }
    }

    /// Cancel specific order via REST API
    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        let cancel_request = CancelOrderRequest {
            username: self.username.clone(),
            order_id: order_id.to_string(),
        };

        let response = self
            .request(
                reqwest::Method::POST,
                "orders/cancel_order",
                Some(serde_json::to_value(cancel_request)?),
            )
            .await?;

        if response.status().is_success() {
            let _cancel_response: CancelOrderResponse = response.json().await?;
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Cancel order failed: {}", error_text)
        }
    }

    /// Get all open orders via REST API
    pub async fn get_open_orders(&self) -> Result<Vec<RestOrderMessage>> {
        let request = GetOpenOrdersRequest {
            username: self.username.clone(),
        };

        let response = self
            .request(
                reqwest::Method::POST,
                "orders/get_open_orders",
                Some(serde_json::to_value(request)?),
            )
            .await?;

        if response.status().is_success() {
            let orders_response: GetOpenOrdersResponse = response.json().await?;
            Ok(orders_response.orders)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get open orders failed: {}", error_text)
        }
    }

    /// Cancel all orders via REST API
    pub async fn cancel_all_orders(&self) -> Result<CancelAllResponse> {
        let request = CancelAllRequest {
            username: self.username.clone(),
        };

        let response = self
            .request(
                reqwest::Method::POST,
                "orders/cancel_all",
                Some(serde_json::to_value(request)?),
            )
            .await?;

        if response.status().is_success() {
            let cancel_response: CancelAllResponse = response.json().await?;
            Ok(cancel_response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Cancel all orders failed: {}", error_text)
        }
    }

    /// Get risk snapshot for the current user via REST API
    pub async fn get_risk_snapshot(&self) -> Result<Value> {
        let path = format!("orders/risk_snapshot/{}", self.username);
        let response = self.request(reqwest::Method::GET, &path, None).await?;

        if response.status().is_success() {
            let risk_data: Value = response.json().await?;
            Ok(risk_data)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get risk snapshot failed: {}", error_text)
        }
    }

    /// Get order history with pagination and filtering
    pub async fn get_order_history(
        &self,
        params: Option<HistoryParams>,
    ) -> Result<ApiResponse<Vec<HistoricalOrder>>> {
        let mut path = "orders/api/v1/orders/history".to_string();

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
                path.push('?');
                path.push_str(&query_params.join("&"));
            }
        }

        let response = self.request(reqwest::Method::GET, &path, None).await?;

        if response.status().is_success() {
            let history_response: crate::types::HistoryResponse = response.json().await?;
            Ok(ApiResponse {
                data: history_response.orders,
                metadata: Some(ResponseMetadata {
                    total: Some(history_response.total),
                    limit: Some(history_response.limit as u32),
                    offset: Some(history_response.offset as u32),
                }),
            })
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get order history failed: {}", error_text)
        }
    }

    /// Get order history with specific filters
    pub async fn get_order_history_filtered(
        &self,
        filters: OrderHistoryFilters,
    ) -> Result<ApiResponse<Vec<HistoricalOrder>>> {
        let mut query_params = Vec::new();

        if let Some(symbol) = filters.symbol {
            query_params.push(format!("symbol={}", symbol));
        }
        if let Some(side) = filters.side {
            query_params.push(format!("side={}", side));
        }
        if let Some(status) = filters.status {
            query_params.push(format!("status={}", status));
        }
        if let Some(order_type) = filters.order_type {
            query_params.push(format!("order_type={}", order_type));
        }

        if let Some(pagination) = filters.pagination {
            if let Some(limit) = pagination.limit {
                query_params.push(format!("limit={}", limit));
            }
            if let Some(offset) = pagination.offset {
                query_params.push(format!("offset={}", offset));
            }
        }

        if let Some(date_range) = filters.date_range {
            if let Some(start) = date_range.start_time {
                query_params.push(format!("start_time={}", start.to_rfc3339()));
            }
            if let Some(end) = date_range.end_time {
                query_params.push(format!("end_time={}", end.to_rfc3339()));
            }
        }

        let path = if query_params.is_empty() {
            "orders/api/v1/orders/history".to_string()
        } else {
            format!("orders/api/v1/orders/history?{}", query_params.join("&"))
        };

        let response = self.request(reqwest::Method::GET, &path, None).await?;

        if response.status().is_success() {
            let history_response: crate::types::HistoryResponse = response.json().await?;
            Ok(ApiResponse {
                data: history_response.orders,
                metadata: Some(ResponseMetadata {
                    total: Some(history_response.total),
                    limit: Some(history_response.limit as u32),
                    offset: Some(history_response.offset as u32),
                }),
            })
        } else {
            let error_text = response.text().await.unwrap_or_default();
            bail!("Get order history filtered failed: {}", error_text)
        }
    }
}
