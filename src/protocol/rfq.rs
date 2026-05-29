use crate::types::Side;
pub use crate::types::{
    QuoteId, RequestId, RfqQuote, RfqQuoteSides, RfqQuoteState, RfqRequestState,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_with::{StringWithSeparator, formats::CommaSeparator, serde_as};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "t")]
pub enum RfqRequest {
    #[serde(rename = "sr")]
    SubmitQuoteRequest(SubmitQuoteRequest),
    #[serde(rename = "xr")]
    CancelQuoteRequest(CancelQuoteRequest),
    #[serde(rename = "rs")]
    GetQuoteRequests(GetQuoteRequests),
    #[serde(rename = "sq")]
    SubmitQuote(SubmitQuote),
    #[serde(rename = "xq")]
    CancelQuote(CancelQuote),
    #[serde(rename = "qs")]
    GetQuotes(GetQuotes),
    #[serde(rename = "aq")]
    AcceptQuote(AcceptQuote),
}

/// Discriminated by the `t` tag so deserialization is unambiguous and
/// stable as variants evolve. An untagged enum here would let a new
/// variant whose JSON shape is a subset of another silently steal
/// decoding from the older variant — fine in tests, catastrophic on a
/// public wire contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "t")]
pub enum RfqResponse {
    #[serde(rename = "sra")]
    SubmitQuoteRequestResponse(SubmitQuoteRequestResponse),
    #[serde(rename = "sqa")]
    SubmitQuoteResponse(SubmitQuoteResponse),
    #[serde(rename = "aqa")]
    AcceptQuoteResponse(AcceptQuoteResponse),
    #[serde(rename = "rsa")]
    GetQuoteRequestsResponse(GetQuoteRequestsResponse),
    #[serde(rename = "qsa")]
    GetQuotesResponse(GetQuotesResponse),
    #[serde(rename = "ack")]
    CancelAck(CancelAck),
    #[serde(rename = "rj")]
    Reject(RfqReject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "t")]
pub enum RfqEvent {
    #[serde(rename = "r")]
    QuoteRequestPosted(QuoteRequestPosted),
    #[serde(rename = "xr")]
    QuoteRequestRemoved(QuoteRequestRemoved),
    #[serde(rename = "q")]
    QuotePosted(Quote),
    #[serde(rename = "xq")]
    QuoteRemoved(QuoteRemoved),
    #[serde(rename = "f")]
    Filled(Filled),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SubmitQuoteRequest {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "q")]
    pub quantity: Decimal,
    #[serde(rename = "r")]
    pub requested_sides: RfqQuoteSides,
    #[serde(rename = "exp")]
    pub expiration: DateTime<Utc>,
    #[serde(rename = "cid", skip_serializing_if = "Option::is_none")]
    pub client_request_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CancelQuoteRequest {
    #[serde(rename = "rid")]
    pub request_id: RequestId,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetQuoteRequests {
    #[serde_as(as = "Option<StringWithSeparator::<CommaSeparator, RfqRequestState>>")]
    #[serde(rename = "s")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub states: Option<Vec<RfqRequestState>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SubmitQuote {
    #[serde(rename = "rid")]
    pub request_id: RequestId,
    #[serde(flatten)]
    pub quote: RfqQuote,
    #[serde(rename = "exp")]
    pub expiration: DateTime<Utc>,
    #[serde(rename = "cid", skip_serializing_if = "Option::is_none")]
    pub client_quote_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CancelQuote {
    #[serde(rename = "qid")]
    pub quote_id: QuoteId,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetQuotes {
    #[serde(rename = "rid")]
    pub request_id: RequestId,
    #[serde_as(as = "Option<StringWithSeparator::<CommaSeparator, RfqQuoteState>>")]
    #[serde(rename = "s")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub states: Option<Vec<RfqQuoteState>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct AcceptQuote {
    #[serde(rename = "qid")]
    pub quote_id: QuoteId,
    #[serde(rename = "d")]
    pub side: Side,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SubmitQuoteRequestResponse {
    #[serde(rename = "rid")]
    pub request_id: RequestId,
    #[serde(rename = "cid", skip_serializing_if = "Option::is_none")]
    pub client_request_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SubmitQuoteResponse {
    #[serde(rename = "qid")]
    pub quote_id: QuoteId,
    #[serde(rename = "cid", skip_serializing_if = "Option::is_none")]
    pub client_quote_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct AcceptQuoteResponse {
    #[serde(rename = "tid")]
    pub trade_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetQuoteRequestsResponse {
    #[serde(rename = "rs")]
    pub requests: Vec<QuoteRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetQuotesResponse {
    #[serde(rename = "qs")]
    pub quotes: Vec<Quote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CancelAck {
    #[serde(rename = "ok")]
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct RfqReject {
    #[serde(rename = "r")]
    pub reason: String,
    #[serde(rename = "txt", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// One RFQ request row in list/status IPC, public broadcast, and subscription
/// snapshots. `requester_user_id` is set for authenticated gateway responses and
/// omitted on the public stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct QuoteRequest {
    #[serde(rename = "rid")]
    pub request_id: RequestId,
    #[serde(rename = "st")]
    pub state: RfqRequestState,
    #[serde(rename = "u", skip_serializing_if = "Option::is_none")]
    pub requester_user_id: Option<String>,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "q")]
    pub quantity: Decimal,
    #[serde(rename = "r")]
    pub requested_sides: RfqQuoteSides,
    #[serde(rename = "exp")]
    pub expiration: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct QuoteRequestPosted {
    #[serde(flatten)]
    pub request: QuoteRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct QuoteRequestRemoved {
    #[serde(rename = "rid")]
    pub request_id: RequestId,
    #[serde(rename = "s")]
    pub state: RfqRequestState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Quote {
    #[serde(rename = "rid")]
    pub request_id: RequestId,
    #[serde(rename = "qid")]
    pub quote_id: QuoteId,
    #[serde(rename = "st")]
    pub state: RfqQuoteState,
    #[serde(rename = "u")]
    pub responder_user_id: String,
    #[serde(flatten)]
    pub quote: RfqQuote,
    #[serde(rename = "exp")]
    pub expiration: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct QuoteRemoved {
    #[serde(rename = "rid")]
    pub request_id: RequestId,
    #[serde(rename = "qid")]
    pub quote_id: QuoteId,
    #[serde(rename = "s")]
    pub state: RfqQuoteState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Filled {
    #[serde(rename = "rid")]
    pub request_id: RequestId,
    #[serde(rename = "qid")]
    pub quote_id: QuoteId,
    #[serde(rename = "tid")]
    pub trade_id: String,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "q")]
    pub quantity: Decimal,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "d")]
    pub accepted_side: Side,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use insta::assert_json_snapshot;

    fn request_id() -> RequestId {
        RequestId::new("R-000000000000000000000004HM").unwrap()
    }

    fn quote_id() -> QuoteId {
        QuoteId::new("Q-00000000000000000000001AYD").unwrap()
    }

    #[test]
    fn serializes_submit_quote_request() {
        let req = RfqRequest::SubmitQuoteRequest(SubmitQuoteRequest {
            symbol: "BTC-PERP".to_string(),
            quantity: "10".parse().unwrap(),
            requested_sides: RfqQuoteSides::Bid,
            expiration: Utc.with_ymd_and_hms(2026, 5, 6, 12, 0, 0).unwrap(),
            client_request_id: Some(42),
        });

        assert_json_snapshot!(req, @r#"
        {
          "t": "sr",
          "s": "BTC-PERP",
          "q": "10",
          "r": "B",
          "exp": "2026-05-06T12:00:00Z",
          "cid": 42
        }
        "#);
    }

    #[test]
    fn serializes_cancel_quote_request() {
        let req = RfqRequest::CancelQuoteRequest(CancelQuoteRequest {
            request_id: request_id(),
        });

        assert_json_snapshot!(req, @r#"
        {
          "t": "xr",
          "rid": "R-000000000000000000000004HM"
        }
        "#);
    }

    #[test]
    fn serializes_quote_request_queries() {
        assert_json_snapshot!(
            RfqRequest::GetQuoteRequests(GetQuoteRequests {
                states: None,
            }),
            @r#"
        {
          "t": "rs"
        }
        "#
        );
        assert_json_snapshot!(
            RfqRequest::GetQuoteRequests(GetQuoteRequests {
                states: Some(vec![RfqRequestState::Active, RfqRequestState::Settling]),
            }),
            @r#"
        {
          "t": "rs",
          "s": "ACTIVE,SETTLING"
        }
        "#
        );
    }

    #[test]
    fn serializes_quote_queries() {
        assert_json_snapshot!(
            RfqRequest::GetQuotes(GetQuotes {
                request_id: request_id(),
                states: None,
            }),
            @r#"
        {
          "t": "qs",
          "rid": "R-000000000000000000000004HM"
        }
        "#
        );
        assert_json_snapshot!(
            RfqRequest::GetQuotes(GetQuotes {
                request_id: request_id(),
                states: Some(vec![RfqQuoteState::Active]),
            }),
            @r#"
        {
          "t": "qs",
          "rid": "R-000000000000000000000004HM",
          "s": "ACTIVE"
        }
        "#
        );
    }

    #[test]
    fn serializes_submit_quote_variants() {
        let expiration = Utc.with_ymd_and_hms(2026, 5, 6, 12, 0, 0).unwrap();

        assert_json_snapshot!(
            RfqRequest::SubmitQuote(SubmitQuote {
                request_id: request_id(),
                quote: RfqQuote::Bid { bid: "99".parse().unwrap() },
                expiration,
                client_quote_id: None,
            }),
            @r#"
        {
          "t": "sq",
          "rid": "R-000000000000000000000004HM",
          "b": "99",
          "exp": "2026-05-06T12:00:00Z"
        }
        "#
        );

        assert_json_snapshot!(
            RfqRequest::SubmitQuote(SubmitQuote {
                request_id: request_id(),
                quote: RfqQuote::Ask { ask: "101".parse().unwrap() },
                expiration,
                client_quote_id: None,
            }),
            @r#"
        {
          "t": "sq",
          "rid": "R-000000000000000000000004HM",
          "a": "101",
          "exp": "2026-05-06T12:00:00Z"
        }
        "#
        );

        assert_json_snapshot!(
            RfqRequest::SubmitQuote(SubmitQuote {
                request_id: request_id(),
                quote: RfqQuote::BidAndAsk {
                    bid: "99".parse().unwrap(),
                    ask: "101".parse().unwrap(),
                },
                expiration,
                client_quote_id: Some(7),
            }),
            @r#"
        {
          "t": "sq",
          "rid": "R-000000000000000000000004HM",
          "b": "99",
          "a": "101",
          "exp": "2026-05-06T12:00:00Z",
          "cid": 7
        }
        "#
        );
    }

    #[test]
    fn rejects_submit_quote_without_price() {
        let raw = r#"{
          "t": "sq",
          "rid": "R-000000000000000000000004HM",
          "exp": "2026-05-06T12:00:00Z"
        }"#;

        assert!(serde_json::from_str::<RfqRequest>(raw).is_err());
    }

    #[test]
    fn serializes_rfq_events() {
        let request = QuoteRequest {
            request_id: request_id(),
            state: RfqRequestState::Active,
            requester_user_id: None,
            symbol: "BTC-PERP".to_string(),
            quantity: "10".parse().unwrap(),
            requested_sides: RfqQuoteSides::Ask,
            expiration: Utc.with_ymd_and_hms(2026, 5, 6, 12, 0, 0).unwrap(),
        };

        assert_json_snapshot!(
            RfqEvent::QuoteRequestPosted(QuoteRequestPosted { request }),
            @r#"
        {
          "t": "r",
          "rid": "R-000000000000000000000004HM",
          "st": "ACTIVE",
          "s": "BTC-PERP",
          "q": "10",
          "r": "A",
          "exp": "2026-05-06T12:00:00Z"
        }
        "#
        );

        assert_json_snapshot!(
            RfqEvent::QuoteRequestRemoved(QuoteRequestRemoved {
                request_id: request_id(),
                state: RfqRequestState::Expired,
            }),
            @r#"
        {
          "t": "xr",
          "rid": "R-000000000000000000000004HM",
          "s": "EXPIRED"
        }
        "#
        );

        assert_json_snapshot!(
            RfqEvent::QuotePosted(Quote {
                request_id: request_id(),
                quote_id: quote_id(),
                state: RfqQuoteState::Active,
                responder_user_id: "000000-0000-000H".to_string(),
                quote: RfqQuote::BidAndAsk {
                    bid: "99".parse().unwrap(),
                    ask: "101".parse().unwrap(),
                },
                expiration: Utc.with_ymd_and_hms(2026, 5, 6, 12, 0, 0).unwrap(),
            }),
            @r#"
        {
          "t": "q",
          "rid": "R-000000000000000000000004HM",
          "qid": "Q-00000000000000000000001AYD",
          "st": "ACTIVE",
          "u": "000000-0000-000H",
          "b": "99",
          "a": "101",
          "exp": "2026-05-06T12:00:00Z"
        }
        "#
        );

        assert_json_snapshot!(
            RfqEvent::QuoteRemoved(QuoteRemoved {
                request_id: request_id(),
                quote_id: quote_id(),
                state: RfqQuoteState::Canceled,
            }),
            @r#"
        {
          "t": "xq",
          "rid": "R-000000000000000000000004HM",
          "qid": "Q-00000000000000000000001AYD",
          "s": "CANCELED"
        }
        "#
        );

        assert_json_snapshot!(
            RfqEvent::Filled(Filled {
                request_id: request_id(),
                quote_id: quote_id(),
                trade_id: "T-00000000000000000000000000".to_string(),
                symbol: "BTC-PERP".to_string(),
                quantity: "10".parse().unwrap(),
                price: "101".parse().unwrap(),
                accepted_side: Side::Buy,
            }),
            @r#"
        {
          "t": "f",
          "rid": "R-000000000000000000000004HM",
          "qid": "Q-00000000000000000000001AYD",
          "tid": "T-00000000000000000000000000",
          "s": "BTC-PERP",
          "q": "10",
          "p": "101",
          "d": "B"
        }
        "#
        );
    }

    /// Every `RfqResponse` variant must round-trip through JSON unambiguously.
    /// Snapshots also pin the wire tag for each variant — changing them is a
    /// breaking change to the public protocol.
    #[test]
    fn serializes_and_round_trips_rfq_responses() {
        let cases: Vec<(RfqResponse, &str)> = vec![
            (
                RfqResponse::SubmitQuoteRequestResponse(SubmitQuoteRequestResponse {
                    request_id: request_id(),
                    client_request_id: Some(7),
                }),
                "sra",
            ),
            (
                RfqResponse::SubmitQuoteResponse(SubmitQuoteResponse {
                    quote_id: quote_id(),
                    client_quote_id: None,
                }),
                "sqa",
            ),
            (
                RfqResponse::AcceptQuoteResponse(AcceptQuoteResponse {
                    trade_id: "T-00000000000000000000000000".to_string(),
                }),
                "aqa",
            ),
            (
                RfqResponse::GetQuoteRequestsResponse(GetQuoteRequestsResponse {
                    requests: vec![],
                }),
                "rsa",
            ),
            (
                RfqResponse::GetQuotesResponse(GetQuotesResponse { quotes: vec![] }),
                "qsa",
            ),
            (RfqResponse::CancelAck(CancelAck { ok: true }), "ack"),
            (
                RfqResponse::Reject(RfqReject {
                    reason: "invalid".to_string(),
                    message: None,
                }),
                "rj",
            ),
        ];

        for (resp, expected_tag) in cases {
            let json = serde_json::to_value(&resp).expect("serialize");
            assert_eq!(
                json.get("t").and_then(|v| v.as_str()),
                Some(expected_tag),
                "wrong tag for {:?}",
                resp,
            );
            let back: RfqResponse =
                serde_json::from_value(json.clone()).expect("round-trip decode");
            // Round-trip JSON equality guarantees the discriminant resolves the
            // same variant in both directions.
            let back_json = serde_json::to_value(&back).expect("re-serialize");
            assert_eq!(
                json, back_json,
                "round-trip mismatch for tag {expected_tag}"
            );
        }
    }
}
