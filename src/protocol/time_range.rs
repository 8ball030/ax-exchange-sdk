//! Time range params and responses for API endpoints.

use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
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
    /// Build from optional `DateTime<Utc>` bounds. Any bound whose nanosecond
    /// representation overflows `i64` (i.e. before 1677 or after 2262), or
    /// that lies before the UNIX epoch, is dropped to `None`.
    pub fn from_datetimes(start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> Self {
        fn to_unsigned(d: DateTime<Utc>) -> Option<u64> {
            d.timestamp_nanos_opt().and_then(|n| u64::try_from(n).ok())
        }
        Self {
            start_timestamp_ns: start.and_then(to_unsigned),
            end_timestamp_ns: end.and_then(to_unsigned),
        }
    }

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
    fn from_datetimes_drops_pre_epoch() {
        let pre_epoch: DateTime<Utc> = "1969-12-31T23:59:59Z".parse().unwrap();
        let post_epoch: DateTime<Utc> = "2024-01-01T00:00:00Z".parse().unwrap();
        let r = TimeRangeNs::from_datetimes(Some(pre_epoch), Some(post_epoch));
        assert_eq!(r.start_timestamp_ns, None);
        assert_eq!(r.end_timestamp_ns, Some(1_704_067_200_000_000_000));
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
