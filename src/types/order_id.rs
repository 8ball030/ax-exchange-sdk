//! Order ID Type
//!
//! This module contains the OrderId newtype for type safety.

use serde::{Deserialize, Serialize};

/// Strong type for Order IDs to prevent mixing with other string values
#[derive(
    Default,
    Debug,
    derive_more::Display,
    derive_more::AsRef,
    derive_more::FromStr,
    derive_more::Into,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct OrderId(String);

impl<T: AsRef<str>> PartialEq<T> for OrderId {
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

impl OrderId {
    /// Create a new OrderId from a string
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into the inner string value
    pub fn into_inner(self) -> String {
        self.0
    }
}
