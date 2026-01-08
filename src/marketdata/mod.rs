use crate::protocol::marketdata_publisher::{L1BookUpdate, L2BookUpdate, L3BookUpdate};
pub mod ws_client;
pub use ws_client::MarketdataWsClient;

#[derive(Debug, Clone)]
/// Convenience wrapper around the possible book updates
/// to treat these as a single type.
pub enum BookUpdate {
    L1(L1BookUpdate),
    L2(L2BookUpdate),
    L3(L3BookUpdate),
}
