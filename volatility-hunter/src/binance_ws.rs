//! 币安 WebSocket 数据源

use anyhow::{Context, Result};
use futures_util::StreamExt;
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{info, warn, error};

use crate::config::BinanceConfig;
use crate::PriceTick;

pub struct BinanceFeed {
    config: BinanceConfig,
}

impl BinanceFeed {
    pub fn new(config: &BinanceConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub async fn stream(
        &self,
        tx: mpsc::Sender<PriceTick>,
        symbols: Vec<String>,
    ) -> Result<()> {
        let streams: Vec<String> = symbols
            .iter()
            .map(|s| format!("{}@trade", s.to_lowercase()))
            .collect();
        
        let stream_path = streams.join("/");
        let url = format!("{}/{}", self.config.ws_url, stream_path);

        info!("Connecting to Binance WebSocket: {}", url);

        loop {
            match self.connect_and_stream(&url, &tx).await {
                Ok(()) => {
                    warn!("Binance connection closed, reconnecting...");
                }
                Err(e) => {
                    error!("Binance connection error: {}, reconnecting in 5s...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn connect_and_stream(&self, url: &str, tx: &mpsc::Sender<PriceTick>) -> Result<()> {
        let (ws_stream, _) = connect_async(url)
            .await
            .context("Failed to connect to Binance WebSocket")?;

        info!("Connected to Binance WebSocket");

        let (_, mut read) = ws_stream.split();

        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    if let Some(tick) = parse_trade(&text) {
                        if tx.send(tick).await.is_err() {
                            break;
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    warn!("Binance WebSocket closed");
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

fn parse_trade(text: &str) -> Option<PriceTick> {
    #[derive(Deserialize)]
    struct BinanceTrade {
        e: String,
        s: String,
        p: String,
        q: String,
        T: u64,
    }

    let trade: BinanceTrade = serde_json::from_str(text).ok()?;
    
    if trade.e != "trade" {
        return None;
    }

    let price: f64 = trade.p.parse().ok()?;
    let volume: f64 = trade.q.parse().ok()?;

    Some(PriceTick {
        symbol: trade.s,
        price,
        timestamp: trade.T,
        volume,
    })
}
