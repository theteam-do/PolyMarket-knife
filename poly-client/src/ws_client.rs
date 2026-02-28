//! WebSocket 客户端 - 支持自动重连和心跳

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

/// WebSocket 客户端配置
#[derive(Debug, Clone)]
pub struct WsConfig {
    pub url: String,
    pub reconnect_delay: Duration,
    pub max_reconnects: u32,
    pub heartbeat_interval: Duration,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            url: "wss://clob.polymarket.com/ws".to_string(),
            reconnect_delay: Duration::from_secs(5),
            max_reconnects: 10,
            heartbeat_interval: Duration::from_secs(30),
        }
    }
}

/// WebSocket 客户端
pub struct WsClient {
    config: WsConfig,
}

impl WsClient {
    pub fn new(config: WsConfig) -> Self {
        Self { config }
    }
    
    /// 连接并订阅
    pub async fn connect_and_subscribe(
        &self,
        channels: Vec<String>,
    ) -> Result<WebSocketStream> {
        let (ws_stream, _) = connect_async(&self.config.url)
            .await
            .context("Failed to connect to WebSocket")?;
        
        info!("Connected to WebSocket: {}", self.config.url);
        
        // 发送订阅消息
        let (_, mut write) = ws_stream.split();
        
        for channel in channels {
            let subscribe_msg = SubscribeMessage {
                r#type: "subscribe".to_string(),
                channel,
            };
            
            write
                .send(Message::Text(
                    serde_json::to_string(&subscribe_msg)?.into(),
                ))
                .await
                .context("Failed to send subscribe message")?;
        }
        
        Ok(ws_stream)
    }
    
    /// 带自动重连的数据流
    pub async fn stream_with_reconnect(
        &self,
        channels: Vec<String>,
        tx: mpsc::Sender<WsMessage>,
    ) -> Result<()> {
        let mut reconnects = 0;
        
        loop {
            match self.connect_and_stream(&channels, &tx).await {
                Ok(()) => {
                    info!("WebSocket disconnected");
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                }
            }
            
            reconnects += 1;
            if reconnects >= self.config.max_reconnects {
                return Err(anyhow::anyhow!("Max reconnects reached"));
            }
            
            // 指数退避
            let delay = self.config.reconnect_delay * (2_u32.pow(reconnects));
            info!("Reconnecting in {}s...", delay.as_secs());
            tokio::time::sleep(delay).await;
        }
    }
    
    async fn connect_and_stream(
        &self,
        channels: &[String],
        tx: &mpsc::Sender<WsMessage>,
    ) -> Result<()> {
        let mut ws_stream = self.connect_and_subscribe(channels.to_vec()).await?;
        
        let (_, mut read) = ws_stream.split();
        
        // 心跳定时器
        let mut heartbeat = tokio::time::interval(self.config.heartbeat_interval);
        
        loop {
            tokio::select! {
                msg_result = read.next() => {
                    match msg_result {
                        Some(Ok(Message::Text(text))) => {
                            if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                                if tx.send(msg).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Some(Ok(Message::Ping(data))) => {
                            let _ = read.send(Message::Pong(data)).await;
                        }
                        Some(Ok(Message::Close(_))) => {
                            warn!("WebSocket closed by server");
                            break;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
                _ = heartbeat.tick() => {
                    // 发送心跳
                    let _ = read.send(Message::Ping(vec![])).await;
                }
            }
        }
        
        Ok(())
    }
}

/// WebSocket 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub channel: Option<String>,
    pub data: Option<serde_json::Value>,
}

/// 订阅消息
#[derive(Debug, Serialize)]
pub struct SubscribeMessage {
    #[serde(rename = "type")]
    pub r#type: String,
    pub channel: String,
}

/// WebSocket 流包装器
pub type WebSocketStream = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
>;
