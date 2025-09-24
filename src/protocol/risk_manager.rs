//! Risk Management Protocol Types
//!
//! This module contains types used for risk management protocol communication (over-the-wire).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Admin statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminStats {
    pub total_users: u64,
    pub active_users: u64,
    pub total_volume: Decimal,
    pub total_deposits: Decimal,
    pub total_withdrawals: Decimal,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

/// Stress test scenario configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestScenario {
    pub name: String,
    pub price_shocks: HashMap<String, Decimal>, // symbol -> price shock percentage
    pub volatility_multiplier: Option<Decimal>,
}

/// Liquidation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Liquidation {
    pub symbol: String,
    pub quantity: Decimal,
    pub price: Decimal, // Python service returns price, not side/reason
}

/// Liquidation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidationSummary {
    pub username: String,
    pub liquidations: Vec<Liquidation>,
    pub total_initial_margin_pre_liquidations: Decimal,
    pub total_initial_margin_post_liquidations: Decimal,
}

/// Stress test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResult {
    pub risk_snapshot: crate::types::PythonRiskSnapshot,
    pub liquidation_summary: LiquidationSummary,
}
