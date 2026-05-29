use anyhow::{Result, anyhow};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, fmt, str::FromStr};

const REQUEST_PREFIX: &str = "R-";
const QUOTE_PREFIX: &str = "Q-";

#[derive(
    Debug,
    derive_more::Display,
    derive_more::AsRef,
    derive_more::Into,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct RequestId(String);

#[derive(
    Debug,
    derive_more::Display,
    derive_more::AsRef,
    derive_more::Into,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct QuoteId(String);

impl RequestId {
    pub fn new(id: impl Into<String>) -> Result<Self> {
        let ulid = extract_ulid(&id.into(), REQUEST_PREFIX, "request")?;
        Ok(Self::from_ulid(ulid))
    }

    pub fn new_unchecked(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn generate() -> Self {
        Self::from_ulid(ulid::Ulid::new())
    }

    pub fn from_ulid(ulid: ulid::Ulid) -> Self {
        Self(format!("{REQUEST_PREFIX}{ulid}"))
    }

    pub fn validate(&self) -> Result<()> {
        validate_rfq_id(&self.0, REQUEST_PREFIX, "request")
    }

    pub fn ulid(&self) -> Result<ulid::Ulid> {
        extract_ulid(&self.0, REQUEST_PREFIX, "request")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for RequestId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl AsRef<str> for RequestId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for RequestId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl QuoteId {
    pub fn new(id: impl Into<String>) -> Result<Self> {
        let ulid = extract_ulid(&id.into(), QUOTE_PREFIX, "quote")?;
        Ok(Self::from_ulid(ulid))
    }

    pub fn new_unchecked(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn generate() -> Self {
        Self::from_ulid(ulid::Ulid::new())
    }

    pub fn from_ulid(ulid: ulid::Ulid) -> Self {
        Self(format!("{QUOTE_PREFIX}{ulid}"))
    }

    pub fn validate(&self) -> Result<()> {
        validate_rfq_id(&self.0, QUOTE_PREFIX, "quote")
    }

    pub fn ulid(&self) -> Result<ulid::Ulid> {
        extract_ulid(&self.0, QUOTE_PREFIX, "quote")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for QuoteId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl AsRef<str> for QuoteId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for QuoteId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum RfqQuoteSides {
    #[serde(rename = "B")]
    Bid,
    #[serde(rename = "A")]
    Ask,
    #[serde(rename = "BA")]
    BidAndAsk,
}

impl RfqQuoteSides {
    pub fn includes_bid(self) -> bool {
        matches!(self, Self::Bid | Self::BidAndAsk)
    }

    pub fn includes_ask(self) -> bool {
        matches!(self, Self::Ask | Self::BidAndAsk)
    }
}

impl fmt::Display for RfqQuoteSides {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Bid => "B",
            Self::Ask => "A",
            Self::BidAndAsk => "BA",
        })
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::EnumString,
    strum::Display,
    strum::IntoStaticStr,
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum RfqRequestState {
    #[strum(serialize = "ACTIVE")]
    #[serde(rename = "ACTIVE")]
    Active,
    #[strum(serialize = "SETTLING")]
    #[serde(rename = "SETTLING")]
    Settling,
    #[strum(serialize = "SETTLED")]
    #[serde(rename = "SETTLED")]
    Settled,
    #[strum(serialize = "CANCELED")]
    #[serde(rename = "CANCELED")]
    Canceled,
    #[strum(serialize = "EXPIRED")]
    #[serde(rename = "EXPIRED")]
    Expired,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::EnumString,
    strum::Display,
    strum::IntoStaticStr,
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum RfqQuoteState {
    #[strum(serialize = "ACTIVE")]
    #[serde(rename = "ACTIVE")]
    Active,
    #[strum(serialize = "ACCEPTED")]
    #[serde(rename = "ACCEPTED")]
    Accepted,
    #[strum(serialize = "CANCELED")]
    #[serde(rename = "CANCELED")]
    Canceled,
    #[strum(serialize = "EXPIRED")]
    #[serde(rename = "EXPIRED")]
    Expired,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum RfqQuote {
    BidAndAsk {
        #[serde(rename = "b")]
        bid: Decimal,
        #[serde(rename = "a")]
        ask: Decimal,
    },
    Bid {
        #[serde(rename = "b")]
        bid: Decimal,
    },
    Ask {
        #[serde(rename = "a")]
        ask: Decimal,
    },
}

impl RfqQuote {
    pub fn bid(self) -> Option<Decimal> {
        match self {
            Self::BidAndAsk { bid, .. } | Self::Bid { bid } => Some(bid),
            Self::Ask { .. } => None,
        }
    }

    pub fn ask(self) -> Option<Decimal> {
        match self {
            Self::BidAndAsk { ask, .. } | Self::Ask { ask } => Some(ask),
            Self::Bid { .. } => None,
        }
    }

    pub fn sides(self) -> RfqQuoteSides {
        match self {
            Self::BidAndAsk { .. } => RfqQuoteSides::BidAndAsk,
            Self::Bid { .. } => RfqQuoteSides::Bid,
            Self::Ask { .. } => RfqQuoteSides::Ask,
        }
    }
}

fn validate_rfq_id(id: &str, prefix: &str, label: &str) -> Result<()> {
    extract_ulid(id, prefix, label)?;
    Ok(())
}

fn extract_ulid(id: &str, prefix: &str, label: &str) -> Result<ulid::Ulid> {
    let raw = id
        .strip_prefix(prefix)
        .ok_or_else(|| anyhow!("invalid {label} ID format"))?;
    // Crockford base32 is case-insensitive; normalize so parse accepts lowercase wire values.
    let raw = raw.to_ascii_uppercase();
    ulid::Ulid::from_string(&raw).map_err(|e| anyhow!("invalid ULID in {label} ID: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_id_generate_uses_request_prefix() {
        let request_id = RequestId::generate();

        assert!(request_id.as_str().starts_with("R-"));
        assert!(request_id.ulid().is_ok());
    }

    #[test]
    fn quote_id_generate_uses_quote_prefix() {
        let quote_id = QuoteId::generate();

        assert!(quote_id.as_str().starts_with("Q-"));
        assert!(quote_id.ulid().is_ok());
    }

    #[test]
    fn request_id_formats_and_parses_prefixed_crockford_base32() {
        let id = RequestId::from_ulid(ulid::Ulid(0x1234));

        assert_eq!(id.to_string(), "R-000000000000000000000004HM");
        assert_eq!(
            "R-000000000000000000000004HM".parse::<RequestId>().unwrap(),
            id
        );
    }

    #[test]
    fn quote_id_formats_and_parses_prefixed_crockford_base32() {
        let id = QuoteId::from_ulid(ulid::Ulid(0xabcd));

        assert_eq!(id.to_string(), "Q-00000000000000000000001AYD");
        assert_eq!(
            "Q-00000000000000000000001AYD".parse::<QuoteId>().unwrap(),
            id
        );
    }

    #[test]
    fn rfq_ids_serialize_as_prefixed_crockford_base32_strings() {
        assert_eq!(
            serde_json::to_string(&RequestId::from_ulid(ulid::Ulid(0x1234))).unwrap(),
            "\"R-000000000000000000000004HM\""
        );
        assert_eq!(
            serde_json::to_string(&QuoteId::from_ulid(ulid::Ulid(0xabcd))).unwrap(),
            "\"Q-00000000000000000000001AYD\""
        );
    }

    #[test]
    fn rfq_ids_deserialize_from_prefixed_crockford_base32_strings() {
        assert_eq!(
            serde_json::from_str::<RequestId>("\"R-000000000000000000000004HM\"").unwrap(),
            RequestId::from_ulid(ulid::Ulid(0x1234))
        );
        assert_eq!(
            serde_json::from_str::<QuoteId>("\"Q-00000000000000000000001AYD\"").unwrap(),
            QuoteId::from_ulid(ulid::Ulid(0xabcd))
        );
    }

    #[test]
    fn rfq_ids_parse_lowercase_crockford_base32_strings() {
        assert_eq!(
            "R-000000000000000000000004hm".parse::<RequestId>().unwrap(),
            RequestId::from_ulid(ulid::Ulid(0x1234))
        );
        assert_eq!(
            "Q-00000000000000000000001ayd".parse::<QuoteId>().unwrap(),
            QuoteId::from_ulid(ulid::Ulid(0xabcd))
        );
    }

    #[test]
    fn rfq_ids_reject_invalid_format() {
        assert!("000000000000000000000004HM".parse::<RequestId>().is_err());
        assert!("R-000000000000000000000004HM".parse::<QuoteId>().is_err());
        assert!("Q-00000000000000000000001AYD".parse::<RequestId>().is_err());
        assert!(
            "000000-0000-0000-0000-0000-04HM"
                .parse::<RequestId>()
                .is_err()
        );
        assert!(
            "00000000-0000-0000-0000-000000001234"
                .parse::<QuoteId>()
                .is_err()
        );
    }

    #[test]
    fn rfq_quote_sides_display_matches_json_wire_tokens() {
        assert_eq!(RfqQuoteSides::Bid.to_string(), "B");
        assert_eq!(RfqQuoteSides::Ask.to_string(), "A");
        assert_eq!(RfqQuoteSides::BidAndAsk.to_string(), "BA");
        assert_eq!(
            serde_json::to_string(&RfqQuoteSides::BidAndAsk).unwrap(),
            "\"BA\""
        );
    }
}
