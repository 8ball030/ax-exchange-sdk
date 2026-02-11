//! Pagination protocol types.
//!
//! This module contains pagination parameter shapes used for API protocol communication
//! (over-the-wire). It intentionally does not include server-side paging logic.

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, PickFirst};

pub const DEFAULT_PAGE_SIZE: u32 = 100;

/// Resolve optional limit/offset into concrete values, defaulting to
/// `DEFAULT_PAGE_SIZE` and `0` respectively.
pub fn resolve_limit_offset(limit: Option<u32>, offset: Option<u32>) -> (u32, u32) {
    (limit.unwrap_or(DEFAULT_PAGE_SIZE), offset.unwrap_or(0))
}

/// Limit/offset pagination parameters for API requests.
#[serde_as]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams, utoipa::ToSchema))]
#[cfg_attr(feature = "utoipa", into_params(parameter_in = Query))]
pub struct LimitOffsetPagination {
    #[serde_as(as = "Option<PickFirst<(_, DisplayFromStr)>>")]
    pub limit: Option<u32>,
    #[serde_as(as = "Option<PickFirst<(_, DisplayFromStr)>>")]
    pub offset: Option<u32>,
}

/// Cursor pagination parameters for API requests.
///
/// The `limit` field accepts a number or string on input, matching existing endpoint behavior.
#[serde_as]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams, utoipa::ToSchema))]
#[cfg_attr(feature = "utoipa", into_params(parameter_in = Query))]
pub struct CursorPagination {
    #[serde_as(as = "Option<PickFirst<(_, DisplayFromStr)>>")]
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize)]
    struct LimitOffsetQuery {
        #[serde(flatten)]
        pagination: LimitOffsetPagination,
    }

    #[derive(Debug, Deserialize)]
    struct CursorQuery {
        #[serde(flatten)]
        pagination: CursorPagination,
    }

    #[test]
    fn limit_offset_query_params_decode_as_numbers() {
        let parsed: LimitOffsetQuery = serde_urlencoded::from_str("limit=10&offset=20").unwrap();
        assert_eq!(parsed.pagination.limit, Some(10));
        assert_eq!(parsed.pagination.offset, Some(20));
    }

    #[test]
    fn limit_offset_query_params_empty_is_none() {
        let parsed: LimitOffsetQuery = serde_urlencoded::from_str("").unwrap();
        assert_eq!(parsed.pagination.limit, None);
        assert_eq!(parsed.pagination.offset, None);
    }

    #[test]
    fn limit_offset_query_params_invalid_number_errors() {
        let err = serde_urlencoded::from_str::<LimitOffsetQuery>("limit=abc").unwrap_err();
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn limit_offset_json_accepts_number_or_string() {
        let from_number: LimitOffsetPagination =
            serde_json::from_str(r#"{ "limit": 10 }"#).unwrap();
        let from_string: LimitOffsetPagination =
            serde_json::from_str(r#"{ "limit": "10" }"#).unwrap();
        assert_eq!(from_number.limit, Some(10));
        assert_eq!(from_string.limit, Some(10));
    }

    #[test]
    fn limit_offset_json_invalid_string_errors() {
        let err =
            serde_json::from_str::<LimitOffsetPagination>(r#"{ "limit": "abc" }"#).unwrap_err();
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn cursor_query_params_decode_limit_and_cursor() {
        let parsed: CursorQuery = serde_urlencoded::from_str("limit=10&cursor=abc").unwrap();
        assert_eq!(parsed.pagination.limit, Some(10));
        assert_eq!(parsed.pagination.cursor.as_deref(), Some("abc"));
    }

    #[test]
    fn cursor_query_params_empty_is_none() {
        let parsed: CursorQuery = serde_urlencoded::from_str("").unwrap();
        assert_eq!(parsed.pagination.limit, None);
        assert_eq!(parsed.pagination.cursor, None);
    }

    #[test]
    fn cursor_json_accepts_number_or_string() {
        let from_number: CursorPagination = serde_json::from_str(r#"{ "limit": 10 }"#).unwrap();
        let from_string: CursorPagination = serde_json::from_str(r#"{ "limit": "10" }"#).unwrap();
        assert_eq!(from_number.limit, Some(10));
        assert_eq!(from_string.limit, Some(10));
    }

    #[test]
    fn resolve_limit_offset_defaults() {
        let (limit, offset) = resolve_limit_offset(None, None);
        assert_eq!(limit, DEFAULT_PAGE_SIZE);
        assert_eq!(offset, 0);
    }

    #[test]
    fn resolve_limit_offset_explicit_values() {
        let (limit, offset) = resolve_limit_offset(Some(25), Some(50));
        assert_eq!(limit, 25);
        assert_eq!(offset, 50);
    }
}
