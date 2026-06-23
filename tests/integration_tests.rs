use anyhow::Result;
use ax_exchange_sdk::{
    environment::Environment,
    protocol::order_gateway::OrderReference,
    trading::{OrderState, TimeInForce},
    ArchitectX, PlaceOrder, SelfTradeBehavior, Side,
};
use rust_decimal::Decimal;
use std::str::FromStr;

#[macro_use]
mod common;

#[tokio::test]
async fn test_instruments() -> Result<()> {
    with_private_client!(client, {
        let api = client.api_gateway()?;
        let instruments = api.get_instruments().await?;
        assert!(
            !instruments.instruments.is_empty(),
            "Expected at least one instrument"
        );

        println!("Fetched {} instruments", instruments.instruments.len());
        if let Some(first) = instruments.instruments.first() {
            println!("First instrument: {:?}", first.0.symbol);
        }

        Ok(())
    })
}

#[tokio::test]
async fn test_order_life_cycle() -> Result<()> {
    with_private_client!(client, {
        let order_ws = client.order_gateway_ws().await?;

        println!("Waiting for connection...");
        order_ws.wait_for_connection().await;
        println!("Connected to order gateway.");

        let open_orders = order_ws.get_open_orders().await?;
        println!("Currently have {} open orders.", open_orders.orders.len());

        let symbol = "XAU-PERP";
        // we now get the market price of XAU such that we can place a resting order at a reasonable price level
        let md_api = client.api_gateway()?;
        let xau_instrument = md_api
            .get_tickers()
            .await?
            .tickers
            .into_iter()
            .find(|t| t.symbol == symbol)
            .expect("XAU-PERP ticker not found");
        println!("Current XAU price: {:?}", xau_instrument.bid_price);

        let side = Side::Buy;
        let quantity = 1;
        let price =
            xau_instrument.bid_price.expect("No Price for symbol!") - Decimal::from_str("10.0")?; // place a resting order $10 below the current price

        let place_order = PlaceOrder {
            symbol: symbol.to_string(),
            side,
            quantity,
            price,
            time_in_force: TimeInForce::GoodTillCanceled, // ensure the order stays open until we cancel it
            post_only: true, // ensure the order rests and doesn't take liquidity
            tag: Some("test_order".to_string()),
            clord_id: None,
            self_trade_prevention: SelfTradeBehavior::CancelBoth,
        };

        let res = order_ws.place_order(place_order).await?;

        println!("Placed order: {:?}", res);

        let order_status = client
            .order_gateway()
            .expect("Failed to get order gateway client")
            .order_status(OrderReference::OrderId(res.order_id.clone()))
            .await;

        assert!(
            order_status.is_ok(),
            "Failed to fetch order status after placing order: {:?}",
            order_status.err()
        );

        let status = order_status.unwrap();
        assert_eq!(
            status.order_id, res.order_id,
            "Order ID in status does not match placed order"
        );

        assert!([OrderState::Accepted, OrderState::Pending].contains(&status.state),);

        println!("Order status after placing order: {:?}", status);
        // we now cancel the order immediately to trigger a cancel event as well
        let cancel_res = order_ws.cancel_order(&res.order_id).await;

        assert!(
            cancel_res.is_ok(),
            "Failed to cancel order: {:?}",
            cancel_res.err()
        );

        let cancel_accepted = cancel_res.unwrap();
        assert!(
            cancel_accepted.cancel_request_accepted,
            "Cancel request was not accepted: {:?}",
            cancel_accepted
        );

        let order_status_after_cancel = client
            .order_gateway()
            .expect("Failed to get order gateway client")
            .order_status(OrderReference::OrderId(res.order_id.clone()))
            .await?;

        assert_eq!(
            order_status_after_cancel.order_id, res.order_id,
            "Order ID in status does not match placed order after cancel"
        );

        assert_eq!(
            order_status_after_cancel.state,
            OrderState::Canceled,
            "Order state after cancel is not Canceled"
        );
        println!(
            "Order status after canceling order: {:?}",
            order_status_after_cancel
        );
        Ok(())
    })
}

// we test the endpoint for risk
#[tokio::test]
async fn test_user_risk_snapshot() -> Result<()> {
    with_private_client!(client, {
        match client.refresh_user_token(true).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to refresh user token: {:?}", e);
                return Ok(());
            }
        }
        let api = client.api_gateway()?;
        let risk_snapshot = api.get_risk_snapshot().await;
        assert!(
            risk_snapshot.is_ok(),
            "Failed to fetch risk snapshot: {:?}",
            risk_snapshot.err()
        );
        Ok(())
    })
}
