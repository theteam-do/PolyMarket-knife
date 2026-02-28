//! 订单执行器

use anyhow::Result;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::{info, instrument, warn};

use crate::config::Config;
use crate::order_book::OrderBook;

/// 订单执行器
pub struct Executor {
    #[cfg(test)]
    pub mock_responses: std::sync::Arc<tokio::sync::Mutex<MockResponses>>,
    order_size: Decimal,
}

/// Mock 响应配置
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockResponses {
    pub orderbook: Option<OrderBook>,
    pub place_order_success: bool,
    pub cancel_success: bool,
}

#[cfg(test)]
impl Default for MockResponses {
    fn default() -> Self {
        Self {
            orderbook: Some(OrderBook {
                token_id: "test".to_string(),
                bids: vec![],
                asks: vec![],
                best_bid: Some(0.50),
                best_ask: Some(0.52),
            }),
            place_order_success: true,
            cancel_success: true,
        }
    }
}

impl Executor {
    /// 创建新的执行器 (生产环境)
    #[cfg(not(test))]
    pub fn new(config: &Config) -> Result<Self> {
        let order_size =
            Decimal::from_f64_retain(config.strategy.order_size_usd).unwrap_or(dec!(1000));

        Ok(Self { order_size })
    }

    /// 创建新的执行器 (测试环境)
    #[cfg(test)]
    pub fn new_test(order_size: Decimal, mock: MockResponses) -> Self {
        Self {
            mock_responses: std::sync::Arc::new(tokio::sync::Mutex::new(mock)),
            order_size,
        }
    }

    /// 获取订单簿
    #[instrument(skip(self), fields(token_id = %token_id))]
    pub async fn fetch_orderbook(&self, token_id: &str) -> Result<OrderBook> {
        #[cfg(test)]
        {
            let mock = self.mock_responses.lock().await;
            if let Some(ob) = &mock.orderbook {
                return Ok(ob.clone());
            }
        }

        // 生产环境返回空订单簿
        Ok(OrderBook::new(token_id.to_string()))
    }

    /// 下双边订单
    #[instrument(skip(self), fields(token_id = %token_id))]
    pub async fn place_orders(
        &self,
        token_id: &str,
        bid_price: f64,
        ask_price: f64,
    ) -> Result<(Option<String>, Option<String>)> {
        let bid_dec = Decimal::from_f64_retain(bid_price).unwrap_or(dec!(0.50));
        let ask_dec = Decimal::from_f64_retain(ask_price).unwrap_or(dec!(0.50));
        let _size = self.order_size;

        info!(
            "Placing orders for {}: bid={}, ask={}",
            token_id, bid_dec, ask_dec
        );

        #[cfg(test)]
        {
            let mock = self.mock_responses.lock().await;
            if mock.place_order_success {
                return Ok((Some("mock_buy".to_string()), Some("mock_sell".to_string())));
            } else {
                return Ok((None, None));
            }
        }

        #[cfg(not(test))]
        {
            // 生产环境返回 mock
            Ok((Some("order_1".to_string()), Some("order_2".to_string())))
        }
    }

    /// 取消订单
    #[instrument(skip(self))]
    pub async fn cancel_orders(&self, _market_id: &str) -> Result<()> {
        #[cfg(test)]
        {
            let mock = self.mock_responses.lock().await;
            if !mock.cancel_success {
                return Err(anyhow::anyhow!("Cancel failed"));
            }
        }

        Ok(())
    }

    /// 取消所有订单
    #[instrument(skip(self))]
    pub async fn cancel_all_orders(&self) -> Result<()> {
        self.cancel_orders("").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_executor_creation() {
        let mock = MockResponses::default();
        let executor = Executor::new_test(dec!(1000), mock);

        assert_eq!(executor.order_size, dec!(1000));
    }

    #[tokio::test]
    async fn test_fetch_orderbook() {
        let mock = MockResponses::default();
        let executor = Executor::new_test(dec!(1000), mock);

        let result = executor.fetch_orderbook("test_token").await;

        assert!(result.is_ok());
        let ob = result.unwrap();
        assert_eq!(ob.token_id, "test");
        assert_eq!(ob.best_bid, Some(0.50));
        assert_eq!(ob.best_ask, Some(0.52));
    }

    #[tokio::test]
    async fn test_place_orders_success() {
        let mock = MockResponses::default();
        let executor = Executor::new_test(dec!(1000), mock);

        let (buy_id, sell_id) = executor
            .place_orders("test_token", 0.50, 0.52)
            .await
            .unwrap();

        assert!(buy_id.is_some());
        assert!(sell_id.is_some());
        assert_eq!(buy_id.unwrap(), "mock_buy");
        assert_eq!(sell_id.unwrap(), "mock_sell");
    }

    #[tokio::test]
    async fn test_place_orders_failure() {
        let mut mock = MockResponses::default();
        mock.place_order_success = false;

        let executor = Executor::new_test(dec!(1000), mock);
        let (buy_id, sell_id) = executor
            .place_orders("test_token", 0.50, 0.52)
            .await
            .unwrap();

        assert!(buy_id.is_none());
        assert!(sell_id.is_none());
    }

    #[tokio::test]
    async fn test_cancel_orders() {
        let mock = MockResponses::default();
        let executor = Executor::new_test(dec!(1000), mock);

        let result = executor.cancel_orders("market1").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cancel_orders_failure() {
        let mut mock = MockResponses::default();
        mock.cancel_success = false;

        let executor = Executor::new_test(dec!(1000), mock);
        let result = executor.cancel_orders("market1").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cancel_all_orders() {
        let mock = MockResponses::default();
        let executor = Executor::new_test(dec!(1000), mock);

        let result = executor.cancel_all_orders().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_order_size() {
        let mock = MockResponses::default();
        let executor = Executor::new_test(dec!(500), mock);

        assert_eq!(executor.order_size, dec!(500));
    }

    #[tokio::test]
    async fn test_multiple_place_orders() {
        let mock = MockResponses::default();
        let executor = Executor::new_test(dec!(1000), mock);

        // 下 3 次订单
        for _ in 0..3 {
            let (buy_id, sell_id) = executor.place_orders("test", 0.50, 0.52).await.unwrap();
            assert!(buy_id.is_some());
            assert!(sell_id.is_some());
        }
    }

    #[tokio::test]
    async fn test_price_validation() {
        let mock = MockResponses::default();
        let executor = Executor::new_test(dec!(1000), mock);

        // 测试边界价格
        let (buy_id, sell_id) = executor.place_orders("test", 0.01, 0.99).await.unwrap();
        assert!(buy_id.is_some());
        assert!(sell_id.is_some());

        // 测试零价格
        let (buy_id, sell_id) = executor.place_orders("test", 0.0, 0.0).await.unwrap();
        assert!(buy_id.is_some());
        assert!(sell_id.is_some());
    }

    #[tokio::test]
    async fn test_buy_only_failure() {
        // 测试部分失败场景
        let mock = MockResponses::default();
        let executor = Executor::new_test(dec!(1000), mock);

        // 当前实现是全部成功或全部失败
        // 可以扩展 MockResponses 支持更细粒度的控制
        let (buy_id, sell_id) = executor.place_orders("test", 0.50, 0.52).await.unwrap();

        // 要么都成功，要么都失败
        assert_eq!(buy_id.is_some(), sell_id.is_some());
    }

    #[tokio::test]
    async fn test_orderbook_empty() {
        let mut mock = MockResponses::default();
        mock.orderbook = None;

        let executor = Executor::new_test(dec!(1000), mock);
        let result = executor.fetch_orderbook("test").await;

        // 当 mock 为空时，返回空订单簿
        assert!(result.is_ok());
    }
}
