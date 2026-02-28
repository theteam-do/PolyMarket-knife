//! Polymarket CLOB API 客户端
use serde::Serialize;

use anyhow::{Context, Result};
use reqwest::{Client, Response, StatusCode};
use tracing::{debug, error, instrument};

use super::types::*;
use super::signer::OrderSigner;

/// CLOB API 客户端
pub struct ClobClient {
    client: Client,
    host: String,
    api_key: Option<String>,
    signer: Option<OrderSigner>,
}

impl ClobClient {
    fn with_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(key) = &self.api_key {
            builder.header("X-Api-Key", key)
        } else {
            builder
        }
    }

    /// 创建新的客户端
    pub fn new(host: &str, api_key: Option<String>, private_key: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        let signer = if let Some(pk) = private_key {
            Some(OrderSigner::from_hex(&pk)?)
        } else {
            None
        };

        Ok(Self {
            client,
            host: host.trim_end_matches('/').to_string(),
            api_key,
            signer,
        })
    }

    /// 获取订单簿
    #[instrument(skip(self))]
    pub async fn get_orderbook(&self, token_id: &str) -> Result<OrderBookResponse> {
        let url = format!("{}/book?token_id={}", self.host, token_id);
        
        debug!("Fetching orderbook from: {}", url);
        
        let response = self.with_auth(self.client
            .get(&url))
            .send()
            .await
            .context("Failed to send orderbook request")?;

        self.parse_response(response).await
    }

    /// 下单
    #[instrument(skip(self))]
    pub async fn place_order(&self, request: OrderRequest) -> Result<OrderResponse> {
        let url = format!("{}/order", self.host);
        
        // 签名订单
        let signed_request = self.sign_order(request).await?;
        
        debug!("Placing order: {:?}", signed_request);
        
        let response = self.with_auth(self.client
            .post(&url))
            .json(&signed_request)
            .send()
            .await
            .context("Failed to send order request")?;

        self.parse_response(response).await
    }

    /// 取消订单
    #[instrument(skip(self))]
    pub async fn cancel_order(&self, order_id: &str) -> Result<CancelOrderResponse> {
        let url = format!("{}/cancel-order", self.host);
        
        let request = CancelOrderRequest {
            order_id: order_id.to_string(),
        };
        
        debug!("Cancelling order: {}", order_id);
        
        let response = self.with_auth(self.client
            .post(&url))
            .json(&request)
            .send()
            .await
            .context("Failed to send cancel request")?;

        self.parse_response(response).await
    }

    /// 取消所有订单
    #[instrument(skip(self))]
    pub async fn cancel_all(&self, market: Option<&str>) -> Result<Vec<String>> {
        let url = format!("{}/cancel-all", self.host);
        
        let mut request = serde_json::json!({});
        if let Some(m) = market {
            request["market"] = serde_json::json!(m);
        }
        
        debug!("Cancelling all orders for market: {:?}", market);
        
        let response = self.with_auth(self.client
            .post(&url))
            .json(&request)
            .send()
            .await
            .context("Failed to send cancel-all request")?;

        let result: serde_json::Value = response.json().await?;
        
        if let Some(ids) = result.as_array() {
            Ok(ids.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        } else {
            Ok(vec![])
        }
    }

    /// 签名订单
    async fn sign_order(&self, request: OrderRequest) -> Result<SignedOrderRequest> {
        let Some(signer) = &self.signer else {
            anyhow::bail!("Signer not configured");
        };

        // 生成订单哈希
        let order_hash = signer.hash_order(
            &request.token_id,
            &request.price.to_string(),
            &request.size.to_string(),
            match request.side {
                Side::Buy => "BUY",
                Side::Sell => "SELL",
            },
            request.nonce,
            request.expiration,
        );

        // 签名
        let signature = signer.sign_order(&order_hash)?;

        Ok(SignedOrderRequest {
            order: request,
            signature,
            signer: signer.address().to_string(),
        })
    }

    /// 解析响应
    async fn parse_response<T: serde::de::DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();
        
        if status.is_success() {
            response.json().await.context("Failed to parse response JSON")
        } else {
            let error_text = response.text().await.unwrap_or_default();
            
            if status == StatusCode::UNAUTHORIZED {
                error!("Authentication failed: {}", error_text);
                anyhow::bail!("Authentication failed");
            } else if status == StatusCode::NOT_FOUND {
                error!("Resource not found: {}", error_text);
                anyhow::bail!("Resource not found");
            } else {
                error!("API error ({}): {}", status, error_text);
                anyhow::bail!("API error ({}): {}", status, error_text);
            }
        }
    }

    /// 设置 API Key
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// 设置签名器
    pub fn with_signer(mut self, private_key: String) -> Result<Self> {
        self.signer = Some(OrderSigner::from_hex(&private_key)?);
        Ok(self)
    }
}

/// 签名后的订单请求
#[derive(Debug, Clone, Serialize)]
struct SignedOrderRequest {
    #[serde(flatten)]
    order: OrderRequest,
    signature: String,
    signer: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_client_creation() {
        let client = ClobClient::new(
            "https://testnet-clob.polymarket.com",
            Some("test_key".to_string()),
            Some("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()),
        ).unwrap();
        
        assert!(client.signer.is_some());
        assert!(client.api_key.is_some());
    }

    #[tokio::test]
    async fn test_order_request_serialization() {
        let request = OrderRequest {
            token_id: "123456".to_string(),
            price: dec!(0.50),
            size: dec!(100),
            side: Side::Buy,
            order_type: OrderType::Gtc,
            expiration: 0,
            nonce: 1234567890,
            signer: "0x...".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        
        assert!(json.contains("123456"));
        assert!(json.contains("0.50"));
        assert!(json.contains("BUY"));
    }
}
