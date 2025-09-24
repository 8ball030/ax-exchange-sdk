use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Candle query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleParams {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}
