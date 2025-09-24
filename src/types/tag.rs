//! Tag Type
//!
//! This module contains the Tag newtype with validation.

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    /// Regex for validating tag format - alphanumeric and underscores only
    static ref TAG_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_]+$").expect("Invalid tag regex");
}

/// Strong type for Tag with validation
#[derive(
    Default,
    Debug,
    derive_more::Display,
    derive_more::AsRef,
    derive_more::FromStr,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct Tag(String);

impl<T: AsRef<str>> PartialEq<T> for Tag {
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

impl Tag {
    /// Create a new Tag with validation
    pub fn new(tag: impl Into<String>) -> Result<Self, String> {
        let tag = tag.into();

        if tag.is_empty() {
            return Err("Tag cannot be empty".to_string());
        }

        if tag.len() > 50 {
            return Err("Tag cannot be longer than 50 characters".to_string());
        }

        // Use compiled regex for validation
        if !TAG_REGEX.is_match(&tag) {
            return Err("Tag contains invalid characters".to_string());
        }

        Ok(Self(tag))
    }

    /// Create without validation (for internal use)
    pub fn new_unchecked(tag: impl Into<String>) -> Self {
        Self(tag.into())
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
