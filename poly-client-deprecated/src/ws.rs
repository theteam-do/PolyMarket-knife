//! WebSocket 客户端 - 实时订单簿更新

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{info, warn, error};

use crate::types::OrderBook;

/// WebSocket 客户端
#[derive(Clone)]
pub struct WsClient {
    ws_url: String,
}

impl WsClient {
    pub fn new(host: &str) -> Self {
        // WebSocket URL 通常是 HTTPS URL 替换为 wss://
        let ws_url = host
            .replace("https://", "wss://")
            .replace("http://", "ws://");
        
        Self { ws_url }
    }

    /// 订阅订单簿更新
    pub async fn subscribe_orderbook(
        &self,
        token_ids: Vec<String>,
        tx: mpsc::Sender<OrderBook>,
    ) -> Result<()> {
        let url = format!("{}/ws/orderbook", self.ws_url);
        
        info!("Connecting to WebSocket: {}", url);

        loop {
            match self.connect_and_subscribe(&url, &token_ids, &tx).await {
                Ok(()) => {
                    warn!("WebSocket connection closed, reconnecting...");
                }
                Err(e) => {
                    error!("WebSocket error: {}, reconnecting in 5s...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn connect_and_subscribe(
        &self,
        url: &str,
        token_ids: &[String],
        tx: &mpsc::Sender<OrderBook>,
    ) -> Result<()> {
        let (ws_stream, _) = connect_async(url)
            .await
            .context("Failed to connect to WebSocket")?;

        info!("✅ Connected to WebSocket");

        let (mut write, mut read) = ws_stream.split();

        // 发送订阅消息
        let subscribe_msg = SubscribeMessage {
            type_: "subscribe".to_string(),
            channel: "orderbook".to_string(),
            token_ids: token_ids.to_vec(),
        };

        write
            .send(Message::Text(serde_json::to_string(&subscribe_msg)?.into()))
            .await
            .context("Failed to send subscribe message")?;

        info!("Subscribed to {} markets", token_ids.len());

        // 接收消息
        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    if let Ok(ob) = serde_json::from_str::<OrderBook>(&text) {
                        if tx.send(ob).await.is_err() {
                            break;
                        }
                    }
                }
                Ok(Message::Ping(data)) => {
                    let _ = write.send(Message::Pong(data)).await;
                }
                Ok(Message::Pong(_)) => {}
                Ok(Message::Close(_)) => {
                    warn!("WebSocket closed by server");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct SubscribeMessage {
    #[serde(rename = "type")]
    type_: String,
    channel: String,
    token_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct WsOrderBookUpdate {
    token_id: String,
    bids: Vec<WsLevel>,
    asks: Vec<WsLevel>,
    timestamp: u64,
}

#[derive(Debug, Deserialize)]
struct WsLevel {
    price: String,
    size: String,
}
