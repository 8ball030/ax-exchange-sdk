//! Time range params and responses for API endpoints.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, PickFirst};

/// Time range params for API endpoints.
///
/// Both fields are optional nanosecond timestamps (UNIX epoch):
/// - `start_timestamp_ns`: inclusive lower bound
/// - `end_timestamp_ns`: exclusive upper bound
///
/// Leave either field unset to query from the start (-∞) or through the end (+∞).
#[serde_as]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams, utoipa::ToSchema))]
#[cfg_attr(feature = "utoipa", into_params(parameter_in = Query))]
pub struct TimeRangeNs {
    #[serde_as(as = "Option<PickFirst<(_, DisplayFromStr)>>")]
    pub start_timestamp_ns: Option<u64>,
    #[serde_as(as = "Option<PickFirst<(_, DisplayFromStr)>>")]
    pub end_timestamp_ns: Option<u64>,
}

impl TimeRangeNs {
    pub fn validate(&self) -> Result<()> {
        if let (Some(start), Some(end)) = (self.start_timestamp_ns, self.end_timestamp_ns) {
            if end <= start {
                bail!("end_timestamp_ns must be greater than start_timestamp_ns");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize)]
    struct TimeRangeQuery {
        #[serde(flatten)]
        range: TimeRangeNs,
    }

    #[test]
    fn time_range_query_params_decode_as_numbers() {
        let parsed: TimeRangeQuery =
            serde_urlencoded::from_str("start_timestamp_ns=10&end_timestamp_ns=20").unwrap();
        assert_eq!(parsed.range.start_timestamp_ns, Some(10));
        assert_eq!(parsed.range.end_timestamp_ns, Some(20));
    }

    #[test]
    fn time_range_query_params_empty_is_none() {
        let parsed: TimeRangeQuery = serde_urlencoded::from_str("").unwrap();
        assert_eq!(parsed.range.start_timestamp_ns, None);
        assert_eq!(parsed.range.end_timestamp_ns, None);
    }

    #[test]
    fn validate_allows_missing_bounds() {
        let ok = TimeRangeNs {
            start_timestamp_ns: Some(10),
            end_timestamp_ns: None,
        };
        ok.validate().unwrap();
    }

    #[test]
    fn validate_rejects_end_le_start() {
        let err = TimeRangeNs {
            start_timestamp_ns: Some(10),
            end_timestamp_ns: Some(10),
        }
        .validate()
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            "end_timestamp_ns must be greater than start_timestamp_ns"
        );
    }
}
