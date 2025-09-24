//! Symbol Type
//!
//! This module contains the Symbol newtype for type safety.

use serde::{Deserialize, Serialize};

/// Strong type for Symbol to prevent mixing with other string values
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
pub struct Symbol(String);

impl<T: AsRef<str>> PartialEq<T> for Symbol {
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

impl Symbol {
    /// Create a new Symbol from a string
    pub fn new(symbol: impl Into<String>) -> Self {
        Self(symbol.into())
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
