use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum FundingRateEstimateStatus {
    Ready,
    SettlementPending,
    Unavailable,
}

/// Live funding-rate estimate for a symbol (published to cache and served to clients).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct FundingRateEstimate {
    pub symbol: String,
    pub status: FundingRateEstimateStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub funding_rate: Option<Decimal>,
    pub funding_amount: Option<Decimal>,
    pub benchmark_price: Option<Decimal>,
    pub settlement_price: Option<Decimal>,
    pub timestamp: DateTime<Utc>,
}

impl FundingRateEstimate {
    pub fn unavailable(
        symbol: impl Into<String>,
        reason: impl Into<String>,
        settlement_price: Option<Decimal>,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            status: FundingRateEstimateStatus::Unavailable,
            reason: Some(reason.into()),
            funding_rate: None,
            funding_amount: None,
            benchmark_price: None,
            settlement_price,
            timestamp,
        }
    }

    pub fn settlement_pending(symbol: impl Into<String>, timestamp: DateTime<Utc>) -> Self {
        Self {
            symbol: symbol.into(),
            status: FundingRateEstimateStatus::SettlementPending,
            reason: Some("settlement is currently pending for this symbol".to_string()),
            funding_rate: None,
            funding_amount: None,
            benchmark_price: None,
            settlement_price: None,
            timestamp,
        }
    }
}
