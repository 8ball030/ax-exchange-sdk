//! Order gateway clients

pub mod rest_client;
pub mod ws_client;

pub use rest_client::OrderGatewayRestClient;
pub use ws_client::OrderGatewayWsClient;
