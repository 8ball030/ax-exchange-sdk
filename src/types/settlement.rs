//! Settlement Types
//!
//! This module contains types for settlement operations.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Settlement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementConfig {
    pub auto_settlement: bool,
    pub settlement_frequency: String,
    pub minimum_amount: Decimal,
    pub fee_percentage: Decimal,
    pub supported_currencies: Vec<String>,
}
