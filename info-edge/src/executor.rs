//! 交易执行器 - 使用官方 SDK

use anyhow::{Context, Result};
use alloy::signers::local::LocalSigner;
use alloy::signers::Signer;
use polymarket_client_sdk::clob::types::request::OrdersRequest;
use polymarket_client_sdk::clob::types::{Amount, OrderType, Side as SdkSide};
use polymarket_client_sdk::clob::{Client, Config as SdkConfig};
use polymarket_client_sdk::types::{Decimal, U256};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use rust_decimal_macros::dec;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{info, instrument};

use crate::config::Config;
use crate::nlp::Direction;
use crate::signal::Signal;

/// 交易执行器
pub struct Executor {
    client: Arc<Client>,
    signer: LocalSigner,
    config: Config,
}

impl Executor {
    /// 创建新的执行器
    pub async fn new(config: &Config) -> Result<Self> {
        let private_key = std::env::var(PRIVATE_KEY_VAR)
            .context("Need POLYMARKET_PRIVATE_KEY environment variable")?;
        
        let signer = LocalSigner::from_str(&private_key)?
            .with_chain_id(Some(POLYGON));

        let sdk_config = SdkConfig::builder()
            .use_server_time(true)
            .build();

        let client = Client::new(&config.clob.host, sdk_config)?
            .authentication_builder(&signer)
            .authenticate()
            .await
            .context("Failed to authenticate")?;

        Ok(Self {
            client: Arc::new(client),
            signer,
            config: config.clone(),
        })
    }

    /// 执行交易信号
    #[instrument(skip(self), fields(signal = ?signal))]
    pub async fn execute(&self, signal: &Signal) -> Result<()> {
        let position = self.calculate_position(signal.confidence);
        
        info!(
            "Executing order: direction={:?} market={} confidence={:.2} position=${}",
            signal.direction,
            signal.market,
            signal.confidence,
            position
        );

        let token_id = self.map_market_to_token_id(&signal.market)?;
        
        let side = match signal.direction {
            Direction::Yes => SdkSide::Buy,
            Direction::No => SdkSide::Sell,
            Direction::Neutral => return Ok(()),
        };

        // 创建限价单
        let order = self.client
            .limit_order()
            .token_id(token_id)
            .order_type(OrderType::GTC)
            .price(dec!(0.50))  // 示例价格，实际需要获取市场价格
            .size(position)
            .side(side)
            .build()
            .await?;

        // 签名订单
        let signed_order = self.client.sign(&self.signer, order).await?;
        
        // 提交订单
        let resp = self.client.post_order(signed_order).await?;
        
        info!("Order placed: order_id={} success={}", resp.order_id, resp.success);
        
        Ok(())
    }

    fn map_market_to_token_id(&self, market: &str) -> Result<U256> {
        if let Ok(id) = U256::from_str(market) {
            return Ok(id);
        }

        let mut acc: u128 = 0;
        for b in market.as_bytes() {
            acc = acc.wrapping_mul(131).wrapping_add(*b as u128);
        }

        if acc == 0 {
            anyhow::bail!("unable to derive token_id from market");
        }

        Ok(U256::from(acc))
    }

    /// 计算仓位大小
    fn calculate_position(&self, confidence: f64) -> Decimal {
        let base = Decimal::from_f64_retain(self.config.strategy.max_position_usd)
            .unwrap_or(dec!(1000));
        
        // 高置信度用大仓位，低置信度用小仓位
        if confidence >= self.config.strategy.confidence_threshold {
            base
        } else {
            base * dec!(0.3)
        }
    }

    /// 获取当前订单
    pub async fn get_orders(&self) -> Result<Vec<String>> {
        let request = OrdersRequest::default();
        let page = self.client.orders(&request, None).await?;
        
        Ok(page.data.iter().map(|o| o.order_id.to_string()).collect())
    }

    /// 取消订单
    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        use uuid::Uuid;
        let order_uuid = Uuid::parse_str(order_id)?;
        self.client.cancel_order(order_uuid).await?;
        info!("Order cancelled: {}", order_id);
        Ok(())
    }

    /// 取消所有订单
    pub async fn cancel_all(&self) -> Result<()> {
        self.client.cancel_all(None).await?;
        info!("All orders cancelled");
        Ok(())
    }
}
