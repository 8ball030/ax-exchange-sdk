//! EP3 Types
//!
//! This module contains strong type wrappers for EP3-specific data.

// TODO: don't expose EP3 types to the public

use serde::{Deserialize, Serialize};
use std::fmt;

/// Strong type for EP3 Account to prevent mixing with other string values
#[derive(
    Default,
    Debug,
    derive_more::Display,
    derive_more::AsRef,
    derive_more::From,
    derive_more::FromStr,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
)]
#[as_ref(forward)]
pub struct Ep3Account(String);

impl Ep3Account {
    /// Create a new Ep3Account from a string
    pub fn new(account: impl Into<String>) -> Self {
        Self(account.into())
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

impl From<&str> for Ep3Account {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// EP3 Username that can handle both structured and simple formats
/// This is the main EP3 username type used throughout the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Ep3Username {
    /// Username with firm and user components (e.g., firms/CHI/users/ADX.USER.971)
    WithFirm {
        firm_id: String, // e.g., "CHI"
        user_id: String, // e.g., "ADX.USER.971"
    },
    /// Simple username without firm (e.g., "admin")
    NoFirm(String),
}

impl Ep3Username {
    /// Create username with firm from components
    pub fn with_firm(firm_id: impl Into<String>, user_id: impl Into<String>) -> Self {
        Self::WithFirm {
            firm_id: firm_id.into(),
            user_id: user_id.into(),
        }
    }

    /// Create username with firm from components (alias for with_firm for compatibility)
    pub fn new(firm_id: impl Into<String>, user_id: impl Into<String>) -> Self {
        Self::with_firm(firm_id, user_id)
    }

    /// Create simple username without firm
    pub fn simple(username: impl Into<String>) -> Self {
        Self::NoFirm(username.into())
    }

    /// Create from full EP3 username path or simple username
    pub fn from_full_path(full_path: impl AsRef<str>) -> Result<Self, String> {
        let path = full_path.as_ref();
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() == 4 && parts[0] == "firms" && parts[2] == "users" {
            // WithFirm format: firms/CHI/users/ADX.USER.971
            Ok(Self::new(parts[1], parts[3]))
        } else if parts.len() == 1 {
            // NoFirm format: admin
            Ok(Self::simple(path))
        } else {
            Err(format!("Invalid EP3 username format: {}", path))
        }
    }

    /// Get the firm ID (if WithFirm, or None for NoFirm)
    pub fn firm_id(&self) -> Option<&str> {
        match self {
            Self::WithFirm { firm_id, .. } => Some(firm_id),
            Self::NoFirm(_) => None,
        }
    }

    /// Get the user ID (if WithFirm) or the simple username (if NoFirm)
    pub fn user_id(&self) -> &str {
        match self {
            Self::WithFirm { user_id, .. } => user_id,
            Self::NoFirm(username) => username,
        }
    }

    /// Get the full EP3 username path or simple username
    pub fn full_path(&self) -> String {
        match self {
            Self::WithFirm { firm_id, user_id } => format!("firms/{}/users/{}", firm_id, user_id),
            Self::NoFirm(username) => username.clone(),
        }
    }

    /// Extract firm as Ep3Firm (uses default firm for NoFirm usernames)
    pub fn extract_firm(&self) -> Ep3Firm {
        match self {
            Self::WithFirm { firm_id, .. } => Ep3Firm::new(firm_id.clone()),
            Self::NoFirm(_) => {
                // For NoFirm usernames like "admin", use a default firm or extract from environment
                let default_firm =
                    std::env::var("EP3_PARTICIPANT_FIRM_ID").unwrap_or_else(|_| "CHI".to_string());
                Ep3Firm::new(default_firm)
            }
        }
    }

    /// Convert to owned String
    pub fn into_string(self) -> String {
        self.full_path()
    }

    /// Check if this username has firm information
    pub fn has_firm(&self) -> bool {
        matches!(self, Self::WithFirm { .. })
    }

    /// Check if this is a simple username without firm
    pub fn is_simple(&self) -> bool {
        matches!(self, Self::NoFirm(_))
    }
}

impl fmt::Display for Ep3Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_path())
    }
}

impl Default for Ep3Username {
    fn default() -> Self {
        Self::NoFirm(String::new())
    }
}

impl From<String> for Ep3Username {
    fn from(s: String) -> Self {
        Self::from_full_path(s).expect("Invalid EP3 username format")
    }
}

impl From<&str> for Ep3Username {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

/// Structured EP3 Firm to avoid string parsing
#[derive(
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
pub struct Ep3Firm {
    pub firm_id: String, // e.g., "CHI"
}

impl<T: AsRef<str>> PartialEq<T> for Ep3Firm {
    fn eq(&self, other: &T) -> bool {
        self.firm_id == other.as_ref()
    }
}

impl Ep3Firm {
    /// Create from firm ID (e.g., "CHI")
    pub fn new(firm_id: impl Into<String>) -> Self {
        Self {
            firm_id: firm_id.into(),
        }
    }

    /// Create from full firm path (e.g., "firms/CHI")
    pub fn from_full_path(full_path: impl AsRef<str>) -> Result<Self, String> {
        let path = full_path.as_ref();
        if let Some(firm_id) = path.strip_prefix("firms/") {
            Ok(Self::new(firm_id.to_string()))
        } else {
            Err(format!("Invalid firm path format: {}", path))
        }
    }

    /// Create from EP3 username (extracts firm from structured username or uses default for simple)
    pub fn from_ep3_username(ep3_username: &Ep3Username) -> Self {
        ep3_username.extract_firm()
    }

    /// Get the firm ID (e.g., "CHI")
    pub fn firm_id(&self) -> &str {
        &self.firm_id
    }

    /// Get the full firm path (e.g., "firms/CHI")
    pub fn full_path(&self) -> String {
        format!("firms/{}", self.firm_id)
    }

    /// Get the inner string value for backward compatibility
    pub fn as_str(&self) -> &str {
        &self.firm_id
    }

    /// Convert to owned String
    pub fn into_string(self) -> String {
        self.firm_id
    }
}

impl From<String> for Ep3Firm {
    fn from(s: String) -> Self {
        // Assume it's a firm ID if it doesn't contain "/"
        if s.contains('/') {
            Self::from_full_path(s).expect("Invalid firm path")
        } else {
            Self::new(s)
        }
    }
}

impl From<&str> for Ep3Firm {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}
