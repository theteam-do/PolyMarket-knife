use anyhow::{Context, Result};
use ethers::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct TradeEvent {
    pub from: String,
    pub market: String,
    pub market_id: String,
    pub side: Side,
    pub size_usd: Decimal,
    pub price: Decimal,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

// Polymarket CTF Exchange 合约地址
const POLYMARKET_EXCHANGE: &str = "0x4bFB41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E";

abigen!(
    PolymarketExchange,
    r#"[
        event OrderFilled(bytes32 indexed orderHash, address indexed maker, address indexed taker, uint256 assetId, uint256 makerAmount, uint256 takerAmount, uint256 feeAmount)
    ]"#,
);

pub struct ChainMonitor {
    config: Config,
}

impl ChainMonitor {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// 启动 WebSocket 订阅并将解析的交易发送到 channel
    pub async fn stream_trades(&self, tx: mpsc::Sender<TradeEvent>) -> Result<()> {
        let ws_url = self
            .config
            .polygon
            .ws_rpc_url
            .clone()
            .unwrap_or_else(|| "wss://polygon-mainnet.g.alchemy.com/v2/YOUR_API_KEY".to_string());

        info!("Connecting to Polygon WebSocket RPC: {}", ws_url);
        
        let provider = Provider::<Ws>::connect(&ws_url)
            .await
            .context("Failed to connect to Polygon WS RPC")?;
        let client = Arc::new(provider);

        let exchange_addr = Address::from_str(POLYMARKET_EXCHANGE)?;
        let contract = PolymarketExchange::new(exchange_addr, client.clone());
        let events = contract.order_filled_filter();
        
        let mut stream = events.subscribe().await?;
        
        info!("Subscribed to OrderFilled events on Polygon Exchange: {}", POLYMARKET_EXCHANGE);

        while let Some(log) = stream.next().await {
            match log {
                Ok(event) => {
                    let maker = format!("{:?}", event.maker);
                    let taker = format!("{:?}", event.taker);
                    
                    let is_smart_maker = self.config.strategy.smart_addresses.is_empty() 
                        || self.config.strategy.smart_addresses.iter().any(|a| a.eq_ignore_ascii_case(&maker));
                        
                    let is_smart_taker = self.config.strategy.smart_addresses.is_empty() 
                        || self.config.strategy.smart_addresses.iter().any(|a| a.eq_ignore_ascii_case(&taker));

                    if !is_smart_maker && !is_smart_taker {
                        continue;
                    }
                    
                    let smart_wallet = if is_smart_taker { taker } else { maker };
                    
                    // 将 uint256 转换为精确的 Decimal 以防除不尽或精度丢失
                    let maker_amt_u256 = event.maker_amount.as_u128();
                    let taker_amt_u256 = event.taker_amount.as_u128();
                    let fee_amt_u256 = event.fee_amount.as_u128();
                    
                    if maker_amt_u256 == 0 || taker_amt_u256 == 0 {
                        continue;
                    }

                    // Polymarket 的 USDC 和 CTF Position Token 均采用 6 位精度 (decimals = 6)
                    let scale = Decimal::from(1_000_000u64);
                    let maker_amt = Decimal::from(maker_amt_u256) / scale;
                    let taker_amt = Decimal::from(taker_amt_u256) / scale;
                    let fee_amt = Decimal::from(fee_amt_u256) / scale;

                    // 核心逻辑:
                    // Polymarket 的份额单价始终在 0.0001 到 0.9999 之间
                    // 这意味着：在任何撮合中，USDC 的数量必定小于 Share 的数量
                    let (usdc_amount, share_amount) = if maker_amt < taker_amt {
                        (maker_amt, taker_amt)
                    } else {
                        (taker_amt, maker_amt)
                    };

                    let price = (usdc_amount / share_amount).round_dp(4); // 通常 Tick Size 为 0.0001
                    
                    // 处理成本：把 feeAmount 加到 size_usd 的逻辑里，计算出带有磨损的成本
                    // Taker 支付 fee，如果 smart_wallet 是 Taker，则将其名义仓位价值减去手续费磨损
                    // 因为在 Polymarket，Fee 是从你得到的资产或者你付出的 USDC 中扣除的。
                    // 简化处理：实际支出成本或所得 = 基础 usdc_amount
                    // 此处我们记录带有手续费影响后的净 USD 规模：
                    let mut size_usd = usdc_amount;
                    if is_smart_taker {
                        // Taker 被扣除手续费，所以其净头寸规模会小一点
                        size_usd = size_usd - fee_amt;
                    }

                    if size_usd <= Decimal::ZERO {
                        continue;
                    }

                    // 判断买卖方向
                    // 如果 maker_amt < taker_amt，说明 Maker 付出较少的 USDC，获得较多的 Share，说明 Maker 是买方(Buy)
                    let maker_side = if maker_amt < taker_amt { Side::Buy } else { Side::Sell };
                    
                    // Taker 的方向必然与 Maker 相反
                    let smart_side = if is_smart_taker { 
                        if maker_side == Side::Buy { Side::Sell } else { Side::Buy }
                    } else { 
                        maker_side 
                    };

                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                        
                    // 修正 Token ID 格式，Polymarket CLOB API 通常期望 '0x' 前缀的 16 进制 assetId
                    let mut asset_id_hex = format!("{:x}", event.asset_id);
                    // 保证长度即使有零也不被截断，Polymarket assetId 通常是 64 字符
                    if asset_id_hex.len() < 64 {
                        asset_id_hex = format!("{:0>64}", asset_id_hex);
                    }
                    let asset_id_hex = format!("0x{}", asset_id_hex);

                    let trade_event = TradeEvent {
                        from: smart_wallet,
                        market: asset_id_hex.clone(),
                        market_id: asset_id_hex,
                        side: smart_side,
                        size_usd: size_usd.round_dp(2), // 美元规模保留两位小数
                        price,
                        timestamp,
                    };

                    info!("Decoded trade: {} {} ${:.2} of asset {} @ ${:.4} (Fee: ${:.4})", 
                        trade_event.from, 
                        if trade_event.side == Side::Buy { "BOUGHT" } else { "SOLD" },
                        trade_event.size_usd,
                        trade_event.market_id,
                        trade_event.price,
                        fee_amt
                    );

                    if tx.send(trade_event).await.is_err() {
                        warn!("Trade channel closed, stopping stream.");
                        break;
                    }
                }
                Err(e) => {
                    error!("Error receiving event: {}", e);
                }
            }
        }

        Ok(())
    }
}
