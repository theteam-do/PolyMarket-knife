//! 订单管理客户端

use anyhow::{Context, Result};
use reqwest::Client;
use rust_decimal::Decimal;
use tracing::{info, warn};

use crate::auth::{AuthConfig, AuthMiddleware};
use crate::types::*;

/// 订单管理客户端
#[derive(Clone)]
pub struct OrderClient {
    client: Client,
    base_url: String,
    auth: Option<AuthMiddleware>,
}

impl OrderClient {
    pub fn new(host: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            base_url: host.trim_end_matches('/').to_string(),
            auth: None,
        }
    }

    pub fn with_auth(host: &str, config: &AuthConfig) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            base_url: host.trim_end_matches('/').to_string(),
            auth: Some(AuthMiddleware::new(config)),
        }
    }

    /// 下买单
    pub async fn buy(&self, token_id: &str, price: Decimal, size: Decimal) -> Result<OrderResponse> {
        self.place_order(token_id, price, size, Side::Buy, OrderType::Gtc).await
    }

    /// 下卖单
    pub async fn sell(&self, token_id: &str, price: Decimal, size: Decimal) -> Result<OrderResponse> {
        self.place_order(token_id, price, size, Side::Sell, OrderType::Gtc).await
    }

    /// 下单
    pub async fn place_order(
        &self,
        token_id: &str,
        price: Decimal,
        size: Decimal,
        side: Side,
        order_type: OrderType,
    ) -> Result<OrderResponse> {
        let auth = self.auth.as_ref().context("Authentication required for placing orders")?;

        let order_req = OrderRequest {
            token_id: token_id.to_string(),
            price,
            size,
            side,
            order_type,
            expiration: 0, // GTC
        };

        let path = "/order";
        let body_json = serde_json::to_string(&order_req).context("Failed to serialize order")?;
        
        let url = format!("{}{}", self.base_url, path);
        let headers = auth.add_headers("POST", path, Some(&body_json));

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("POLY-API-KEY", &headers[0].1)
            .header("POLY-API-SIGNATURE", &headers[1].1)
            .header("POLY-API-TIMESTAMP", &headers[2].1)
            .body(body_json)
            .send()
            .await
            .context("Failed to send order")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Order failed: {}", error_text);
        }

        let order_resp: OrderResponse = response
            .json()
            .await
            .context("Failed to parse order response")?;

        info!(
            "Order placed: {} {} {} @ {} (ID: {})",
            order_req.side, order_req.size, order_req.token_id, order_req.price, order_resp.order_id
        );

        Ok(order_resp)
    }

    /// 取消订单
    pub async fn cancel_order(&self, order_id: &str) -> Result<CancelOrderResponse> {
        let auth = self.auth.as_ref().context("Authentication required for cancelling orders")?;

        let cancel_req = CancelOrderRequest {
            order_id: order_id.to_string(),
        };

        let path = "/cancel-order";
        let body_json = serde_json::to_string(&cancel_req).context("Failed to serialize cancel request")?;
        
        let url = format!("{}{}", self.base_url, path);
        let headers = auth.add_headers("POST", path, Some(&body_json));

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("POLY-API-KEY", &headers[0].1)
            .header("POLY-API-SIGNATURE", &headers[1].1)
            .header("POLY-API-TIMESTAMP", &headers[2].1)
            .body(body_json)
            .send()
            .await
            .context("Failed to send cancel request")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Cancel failed: {}", error_text);
        }

        let cancel_resp: CancelOrderResponse = response
            .json()
            .await
            .context("Failed to parse cancel response")?;

        info!("Order cancelled: {}", order_id);
        Ok(cancel_resp)
    }

    /// 取消所有订单
    pub async fn cancel_all(&self) -> Result<Vec<String>> {
        let orders = self.get_orders(None).await?;
        if orders.is_empty() {
            return Ok(vec![]);
        }

        let mut cancelled = Vec::new();
        for order in orders {
            match self.cancel_order(&order.order_id).await {
                Ok(resp) if resp.success => cancelled.push(resp.order_id),
                Ok(resp) => warn!("cancel_all: order {} cancel returned false", resp.order_id),
                Err(e) => warn!("cancel_all: failed to cancel {}: {}", order.order_id, e),
            }
        }

        Ok(cancelled)
    }

    /// 获取用户订单
    pub async fn get_orders(&self, market: Option<&str>) -> Result<Vec<OrderResponse>> {
        let mut url = format!("{}/orders", self.base_url);
        
        if let Some(m) = market {
            url = format!("{}?market={}", url, m);
        }

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch orders")?;

        if !response.status().is_success() {
            anyhow::bail!("Get orders failed: {}", response.status());
        }

        let orders: Vec<OrderResponse> = response
            .json()
            .await
            .context("Failed to parse orders response")?;

        Ok(orders)
    }

    /// 获取用户持仓
    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        let url = format!("{}/positions", self.base_url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch positions")?;

        if !response.status().is_success() {
            anyhow::bail!("Get positions failed: {}", response.status());
        }

        let positions: Vec<Position> = response
            .json()
            .await
            .context("Failed to parse positions response")?;

        Ok(positions)
    }

    /// 获取交易记录
    pub async fn get_trades(&self, limit: u32) -> Result<Vec<Trade>> {
        let url = format!("{}/trades?limit={}", self.base_url, limit);

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch trades")?;

        if !response.status().is_success() {
            anyhow::bail!("Get trades failed: {}", response.status());
        }

        let trades: Vec<Trade> = response
            .json()
            .await
            .context("Failed to parse trades response")?;

        Ok(trades)
    }
}
