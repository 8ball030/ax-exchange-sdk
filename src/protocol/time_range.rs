//! Time range params and responses for API endpoints.

use anyhow::{Result, bail, ensure};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, PickFirst, serde_as};

/// Maximum time window, in nanoseconds, that a historical orders query may
/// span. Queries to `GET /orders` must supply an explicit range no wider than
/// this (7 days); wider or unbounded ranges are rejected with a 400.
pub const MAX_HISTORICAL_ORDERS_WINDOW_NS: u64 = 7 * 24 * 60 * 60 * 1_000_000_000;

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
        if let (Some(start), Some(end)) = (self.start_timestamp_ns, self.end_timestamp_ns)
            && end <= start
        {
            bail!("end_timestamp_ns must be greater than start_timestamp_ns");
        }
        Ok(())
    }

    /// Require an explicit, bounded range no wider than `max_window_ns`.
    ///
    /// Both bounds must be present and `end - start` must not exceed
    /// `max_window_ns`. Unlike silently clamping, this surfaces a clear error so
    /// the caller knows exactly which window was (not) applied. Intended to gate
    /// queries to a bounded window so large lookbacks are rejected rather than
    /// served slowly.
    pub fn ensure_within_max_window(&self, max_window_ns: u64) -> Result<()> {
        let (Some(start), Some(end)) = (self.start_timestamp_ns, self.end_timestamp_ns) else {
            bail!(
                "a bounded time range is required: provide both start_timestamp_ns and end_timestamp_ns"
            );
        };
        ensure!(
            end > start,
            "end_timestamp_ns must be greater than start_timestamp_ns"
        );
        ensure!(
            end - start <= max_window_ns,
            "time range too wide: maximum {} days",
            max_window_ns / (24 * 60 * 60 * 1_000_000_000)
        );
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

    const DAY_NS: u64 = 24 * 60 * 60 * 1_000_000_000;
    const WINDOW_NS: u64 = 7 * DAY_NS;
    const NOW_NS: u64 = 100 * DAY_NS;

    #[test]
    fn ensure_window_rejects_missing_bounds() {
        for r in [
            TimeRangeNs::default(),
            TimeRangeNs {
                start_timestamp_ns: Some(NOW_NS - DAY_NS),
                end_timestamp_ns: None,
            },
            TimeRangeNs {
                start_timestamp_ns: None,
                end_timestamp_ns: Some(NOW_NS),
            },
        ] {
            let err = r.ensure_within_max_window(WINDOW_NS).unwrap_err();
            assert!(err.to_string().contains("bounded time range is required"));
        }
    }

    #[test]
    fn ensure_window_rejects_too_wide() {
        let r = TimeRangeNs {
            start_timestamp_ns: Some(NOW_NS - 8 * DAY_NS),
            end_timestamp_ns: Some(NOW_NS),
        };
        let err = r.ensure_within_max_window(WINDOW_NS).unwrap_err();
        assert_eq!(err.to_string(), "time range too wide: maximum 7 days");
    }

    #[test]
    fn ensure_window_accepts_exact_and_within() {
        let exact = TimeRangeNs {
            start_timestamp_ns: Some(NOW_NS - WINDOW_NS),
            end_timestamp_ns: Some(NOW_NS),
        };
        exact.ensure_within_max_window(WINDOW_NS).unwrap();

        let within = TimeRangeNs {
            start_timestamp_ns: Some(NOW_NS - DAY_NS),
            end_timestamp_ns: Some(NOW_NS),
        };
        within.ensure_within_max_window(WINDOW_NS).unwrap();
    }

    #[test]
    fn ensure_window_rejects_end_le_start() {
        let r = TimeRangeNs {
            start_timestamp_ns: Some(NOW_NS),
            end_timestamp_ns: Some(NOW_NS),
        };
        let err = r.ensure_within_max_window(WINDOW_NS).unwrap_err();
        assert_eq!(
            err.to_string(),
            "end_timestamp_ns must be greater than start_timestamp_ns"
        );
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
