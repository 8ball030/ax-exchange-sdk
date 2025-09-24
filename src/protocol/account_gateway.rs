//! Account Gateway protocol types
//!
//! This module contains types used for account gateway protocol communication (over-the-wire).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request to deposit funds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositRequest {
    pub username: String,
    pub symbol: String,
    pub amount: Decimal,
}

/// Response from deposit request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositResponse {
    pub deposit_id: String,
    pub status: String,
    pub expected_confirmation_time: Option<DateTime<Utc>>,
}

/// Request to withdraw funds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawRequest {
    pub username: String,
    pub symbol: String,
    pub amount: Decimal,
}

/// Response from withdraw request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawResponse {
    pub withdrawal_id: String,
    pub status: String,
    pub expected_confirmation_time: Option<DateTime<Utc>>,
}

/// Request to liquidate positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidateRequest {
    pub username: String,
}

/// Response from liquidation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidateResponse {
    pub successful_cancellations: Vec<String>,
    pub failed_cancellations: Vec<String>,
    pub successful_liquidations: Vec<String>,
    pub failed_liquidations: Vec<String>,
}

/// Admin statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminResponse {
    pub deposits: Decimal,
    pub withdrawals: Decimal,
    pub commissions: Decimal,
    pub trading_volume: Decimal,
    pub deposits_count: i32,
    pub withdrawals_count: i32,
    pub users: Vec<String>,
    pub open_interest: Decimal,
}

/// Trading volume statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingVolumeStats {
    pub username: String,
    pub total_volume: Decimal,
    pub volume_by_symbol: HashMap<String, Decimal>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

/// Deposit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositStats {
    pub total_deposits: Decimal,
    pub deposits_count: Decimal,
}

/// Withdrawal statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalStats {
    pub total_withdrawals: Decimal,
    pub withdrawals_count: Decimal,
}
