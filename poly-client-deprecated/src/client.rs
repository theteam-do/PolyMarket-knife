//! Polymarket CLOB API 客户端主模块

use anyhow::{Context, Result};
use reqwest::{Client, Response, StatusCode};
use rust_decimal::Decimal;
use serde::de::DeserializeOwned;
use tracing::{debug, info, warn};

use crate::auth::{AuthConfig, AuthMiddleware};
use crate::types::*;
use crate::market::MarketClient;
use crate::order::OrderClient;
use crate::ws::WsClient;

/// Polymarket CLOB API 客户端
pub struct PolyClient {
    client: Client,
    base_url: String,
    auth: Option<AuthMiddleware>,
    pub market: MarketClient,
    pub order: OrderClient,
    pub ws: WsClient,
}

impl PolyClient {
    /// 创建未认证的客户端（只读）
    pub fn new(host: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            base_url: host.trim_end_matches('/').to_string(),
            auth: None,
            market: MarketClient::new(host),
            order: OrderClient::new(host),
            ws: WsClient::new(host),
        }
    }

    /// 创建认证客户端（可交易）
    pub fn with_auth(host: &str, config: &AuthConfig) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            base_url: host.trim_end_matches('/').to_string(),
            auth: Some(AuthMiddleware::new(config)),
            market: MarketClient::with_auth(host, config),
            order: OrderClient::with_auth(host, config),
            ws: WsClient::new(host),
        }
    }

    /// 获取订单簿
    pub async fn get_orderbook(&self, token_id: &str) -> Result<OrderBook> {
        let url = format!("{}/book?token_id={}", self.base_url, token_id);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch orderbook")?;

        self.parse_response(response).await
    }

    /// 获取市场价格
    pub async fn get_price(&self, token_id: &str) -> Result<Decimal> {
        let ob = self.get_orderbook(token_id).await?;
        ob.mid_price().context("No price available")
    }

    /// 获取多个市场的订单簿
    pub async fn get_orderbooks(&self, token_ids: &[&str]) -> Result<Vec<OrderBook>> {
        let mut results = Vec::new();
        
        for token_id in token_ids {
            match self.get_orderbook(token_id).await {
                Ok(ob) => results.push(ob),
                Err(e) => {
                    warn!("Failed to fetch orderbook for {}: {}", token_id, e);
                }
            }
        }

        Ok(results)
    }

    /// 发送 GET 请求
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.get(&url);
        
        if let Some(auth) = &self.auth {
            let headers = auth.add_headers("GET", path, None);
            for (key, value) in headers {
                req = req.header(&key, value);
            }
        }

        let response = req.send().await.context("GET request failed")?;
        self.parse_response(response).await
    }

    /// 发送 POST 请求
    async fn post<T: DeserializeOwned, B: serde::Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let body_json = serde_json::to_string(body).context("Failed to serialize body")?;
        
        let mut req = self.client
            .post(&url)
            .header("Content-Type", "application/json");
        
        if let Some(auth) = &self.auth {
            let headers = auth.add_headers("POST", path, Some(&body_json));
            for (key, value) in headers {
                req = req.header(&key, value);
            }
        }

        let response = req.body(body_json).send().await.context("POST request failed")?;
        self.parse_response(response).await
    }

    /// 解析响应
    async fn parse_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();
        
        if status.is_success() {
            response.json().await.context("Failed to parse response JSON")
        } else {
            let error_text = response.text().await.unwrap_or_default();
            if status == StatusCode::UNAUTHORIZED {
                anyhow::bail!("Authentication failed: {}", error_text);
            } else if status == StatusCode::NOT_FOUND {
                anyhow::bail!("Resource not found: {}", error_text);
            } else {
                anyhow::bail!("API error ({}): {}", status, error_text);
            }
        }
    }

    /// 检查连接状态
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl Clone for PolyClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            auth: self.auth.clone(),
            market: self.market.clone(),
            order: self.order.clone(),
            ws: self.ws.clone(),
        }
    }
}
