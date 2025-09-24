//! Settlement Protocol Types
//!
//! This module contains types used for settlement protocol communication (over-the-wire).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Settlement status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementStatus {
    pub status: String,
    pub last_settlement: Option<DateTime<Utc>>,
    pub next_settlement: Option<DateTime<Utc>>,
    pub pending_settlements: u32,
}

/// Settlement record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementRecord {
    pub id: String,
    pub username: String,
    pub symbol: String,
    pub amount: Decimal,
    pub settlement_type: String,
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub confirmation_time: Option<DateTime<Utc>>,
}
