use crate::{
    protocol::{
        common::{Fill, Timestamp},
        pagination::{TimeseriesPage, TimeseriesPagination},
        ws,
    },
    types::{ClientOrderId, Order, OrderId, OrderRejectReason, OrderState, Side},
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator};

/// Query parameters for the order gateway WebSocket endpoint (`/ws`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct WsQueryParams {
    /// When true, all orders placed on this connection will be cancelled on disconnect.
    #[serde(default)]
    pub cancel_on_disconnect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(tag = "t")]
pub enum OrderGatewayRequest {
    #[serde(rename = "X")]
    CancelAllOrders(CancelAllOrdersRequest),
    #[serde(rename = "x")]
    CancelOrder(CancelOrderRequest),
    #[serde(rename = "s")]
    GetOrderStatus(GetOrderStatusRequest),
    #[serde(rename = "o")]
    GetOpenOrders(GetOpenOrdersRequest),
    #[serde(rename = "p")]
    PlaceOrder(PlaceOrderRequest),
    #[serde(rename = "r")]
    ReplaceOrder(ReplaceOrderRequest),
}

/// Request types for the admin firehose websocket endpoint (/admin/ws)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(tag = "t")]
pub enum AdminFirehoseRequest {
    #[serde(rename = "s")]
    Subscribe(AdminSubscribeRequest),
}

#[derive(Debug)]
#[repr(u8)]
pub enum OrderGatewayRequestType {
    CancelAllOrders,
    CancelOrder,
    GetOrderStatus,
    GetOpenOrders,
    PlaceOrder,
    ReplaceOrder,
}

/// Expected response types from the order gateway.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum OrderGatewayResponse {
    CancelAllOrdersResponse(CancelAllOrdersResponse),
    CancelOrderResponse(CancelOrderResponse),
    GetOrderStatusResponse(GetOrderStatusResponse),
    GetOpenOrdersResponse(GetOpenOrdersResponse),
    LoginResponse(LoginResponse),
    PlaceOrderResponse(PlaceOrderResponse),
    ReplaceOrderResponse(ReplaceOrderResponse),
}

/// Expected response types from the admin firehose endpoint.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum AdminFirehoseResponse {
    AdminLoginResponse(AdminLoginResponse),
    AdminSubscribeResponse(AdminSubscribeResponse),
}

/// Expected message types from the order gateway.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum OrderGatewayMessage {
    Event(OrderGatewayEvent),
    Response(ws::Response<OrderGatewayResponse>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct LoginResponse {
    #[serde(rename = "li")]
    pub logged_in: String,
    #[serde(rename = "o")]
    pub open_orders: Option<Vec<OrderDetails>>,
}

impl LoginResponse {
    pub fn into_open_orders(self) -> Result<Vec<Order>> {
        let mut oos = vec![];
        if let Some(orders) = self.open_orders {
            for order in orders {
                oos.push(order.try_into()?);
            }
        }
        Ok(oos)
    }
}

/// Login response for admin firehose websocket endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AdminLoginResponse {
    #[serde(rename = "li")]
    pub logged_in: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PlaceOrderRequest {
    /// Order symbol; e.g. GBPUSD-PERP, EURUSD-PERP
    #[serde(rename = "s")]
    pub symbol: String,
    /// Order side; buying ("B") or selling ("S")
    #[serde(rename = "d")]
    pub side: Side,
    /// Order quantity in contracts; e.g. 100, 1000
    #[serde(rename = "q")]
    pub quantity: u64,
    /// Order price in USD as decimal string; e.g. "1.2345"
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Order time in force; e.g. "GTC", "IOC".
    /// "DAY" is accepted but deprecated and will be removed in a future release — use "GTC" instead.
    #[serde(rename = "tif")]
    pub time_in_force: String,
    /// Whether the order is post-only (maker-or-cancel)
    #[serde(rename = "po")]
    pub post_only: bool,
    /// Optional order tag; maximum 10 alphanumeric characters
    #[serde(rename = "tag", skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Optional client order ID; 64 bit integer
    #[serde(rename = "cid", skip_serializing_if = "Option::is_none")]
    pub clord_id: Option<ClientOrderId>,
    /// Self-trade prevention behavior (defaults to rejecting the incoming aggressor)
    #[serde(rename = "st", default)]
    pub self_trade_prevention: crate::types::SelfTradeBehavior,
}

impl PlaceOrderRequest {
    /// Convert this place order request into a pending order
    pub fn into_pending_order(self, order_id: OrderId, user_id: String) -> crate::types::Order {
        crate::types::Order {
            order_id,
            user_id,
            symbol: self.symbol,
            side: self.side,
            quantity: self.quantity,
            price: self.price,
            time_in_force: self.time_in_force,
            tag: self.tag,
            clord_id: self.clord_id,
            post_only: self.post_only,
            timestamp: Utc::now(),
            order_state: OrderState::Pending,
            filled_quantity: 0,
            remaining_quantity: self.quantity,
            completion_time: None,
            reject_reason: None,
            reject_message: None,
        }
    }
}

impl From<crate::types::PlaceOrder> for PlaceOrderRequest {
    fn from(value: crate::types::PlaceOrder) -> Self {
        Self {
            symbol: value.symbol,
            side: value.side,
            quantity: value.quantity,
            price: value.price,
            time_in_force: value.time_in_force,
            post_only: value.post_only,
            tag: value.tag,
            clord_id: value.clord_id,
            self_trade_prevention: value.self_trade_prevention,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PlaceOrderResponse {
    /// Order ID of the placed order; e.g. "ORD-1234567890"
    #[serde(rename = "oid")]
    pub order_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct InitialMarginRequirementResponse {
    /// Initial margin percentage for the order symbol
    #[serde(rename = "im_pct")]
    pub initial_margin_percentage: Decimal,
    /// Initial margin requirement for the order; e.g. "1000.00"
    #[serde(rename = "im")]
    pub initial_margin_requirement: Decimal,
    /// Current signed position in the order symbol
    #[serde(rename = "pos")]
    pub signed_position: i64,
    /// Multiplier for the order symbol
    #[serde(rename = "mult")]
    pub contract_multiplier: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PreviewOrderResponse {
    /// Initial margin percentage for the instrument (e.g. 10 means 10% IM)
    #[serde(rename = "im_pct")]
    pub initial_margin_pct_required: Decimal,
    /// Additional initial margin required to place this order; zero if the
    /// order would reduce the overall margin requirement (e.g. a closing trade)
    #[serde(rename = "im")]
    pub initial_margin_required: Decimal,
    /// Current signed position in the symbol before the order fills
    #[serde(rename = "pos_before")]
    pub signed_position_before: i64,
    /// Projected signed position in the symbol after the order fills
    #[serde(rename = "pos_after")]
    pub signed_position_after: i64,
    /// Estimated liquidation price after the order fills, based on current
    /// equity and maintenance margin; None if the resulting position is flat
    #[serde(rename = "liq")]
    pub estimated_liquidation_price: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CancelOrderRequest {
    /// Identifier of the order to cancel; either `oid` (server order id) or
    /// `cid` (client order id).
    #[serde(flatten)]
    pub order: OrderReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CancelOrderResponse {
    /// Whether the cancel request has been accepted; e.g. true, false
    #[serde(rename = "cxl_rx")]
    pub cancel_request_accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ReplaceOrderRequest {
    /// Identifier of the order to replace; either `oid` (server order id) or
    /// `cid` (client order id).
    #[serde(flatten)]
    pub order: OrderReference,
    /// New price for the replacement order (optional, inherits from original if not provided)
    #[serde(rename = "p", skip_serializing_if = "Option::is_none")]
    pub price: Option<Decimal>,
    /// New quantity for the replacement order (optional, inherits from original if not provided)
    #[serde(rename = "q", skip_serializing_if = "Option::is_none")]
    pub quantity: Option<u64>,
    /// New time in force for the replacement order (optional, inherits from original if not provided)
    #[serde(rename = "tif", skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    /// Whether the replacement order is post-only (optional, inherits from original if not provided)
    #[serde(rename = "po", skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ReplaceOrderResponse {
    /// Order ID of the new replacement order; e.g. "ORD-1234567890"
    #[serde(rename = "oid")]
    pub order_id: OrderId,
}

/// Request to cancel all orders for the authenticated user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema, utoipa::IntoParams))]
pub struct CancelAllOrdersRequest {
    /// Optional symbol filter. If provided, only orders for this symbol will be canceled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

/// Response for canceling all orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CancelAllOrdersResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema, utoipa::IntoParams))]
pub struct GetOpenOrdersRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetOpenOrdersResponse {
    pub orders: Vec<OrderDetails>,
}

/// Admin-only request to subscribe to firehose events for all users
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct AdminSubscribeRequest {
    /// Subscribe to all fills (includes partial fills and full fills)
    #[serde(rename = "f", default)]
    pub fills: bool,
    /// Subscribe to all order state changes (acks, cancels, rejects, expires, etc.)
    #[serde(rename = "o", default)]
    pub orders: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct AdminSubscribeResponse {
    /// Confirmation message
    #[serde(rename = "msg")]
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(tag = "t")]
pub enum OrderGatewayEvent {
    // TODO: deprecate in favor of WS native ping
    #[serde(rename = "h")]
    Heartbeat(Timestamp),
    #[serde(rename = "e")]
    CancelRejected(CancelRejected),
    #[serde(rename = "n")]
    OrderAcked(OrderAcked),
    #[serde(rename = "c")]
    OrderCanceled(OrderCanceled),
    #[serde(rename = "r")]
    OrderReplacedOrAmended(OrderReplacedOrAmended),
    #[serde(rename = "j")]
    OrderRejected(OrderRejected),
    #[serde(rename = "x")]
    OrderExpired(OrderExpired),
    #[serde(rename = "d")]
    OrderDoneForDay(OrderDoneForDay),
    #[serde(rename = "p")]
    OrderPartiallyFilled(OrderPartiallyFilled),
    #[serde(rename = "f")]
    OrderFilled(OrderFilled),
}

impl OrderGatewayEvent {
    /// Order ID that the event pertains to, if applicable and singular.
    pub fn order_id(&self) -> Option<&OrderId> {
        match self {
            OrderGatewayEvent::Heartbeat(..) => None,
            OrderGatewayEvent::CancelRejected(rej) => Some(&rej.order_id),
            OrderGatewayEvent::OrderAcked(ack) => Some(&ack.order.order_id),
            OrderGatewayEvent::OrderCanceled(ccl) => Some(&ccl.order.order_id),
            OrderGatewayEvent::OrderReplacedOrAmended(roa) => Some(&roa.replaced_order.order_id),
            OrderGatewayEvent::OrderRejected(rej) => Some(&rej.order.order_id),
            OrderGatewayEvent::OrderExpired(exp) => Some(&exp.order.order_id),
            OrderGatewayEvent::OrderDoneForDay(done) => Some(&done.order.order_id),
            OrderGatewayEvent::OrderPartiallyFilled(fill) => Some(&fill.order.order_id),
            OrderGatewayEvent::OrderFilled(fill) => Some(&fill.order.order_id),
        }
    }

    /// Symbol that the event pertains to, if applicable and singular.
    pub fn symbol(&self) -> Option<&str> {
        match self {
            OrderGatewayEvent::Heartbeat(..) => None,
            OrderGatewayEvent::CancelRejected(..) => None,
            OrderGatewayEvent::OrderAcked(ack) => Some(&ack.order.symbol),
            OrderGatewayEvent::OrderCanceled(ccl) => Some(&ccl.order.symbol),
            OrderGatewayEvent::OrderReplacedOrAmended(roa) => Some(&roa.replaced_order.symbol),
            OrderGatewayEvent::OrderRejected(rej) => Some(&rej.order.symbol),
            OrderGatewayEvent::OrderExpired(exp) => Some(&exp.order.symbol),
            OrderGatewayEvent::OrderDoneForDay(done) => Some(&done.order.symbol),
            OrderGatewayEvent::OrderPartiallyFilled(fill) => Some(&fill.order.symbol),
            OrderGatewayEvent::OrderFilled(fill) => Some(&fill.order.symbol),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct CancelRejected {
    #[serde(flatten)]
    pub timestamp: Timestamp,
    #[serde(rename = "oid")]
    pub order_id: OrderId,
    #[serde(rename = "cid", skip_serializing_if = "Option::is_none")]
    pub clord_id: Option<ClientOrderId>,
    #[serde(rename = "r")]
    pub reject_reason: String,
    #[serde(rename = "txt")]
    pub reject_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderAcked {
    #[serde(flatten)]
    pub timestamp: Timestamp,
    #[serde(rename = "eid")]
    pub execution_id: String,
    #[serde(rename = "o")]
    pub order: OrderDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderCanceled {
    #[serde(flatten)]
    pub timestamp: Timestamp,
    #[serde(rename = "eid")]
    pub execution_id: String,
    #[serde(rename = "o")]
    pub order: OrderDetails,
    #[serde(rename = "xr")]
    pub cancel_reason: String,
    #[serde(rename = "txt")]
    pub cancel_message: String,
}

/// Event emitted when an order is replaced (cancel-replace) or amended.
///
/// The `replaced_order` field contains the **old** (replaced) order in its
/// terminal `Replaced` state.  The `replacement_order_id` contains the ID
/// of the **new** order that supersedes it (if this was a cancel-replace
/// rather than an in-place amend), and `replacement_order` contains its
/// full details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderReplacedOrAmended {
    #[serde(flatten)]
    pub timestamp: Timestamp,
    #[serde(rename = "eid")]
    pub execution_id: String,
    /// The old (replaced) order, now in terminal `Replaced` state.
    #[serde(rename = "ro")]
    pub replaced_order: OrderDetails,
    /// The new replacement order's ID, if this was a cancel-replace.
    #[serde(rename = "noid", skip_serializing_if = "Option::is_none")]
    pub replacement_order_id: Option<OrderId>,
    /// The new replacement order's details, if this was a cancel-replace.
    #[serde(rename = "no", skip_serializing_if = "Option::is_none")]
    pub replacement_order: Option<OrderDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderRejected {
    #[serde(flatten)]
    pub timestamp: Timestamp,
    #[serde(rename = "eid")]
    pub execution_id: String,
    #[serde(rename = "o")]
    pub order: OrderDetails,
    #[serde(rename = "r")]
    pub reject_reason: Option<OrderRejectReason>,
    #[serde(rename = "txt")]
    pub reject_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderExpired {
    #[serde(flatten)]
    pub timestamp: Timestamp,
    #[serde(rename = "eid")]
    pub execution_id: String,
    #[serde(rename = "o")]
    pub order: OrderDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderDoneForDay {
    #[serde(flatten)]
    pub timestamp: Timestamp,
    #[serde(rename = "eid")]
    pub execution_id: String,
    #[serde(rename = "o")]
    pub order: OrderDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderPartiallyFilled {
    #[serde(flatten)]
    pub timestamp: Timestamp,
    #[serde(rename = "eid")]
    pub execution_id: String,
    #[serde(rename = "o")]
    pub order: OrderDetails,
    // TODO: retag as "x"
    #[serde(rename = "xs")]
    pub fill: FillDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderFilled {
    #[serde(flatten)]
    pub timestamp: Timestamp,
    #[serde(rename = "eid")]
    pub execution_id: String,
    #[serde(rename = "o")]
    pub order: OrderDetails,
    // TODO: retag as "x"
    #[serde(rename = "xs")]
    pub fill: FillDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct OrderDetails {
    #[serde(rename = "oid")]
    pub order_id: OrderId,
    #[serde(rename = "u")]
    pub user_id: String,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "q")]
    pub quantity: u64,
    #[serde(rename = "xq")]
    pub filled_quantity: u64,
    #[serde(rename = "rq")]
    pub remaining_quantity: u64,
    #[serde(rename = "o")]
    pub order_state: OrderState,
    #[serde(rename = "d")]
    pub side: Side,
    #[serde(rename = "tif")]
    pub time_in_force: String,
    #[serde(rename = "cid", skip_serializing_if = "Option::is_none")]
    pub clord_id: Option<ClientOrderId>,
    #[serde(rename = "tag", skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(rename = "po", default)]
    pub post_only: bool,
    #[serde(rename = "r", skip_serializing_if = "Option::is_none")]
    pub reject_reason: Option<OrderRejectReason>,
    #[serde(rename = "txt", skip_serializing_if = "Option::is_none")]
    pub reject_message: Option<String>,
    #[serde(flatten)]
    pub timestamp: Timestamp,
}

impl TryFrom<OrderDetails> for crate::types::Order {
    type Error = anyhow::Error;

    fn try_from(value: OrderDetails) -> Result<Self, Self::Error> {
        Ok(crate::types::Order {
            order_id: value.order_id,
            user_id: value.user_id,
            symbol: value.symbol,
            price: value.price,
            quantity: value.quantity,
            filled_quantity: value.filled_quantity,
            remaining_quantity: value.remaining_quantity,
            order_state: value.order_state,
            side: value.side,
            time_in_force: value.time_in_force,
            tag: value.tag,
            clord_id: value.clord_id,
            post_only: value.post_only,
            timestamp: value
                .timestamp
                .as_datetime()
                .ok_or_else(|| anyhow!("invalid timestamp"))?,
            completion_time: None,
            reject_reason: value.reject_reason,
            reject_message: value.reject_message,
        })
    }
}

impl From<crate::types::Order> for OrderDetails {
    fn from(value: crate::types::Order) -> Self {
        Self {
            order_id: value.order_id,
            user_id: value.user_id,
            symbol: value.symbol,
            price: value.price,
            quantity: value.quantity,
            filled_quantity: value.filled_quantity,
            remaining_quantity: value.remaining_quantity,
            order_state: value.order_state,
            side: value.side,
            time_in_force: value.time_in_force,
            tag: value.tag,
            clord_id: value.clord_id,
            post_only: value.post_only,
            reject_reason: value.reject_reason,
            reject_message: value.reject_message,
            timestamp: Timestamp {
                ts: value.timestamp.timestamp() as i32,
                tn: value.timestamp.timestamp_subsec_nanos(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct FillDetails {
    #[serde(rename = "tid")]
    pub trade_id: String,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "q")]
    pub quantity: u64,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "d")]
    pub side: String,
    #[serde(rename = "agg")]
    pub is_taker: bool,
}

/// Order history query filters
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema, utoipa::IntoParams))]
pub struct GetOrdersRequest {
    pub symbol: Option<String>,
    #[serde(flatten)]
    pub timeseries: TimeseriesPagination,
    /// Optional comma-separated order state filter, e.g. `FILLED,CANCELED,REPLACED`
    #[serde_as(as = "Option<StringWithSeparator::<CommaSeparator, OrderState>>")]
    #[serde(default)]
    pub order_states: Option<Vec<OrderState>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetOrdersResponse {
    pub orders: Vec<OrderDetails>,
    #[serde(flatten)]
    pub page: TimeseriesPage,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum OrderReference {
    #[serde(rename = "oid")]
    OrderId(OrderId),
    #[serde(rename = "cid")]
    ClientOrderId(ClientOrderId),
}

impl<'de> Deserialize<'de> for OrderReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            #[serde(default)]
            oid: Option<OrderId>,
            #[serde(default)]
            cid: Option<ClientOrderId>,
        }
        let h = Helper::deserialize(deserializer)?;
        match (h.oid, h.cid) {
            (Some(oid), None) => Ok(OrderReference::OrderId(oid)),
            (None, Some(cid)) => Ok(cid.into()),
            (Some(_), Some(_)) => Err(serde::de::Error::custom(
                "oid and cid are mutually exclusive; provide exactly one",
            )),
            (None, None) => Err(serde::de::Error::custom(
                "exactly one of oid or cid must be provided",
            )),
        }
    }
}

impl From<OrderId> for OrderReference {
    fn from(id: OrderId) -> Self {
        OrderReference::OrderId(id)
    }
}

impl From<ClientOrderId> for OrderReference {
    fn from(id: ClientOrderId) -> Self {
        OrderReference::ClientOrderId(id)
    }
}

/// Query an order's current status by server order ID or client order ID.
///
/// `Serialize` and `Deserialize` are implemented manually rather than via
/// `#[serde(flatten)]` so that the request round-trips through both JSON
/// (`reqwest::json`) and form-urlencoded (`reqwest::query`/`axum::Query`).
/// `serde(flatten)` routes deserialization through serde's intermediate
/// `Content` map, which does not coerce form-encoded strings into the
/// numeric `u64` inside `ClientOrderId`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetOrderStatusRequest {
    /// Identifier of the order; either `oid` (server order id) or `cid` (client order id).
    pub order: OrderReference,
}

impl Serialize for GetOrderStatusRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut m = serializer.serialize_map(Some(1))?;
        match &self.order {
            OrderReference::OrderId(oid) => m.serialize_entry("oid", oid)?,
            OrderReference::ClientOrderId(cid) => m.serialize_entry("cid", &cid.0)?,
        }
        m.end()
    }
}

impl<'de> Deserialize<'de> for GetOrderStatusRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            #[serde(default)]
            oid: Option<OrderId>,
            #[serde(default)]
            cid: Option<ClientOrderId>,
        }
        let h = Helper::deserialize(deserializer)?;
        let order = match (h.oid, h.cid) {
            (Some(oid), None) => OrderReference::OrderId(oid),
            (None, Some(cid)) => cid.into(),
            (Some(_), Some(_)) => {
                return Err(serde::de::Error::custom(
                    "oid and cid are mutually exclusive; provide exactly one",
                ));
            }
            (None, None) => {
                return Err(serde::de::Error::custom(
                    "exactly one of oid or cid must be provided",
                ));
            }
        };
        Ok(Self { order })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema, utoipa::IntoParams))]
pub struct OrderStatus {
    pub symbol: String,
    pub order_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clord_id: Option<ClientOrderId>,
    pub state: OrderState,
    // TODO: should we have default values for these?
    pub filled_quantity: Option<u64>,
    pub remaining_quantity: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject_reason: Option<OrderRejectReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema, utoipa::IntoParams))]
pub struct GetOrderStatusResponse {
    pub status: OrderStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema, utoipa::IntoParams))]
pub struct GetOrderFillsRequest {
    pub order_id: OrderId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema, utoipa::IntoParams))]
pub struct GetOrderFillsResponse {
    pub fills: Vec<Fill>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SelfTradeBehavior;
    use insta::assert_json_snapshot;

    #[test]
    fn place_order_request_with_stp() {
        let req = PlaceOrderRequest {
            symbol: "EURUSD-PERP".to_string(),
            side: Side::Buy,
            quantity: 100,
            price: "1.2345".parse().unwrap(),
            time_in_force: "GTC".to_string(),
            post_only: false,
            tag: None,
            clord_id: None,
            self_trade_prevention: SelfTradeBehavior::CancelIncoming,
        };
        assert_json_snapshot!(req, @r#"
        {
          "s": "EURUSD-PERP",
          "d": "B",
          "q": 100,
          "p": "1.2345",
          "tif": "GTC",
          "po": false,
          "st": "CancelIncoming"
        }
        "#);
    }

    #[test]
    fn place_order_request_default_stp() {
        let req = PlaceOrderRequest {
            symbol: "EURUSD-PERP".to_string(),
            side: Side::Buy,
            quantity: 100,
            price: "1.2345".parse().unwrap(),
            time_in_force: "GTC".to_string(),
            post_only: false,
            tag: None,
            clord_id: None,
            self_trade_prevention: SelfTradeBehavior::default(),
        };
        assert_json_snapshot!(req, @r#"
        {
          "s": "EURUSD-PERP",
          "d": "B",
          "q": 100,
          "p": "1.2345",
          "tif": "GTC",
          "po": false,
          "st": "CancelIncoming"
        }
        "#);
    }

    fn deser_place_order_with_stp(st_value: &str) -> PlaceOrderRequest {
        let json = format!(
            r#"{{"s":"TEST","d":"B","q":1,"p":"1.00","tif":"GTC","po":false,"st":"{st_value}"}}"#
        );
        serde_json::from_str(&json).unwrap_or_else(|e| panic!("failed to deser st={st_value}: {e}"))
    }

    #[test]
    fn place_order_request_stp_alias_xi() {
        assert_eq!(
            deser_place_order_with_stp("xi").self_trade_prevention,
            SelfTradeBehavior::CancelIncoming
        );
    }

    #[test]
    fn place_order_request_stp_alias_xr() {
        assert_eq!(
            deser_place_order_with_stp("xr").self_trade_prevention,
            SelfTradeBehavior::CancelResting
        );
    }

    #[test]
    fn place_order_request_stp_alias_xb() {
        assert_eq!(
            deser_place_order_with_stp("xb").self_trade_prevention,
            SelfTradeBehavior::CancelBoth
        );
    }

    #[test]
    fn place_order_request_stp_full_names() {
        assert_eq!(
            deser_place_order_with_stp("CancelIncoming").self_trade_prevention,
            SelfTradeBehavior::CancelIncoming
        );
        assert_eq!(
            deser_place_order_with_stp("CancelResting").self_trade_prevention,
            SelfTradeBehavior::CancelResting
        );
        assert_eq!(
            deser_place_order_with_stp("CancelBoth").self_trade_prevention,
            SelfTradeBehavior::CancelBoth
        );
    }

    #[test]
    fn place_order_request_stp_omitted_defaults() {
        let json = r#"{"s":"TEST","d":"B","q":1,"p":"1.00","tif":"GTC","po":false}"#;
        let req: PlaceOrderRequest = serde_json::from_str(json).expect("deser without st");
        assert_eq!(req.self_trade_prevention, SelfTradeBehavior::CancelIncoming);
    }

    #[test]
    fn order_reference_serialization() {
        assert_json_snapshot!(
            OrderReference::from(OrderId::new_unchecked("ORD-12345")), @r#"
        {
          "oid": "ORD-12345"
        }
        "#
        );
        assert_json_snapshot!(OrderReference::from(ClientOrderId(42)), @r#"
        {
          "cid": 42
        }
        "#);
    }

    #[test]
    fn order_status_request_serialization() {
        let request_with_order_id = GetOrderStatusRequest {
            order: OrderId::new_unchecked("O-12345").into(),
        };
        let request_with_client_id = GetOrderStatusRequest {
            order: ClientOrderId(42).into(),
        };

        assert_json_snapshot!(request_with_order_id, @r#"
        {
          "oid": "O-12345"
        }
        "#);
        assert_json_snapshot!(request_with_client_id, @r#"
        {
          "cid": 42
        }
        "#);
    }

    #[test]
    fn order_status_request_urlencoded_deserialization() {
        let by_oid: GetOrderStatusRequest =
            serde_urlencoded::from_str("oid=O-12345").expect("urldecode by oid");
        assert!(matches!(by_oid.order, OrderReference::OrderId(_)));
        let by_cid: GetOrderStatusRequest =
            serde_urlencoded::from_str("cid=42").expect("urldecode by cid");
        assert!(matches!(by_cid.order, OrderReference::ClientOrderId(_)));
    }

    #[test]
    fn order_status_request_urlencoded_deserialization_rejects_both() {
        let res: Result<GetOrderStatusRequest, _> =
            serde_urlencoded::from_str("oid=O-12345&cid=42");
        assert!(res.is_err());
    }

    #[test]
    fn order_status_request_urlencoded_deserialization_rejects_neither() {
        let res: Result<GetOrderStatusRequest, _> = serde_urlencoded::from_str("");
        assert!(res.is_err());
    }

    #[test]
    fn order_status_request_urlencoded_serialization() {
        let by_oid = GetOrderStatusRequest {
            order: OrderId::new_unchecked("O-12345").into(),
        };
        let by_cid = GetOrderStatusRequest {
            order: ClientOrderId(42).into(),
        };
        assert_eq!(
            serde_urlencoded::to_string(&by_oid).expect("urlencode by oid"),
            "oid=O-12345",
        );
        assert_eq!(
            serde_urlencoded::to_string(&by_cid).expect("urlencode by cid"),
            "cid=42",
        );
    }

    #[test]
    fn order_status_request_deserialization() {
        let json_oid = r#"{"oid": "O-12345"}"#;
        let json_cid = r#"{"cid": 42}"#;

        let parsed: GetOrderStatusRequest = serde_json::from_str(json_oid).expect("parse with oid");
        assert_json_snapshot!(parsed, @r#"
        {
          "oid": "O-12345"
        }
        "#);

        let parsed: GetOrderStatusRequest = serde_json::from_str(json_cid).expect("parse with cid");
        assert_json_snapshot!(parsed, @r#"
        {
          "cid": 42
        }
        "#);
    }

    #[test]
    fn cancel_order_request_serialization() {
        let by_oid = CancelOrderRequest {
            order: OrderId::new_unchecked("O-12345").into(),
        };
        let by_cid = CancelOrderRequest {
            order: ClientOrderId(42).into(),
        };
        assert_json_snapshot!(by_oid, @r#"
        {
          "oid": "O-12345"
        }
        "#);
        assert_json_snapshot!(by_cid, @r#"
        {
          "cid": 42
        }
        "#);
    }

    #[test]
    fn cancel_order_request_deserialization() {
        let json_oid = r#"{"oid": "O-12345"}"#;
        let json_cid = r#"{"cid": 42}"#;

        let parsed: CancelOrderRequest = serde_json::from_str(json_oid).expect("parse with oid");
        assert_json_snapshot!(parsed, @r#"
        {
          "oid": "O-12345"
        }
        "#);

        let parsed: CancelOrderRequest = serde_json::from_str(json_cid).expect("parse with cid");
        assert_json_snapshot!(parsed, @r#"
        {
          "cid": 42
        }
        "#);
    }

    #[test]
    fn cancel_order_request_missing_identifier_fails() {
        let json = r#"{}"#;
        assert!(serde_json::from_str::<CancelOrderRequest>(json).is_err());
    }

    #[test]
    fn cancel_order_request_both_identifiers_fails() {
        let json = r#"{"oid": "O-12345", "cid": 42}"#;
        let err = serde_json::from_str::<CancelOrderRequest>(json).unwrap_err();
        assert!(
            err.to_string().contains("mutually exclusive"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn replace_order_request_both_identifiers_fails() {
        let json = r#"{"oid": "O-12345", "cid": 42, "p": "100.50"}"#;
        let err = serde_json::from_str::<ReplaceOrderRequest>(json).unwrap_err();
        assert!(
            err.to_string().contains("mutually exclusive"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn replace_order_request_missing_identifier_fails() {
        let json = r#"{"p": "100.50"}"#;
        assert!(serde_json::from_str::<ReplaceOrderRequest>(json).is_err());
    }

    #[test]
    fn replace_order_request_serialization() {
        let by_oid = ReplaceOrderRequest {
            order: OrderId::new_unchecked("O-12345").into(),
            price: Some("100.50".parse().unwrap()),
            quantity: None,
            time_in_force: None,
            post_only: None,
        };
        let by_cid = ReplaceOrderRequest {
            order: ClientOrderId(99).into(),
            price: Some("100.50".parse().unwrap()),
            quantity: None,
            time_in_force: None,
            post_only: None,
        };
        assert_json_snapshot!(by_oid, @r#"
        {
          "oid": "O-12345",
          "p": "100.50"
        }
        "#);
        assert_json_snapshot!(by_cid, @r#"
        {
          "cid": 99,
          "p": "100.50"
        }
        "#);
    }

    #[test]
    fn cancel_all_orders_request_serialization() {
        assert_json_snapshot!(
            CancelAllOrdersRequest { symbol: Some("TEST-PERP".to_string()) },
            @r#"
        {
          "symbol": "TEST-PERP"
        }
        "#
        );
        assert_json_snapshot!(
            CancelAllOrdersRequest { symbol: None },
            @"{}"
        );
    }

    #[test]
    fn cancel_all_orders_ws_request_serialization() {
        let wrapped = ws::Request {
            request_id: 7,
            request: OrderGatewayRequest::CancelAllOrders(CancelAllOrdersRequest {
                symbol: Some("EURUSD-PERP".to_string()),
            }),
        };
        assert_json_snapshot!(wrapped, @r#"
        {
          "rid": 7,
          "t": "X",
          "symbol": "EURUSD-PERP"
        }
        "#);

        let wrapped_no_symbol = ws::Request {
            request_id: 8,
            request: OrderGatewayRequest::CancelAllOrders(CancelAllOrdersRequest { symbol: None }),
        };
        assert_json_snapshot!(wrapped_no_symbol, @r#"
        {
          "rid": 8,
          "t": "X"
        }
        "#);
    }

    #[test]
    fn test_get_orders_request_serialization() {
        use crate::protocol::{
            pagination::CursorPagination, sort::SortDirection, time_range::TimeRangeNs,
        };
        // 2024-01-01T00:00:00Z = 1_704_067_200_000_000_000
        // 2024-01-31T23:59:59Z = 1_706_745_599_000_000_000
        let request = GetOrdersRequest {
            symbol: Some("BTCUSD-PERP".to_string()),
            timeseries: TimeseriesPagination {
                range: TimeRangeNs::from_datetimes(
                    Some("2024-01-01T00:00:00Z".parse().unwrap()),
                    Some("2024-01-31T23:59:59Z".parse().unwrap()),
                ),
                sort_ts: Some(SortDirection::Desc),
                pagination: CursorPagination {
                    limit: Some(100),
                    cursor: Some("1704067200000000000:ORD-1".to_string()),
                },
            },
            order_states: Some(vec![OrderState::Filled]),
        };
        assert_json_snapshot!(request, @r#"
        {
          "symbol": "BTCUSD-PERP",
          "start_timestamp_ns": 1704067200000000000,
          "end_timestamp_ns": 1706745599000000000,
          "sort_ts": "desc",
          "limit": 100,
          "cursor": "1704067200000000000:ORD-1",
          "order_states": "FILLED"
        }
        "#);
    }
}
