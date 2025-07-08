use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserTokenRequest {
    pub username: String,
    pub password: String,
    pub expiration_seconds: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserTokenResponse {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetInstrumentResponse {
    pub symbol: String,
    pub tick_size: Decimal,
    pub base_currency: String,
    pub multiplier: i32,
    pub minimum_trade_quantity: i32,
    pub description: String,
    pub product_id: String,
    pub state: String,
    pub price_scale: i32,
}
