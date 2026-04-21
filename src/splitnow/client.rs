use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use splitnow::{
    AssetId, ExchangerId, NetworkId, Order, OrderData, OrderLegStatus, OrderStatus, QuoteData,
    SplitNow, WalletDistribution,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AvailableExchanger {
    pub id: ExchangerId,
    pub name: String,
}

#[derive(Clone)]
pub struct SplitnowClient {
    inner: Arc<SplitNow>,
    api_key: Arc<String>,
    http: Client,
}

impl SplitnowClient {
    pub fn new(api_key: String) -> Result<Self> {
        let inner = SplitNow::new(api_key.clone()).context("construct SplitNow client")?;
        Ok(Self {
            inner: Arc::new(inner),
            api_key: Arc::new(api_key),
            http: Client::new(),
        })
    }

    pub async fn health(&self) -> Result<bool> {
        self.inner.get_health().await.context("splitnow get_health")
    }

    pub async fn exchangers(&self) -> Result<Vec<AvailableExchanger>> {
        let url = "https://splitnow.io/api/exchangers/";
        let response = self
            .http
            .get(url)
            .header("Content-Type", "application/json")
            .header("x-api-key", self.api_key.as_str())
            .send()
            .await
            .context("request /exchangers/")?;

        let status = response.status();
        let body = response.text().await.context("read /exchangers/ body")?;
        if !status.is_success() {
            anyhow::bail!("splitnow /exchangers/ http {status}");
        }

        parse_exchangers_response(&body).context("parse /exchangers/ response")
    }

    pub async fn quote(
        &self,
        from_amount: f64,
        from: (AssetId, NetworkId),
        to: (AssetId, NetworkId),
    ) -> Result<QuoteData> {
        self.inner
            .create_and_fetch_quote(from_amount, from.0, from.1, to.0, to.1)
            .await
            .context("splitnow create_and_fetch_quote")
    }

    pub async fn order(
        &self,
        quote_id: String,
        from_amount: f64,
        from: (AssetId, NetworkId),
        distributions: Vec<WalletDistribution>,
    ) -> Result<OrderData> {
        self.inner
            .create_and_fetch_order(quote_id, from_amount, from.0, from.1, distributions)
            .await
            .context("splitnow create_and_fetch_order")
    }

    pub async fn order_details(&self, order_id: String) -> Result<Order> {
        let url = format!("https://splitnow.io/api/orders/{order_id}");
        let response = self
            .http
            .get(&url)
            .header("Content-Type", "application/json")
            .header("x-api-key", self.api_key.as_str())
            .send()
            .await
            .with_context(|| format!("request /orders/{order_id}"))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .with_context(|| format!("read /orders/{order_id} body"))?;
        if !status.is_success() {
            anyhow::bail!("splitnow /orders/{order_id} http {status}");
        }

        parse_order_response(&body).with_context(|| format!("parse /orders/{order_id} response"))
    }
}

#[derive(Debug, Deserialize)]
struct ExchangerRecord {
    id: ExchangerId,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExchangerList {
    exchangers: Vec<ExchangerRecord>,
}

#[derive(Debug, Deserialize)]
struct WrappedResponse {
    success: bool,
    data: Option<ExchangerList>,
    error: Option<String>,
}

fn parse_exchangers_response(body: &str) -> Result<Vec<AvailableExchanger>> {
    if let Ok(wrapped) = serde_json::from_str::<WrappedResponse>(body) {
        if !wrapped.success {
            anyhow::bail!(
                "splitnow /exchangers/ api error: {}",
                wrapped.error.unwrap_or_else(|| "unknown error".into())
            );
        }
        if let Some(data) = wrapped.data {
            return Ok(map_exchangers(data.exchangers));
        }
    }

    if let Ok(list) = serde_json::from_str::<ExchangerList>(body) {
        return Ok(map_exchangers(list.exchangers));
    }

    if let Ok(items) = serde_json::from_str::<Vec<ExchangerRecord>>(body) {
        return Ok(map_exchangers(items));
    }

    anyhow::bail!("unsupported exchangers response shape")
}

fn map_exchangers(items: Vec<ExchangerRecord>) -> Vec<AvailableExchanger> {
    items
        .into_iter()
        .map(|item| AvailableExchanger {
            id: item.id,
            name: item.name.unwrap_or_else(|| format!("{:?}", item.id)),
        })
        .collect()
}

fn parse_order_response(body: &str) -> Result<Order> {
    let value: Value = serde_json::from_str(body).context("decode order json")?;

    if let Some(success) = value.get("success").and_then(Value::as_bool) {
        if !success {
            anyhow::bail!(
                "splitnow /orders/ api error: {}",
                value
                    .get("error")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown error")
            );
        }
        if let Some(data) = value.get("data") {
            return deserialize_order_value(data.clone());
        }
    }

    deserialize_order_value(value)
}

fn deserialize_order_value(mut value: Value) -> Result<Order> {
    if let Ok(order) = serde_json::from_value::<Order>(value.clone()) {
        return Ok(order);
    }

    sanitize_order_value(&mut value);
    serde_json::from_value::<Order>(value).context("unsupported order response shape")
}

fn sanitize_order_value(value: &mut Value) {
    let Some(order) = value.as_object_mut() else {
        return;
    };

    let order_status = infer_order_status(
        order.get("statusShort").and_then(Value::as_str),
        order.get("statusText").and_then(Value::as_str),
    );
    order.insert(
        "statusShort".into(),
        Value::String(order_status_short_name(order_status).into()),
    );
    order.insert(
        "statusText".into(),
        Value::String(order_status_text_name(order_status).into()),
    );
    order.insert(
        "status".into(),
        Value::String(order_api_status_name(order_status).into()),
    );

    if let Some(order_legs) = order.get_mut("orderLegs").and_then(Value::as_array_mut) {
        for leg in order_legs {
            sanitize_order_leg_value(leg);
        }
    }
}

fn sanitize_order_leg_value(value: &mut Value) {
    let Some(leg) = value.as_object_mut() else {
        return;
    };

    let leg_status = infer_order_leg_status(
        leg.get("statusShort").and_then(Value::as_str),
        leg.get("statusText").and_then(Value::as_str),
    );
    leg.insert(
        "statusShort".into(),
        Value::String(order_leg_status_short_name(leg_status).into()),
    );
    leg.insert(
        "statusText".into(),
        Value::String(order_leg_status_text_name(leg_status).into()),
    );
    leg.insert(
        "status".into(),
        Value::String(order_leg_api_status_name(leg_status).into()),
    );
}

fn infer_order_status(status_short: Option<&str>, status_text: Option<&str>) -> OrderStatus {
    match status_short.unwrap_or_default() {
        "pending" => OrderStatus::Pending,
        "sending" => OrderStatus::Sending,
        "monitoring" => OrderStatus::Monitoring,
        "expired" => OrderStatus::Expired,
        "halted" => OrderStatus::Halted,
        "failed" => OrderStatus::Failed,
        "refunded" => OrderStatus::Refunded,
        "completed" => OrderStatus::Completed,
        _ => match status_text.unwrap_or_default() {
            "Awaiting Deposit" => OrderStatus::Pending,
            "Settling Order Legs" => OrderStatus::Sending,
            "Monitoring Order Legs" => OrderStatus::Monitoring,
            "Order Expired" => OrderStatus::Expired,
            "Order Halted" => OrderStatus::Halted,
            "Order Failed" => OrderStatus::Failed,
            "Order Refunded" => OrderStatus::Refunded,
            "Order Completed" => OrderStatus::Completed,
            _ => OrderStatus::Sending,
        },
    }
}

fn order_status_short_name(status: OrderStatus) -> &'static str {
    match status {
        OrderStatus::Pending => "pending",
        OrderStatus::Sending => "sending",
        OrderStatus::Monitoring => "monitoring",
        OrderStatus::Expired => "expired",
        OrderStatus::Halted => "halted",
        OrderStatus::Failed => "failed",
        OrderStatus::Refunded => "refunded",
        OrderStatus::Completed => "completed",
    }
}

fn order_status_text_name(status: OrderStatus) -> &'static str {
    match status {
        OrderStatus::Pending => "Awaiting Deposit",
        OrderStatus::Sending => "Settling Order Legs",
        OrderStatus::Monitoring => "Monitoring Order Legs",
        OrderStatus::Expired => "Order Expired",
        OrderStatus::Halted => "Order Halted",
        OrderStatus::Failed => "Order Failed",
        OrderStatus::Refunded => "Order Refunded",
        OrderStatus::Completed => "Order Completed",
    }
}

fn order_api_status_name(status: OrderStatus) -> &'static str {
    match status {
        OrderStatus::Pending => "pending_deposit_wallet",
        OrderStatus::Sending => "settling_order_legs",
        OrderStatus::Monitoring => "monitoring",
        OrderStatus::Expired => "expired",
        OrderStatus::Halted => "halted",
        OrderStatus::Failed => "failed",
        OrderStatus::Refunded => "refunded",
        OrderStatus::Completed => "completed",
    }
}

fn infer_order_leg_status(status_short: Option<&str>, status_text: Option<&str>) -> OrderLegStatus {
    match status_short.unwrap_or_default() {
        "waiting" => OrderLegStatus::Waiting,
        "pending" => OrderLegStatus::Pending,
        "sending" => OrderLegStatus::Sending,
        "confirming" => OrderLegStatus::Confirming,
        "exchanging" => OrderLegStatus::Exchanging,
        "withdrawing" => OrderLegStatus::Withdrawing,
        "expired" => OrderLegStatus::Expired,
        "halted" => OrderLegStatus::Halted,
        "failed" => OrderLegStatus::Failed,
        "refunded" => OrderLegStatus::Refunded,
        "completed" => OrderLegStatus::Completed,
        _ => match status_text.unwrap_or_default() {
            "Awaiting Creation" => OrderLegStatus::Waiting,
            "Awaiting Deposit" => OrderLegStatus::Pending,
            "Sending Deposit" => OrderLegStatus::Sending,
            "Confirming Deposit" => OrderLegStatus::Confirming,
            "Exchanging Funds" => OrderLegStatus::Exchanging,
            "Withdrawing Funds" => OrderLegStatus::Withdrawing,
            "Order Leg Expired" => OrderLegStatus::Expired,
            "Order Leg Halted" => OrderLegStatus::Halted,
            "Order Leg Failed" => OrderLegStatus::Failed,
            "Order Leg Refunded" => OrderLegStatus::Refunded,
            "Order Leg Completed" => OrderLegStatus::Completed,
            _ => OrderLegStatus::Sending,
        },
    }
}

fn order_leg_status_short_name(status: OrderLegStatus) -> &'static str {
    match status {
        OrderLegStatus::Waiting => "waiting",
        OrderLegStatus::Pending => "pending",
        OrderLegStatus::Sending => "sending",
        OrderLegStatus::Confirming => "confirming",
        OrderLegStatus::Exchanging => "exchanging",
        OrderLegStatus::Withdrawing => "withdrawing",
        OrderLegStatus::Expired => "expired",
        OrderLegStatus::Halted => "halted",
        OrderLegStatus::Failed => "failed",
        OrderLegStatus::Refunded => "refunded",
        OrderLegStatus::Completed => "completed",
    }
}

fn order_leg_status_text_name(status: OrderLegStatus) -> &'static str {
    match status {
        OrderLegStatus::Waiting => "Awaiting Creation",
        OrderLegStatus::Pending => "Awaiting Deposit",
        OrderLegStatus::Sending => "Sending Deposit",
        OrderLegStatus::Confirming => "Confirming Deposit",
        OrderLegStatus::Exchanging => "Exchanging Funds",
        OrderLegStatus::Withdrawing => "Withdrawing Funds",
        OrderLegStatus::Expired => "Order Leg Expired",
        OrderLegStatus::Halted => "Order Leg Halted",
        OrderLegStatus::Failed => "Order Leg Failed",
        OrderLegStatus::Refunded => "Order Leg Refunded",
        OrderLegStatus::Completed => "Order Leg Completed",
    }
}

fn order_leg_api_status_name(status: OrderLegStatus) -> &'static str {
    match status {
        OrderLegStatus::Waiting => "waiting",
        OrderLegStatus::Pending => "pending",
        OrderLegStatus::Sending => "sending_to_provider_deposit",
        OrderLegStatus::Confirming => "provider_deposit_detected",
        OrderLegStatus::Exchanging => "provider_deposit_confirmed",
        OrderLegStatus::Withdrawing => "provider_exchange_confirmed",
        OrderLegStatus::Expired => "expired",
        OrderLegStatus::Halted => "halted",
        OrderLegStatus::Failed => "failed",
        OrderLegStatus::Refunded => "refunded",
        OrderLegStatus::Completed => "completed",
    }
}

#[cfg(test)]
mod tests {
    use super::parse_order_response;

    #[test]
    fn parse_order_response_tolerates_unknown_backend_status_names() {
        let body = r#"{
          "success": true,
          "data": {
            "_id": "69e6ba2a721b455f519bafbe",
            "status": "creating_order_legs_finalizing",
            "statusShort": "sending",
            "statusText": "Settling Order Legs",
            "type": "floating_rate",
            "shortId": "LIV07Y",
            "userId": null,
            "apiKeyId": null,
            "quoteId": "quote_123",
            "orderInput": {
              "fromAmount": 1.0,
              "fromAssetId": "sol",
              "fromNetworkId": "solana"
            },
            "orderOutputs": [{
              "toDistributionId": 0,
              "toAddress": "0xabc",
              "toPctBips": 10000,
              "toAmount": 1.0,
              "toAssetId": "eth",
              "toNetworkId": "ethereum",
              "toExchangerId": "binance"
            }],
            "orderLegs": [{
              "status": "provider_withdrawal_broadcast",
              "statusShort": "withdrawing",
              "statusText": "Withdrawing Funds",
              "type": "floating_rate",
              "orderId": "LIV07Y",
              "orderLegInput": {
                "fromAmount": 1.0,
                "fromAssetId": "sol",
                "fromNetworkId": "solana"
              },
              "orderLegOutput": {
                "toDistributionId": 0,
                "toAddress": "0xabc",
                "toPctBips": 10000,
                "toAmount": 1.0,
                "toAssetId": "eth",
                "toNetworkId": "ethereum",
                "toExchangerId": "binance"
              },
              "createdAt": null,
              "updatedAt": null
            }],
            "expiredAt": null,
            "createdAt": null,
            "updatedAt": null,
            "depositWalletAddress": "deposit",
            "depositAmount": 1.0
          }
        }"#;

        let order = parse_order_response(body).expect("order should parse");
        assert_eq!(order.short_id, "LIV07Y");
        assert_eq!(format!("{:?}", order.status_short), "Sending");
        assert_eq!(order.status_text.to_string(), "Settling Order Legs");
        assert_eq!(order.order_legs.len(), 1);
    }
}
