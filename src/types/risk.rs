//! Risk Management Types
//!
//! This module contains types for risk management and monitoring.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// User status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatus {
    pub username: String,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub permissions: Vec<String>,
    pub risk_limits: BTreeMap<String, Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSnapshot {
    pub username: String,
    pub timestamp: DateTime<Utc>,
    pub total_exposure: Decimal,
    pub margin_requirement: Decimal,
    pub available_margin: Decimal,
    pub positions: Vec<RiskPosition>,
    pub risk_metrics: RiskMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskPosition {
    pub symbol: String,
    pub quantity: Decimal,
    pub average_price: Decimal,
    pub market_value: Decimal,
    pub unrealized_pnl: Decimal,
    pub margin_requirement: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub var_1d: Decimal,
    pub var_5d: Decimal,
    pub expected_shortfall: Decimal,
    pub beta: Decimal,
    pub correlation: HashMap<String, Decimal>,
}

// Python Risk Manager types (actual response format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonRiskSnapshot {
    pub timestamp: String, // Python service returns timestamp as string
    pub username: String,
    pub positions: Vec<PythonRiskPosition>,
    pub cash_balance: Decimal,
    pub total_realized_pnl: Decimal,
    pub total_unrealized_pnl: Decimal,
    pub total_initial_margin: Decimal,
    pub total_maintenance_margin: Decimal,
    pub total_equity: Decimal,
    pub available_initial_margin: Decimal,
    pub available_maintenance_margin: Decimal,
    pub excess_liquidity: Decimal,
    pub buying_power: Decimal,
    pub leverage: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonRiskPosition {
    pub symbol: String,
    pub position: Decimal,
    pub market_price: Decimal,
    pub market_value: Decimal,
    pub average_cost: Decimal,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub initial_margin_req: Decimal,
    pub maintenance_margin_req: Decimal,
}
