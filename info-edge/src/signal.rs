//! 信号生成器

use crate::collector::NewsItem;
use crate::config::StrategyConfig;
use crate::nlp::{Direction, SentimentResult};

#[derive(Debug)]
pub struct Signal {
    pub market: String,
    pub direction: Direction,
    pub confidence: f64,
    pub expected_return: f64,
}

pub struct SignalGenerator {
    config: StrategyConfig,
}

impl SignalGenerator {
    pub fn new(config: &StrategyConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub fn generate(&self, item: &NewsItem, sentiment: &SentimentResult) -> Option<Signal> {
        // 置信度检查
        if sentiment.confidence < self.config.confidence_threshold {
            return None;
        }

        // 中性信号不交易
        if sentiment.direction == Direction::Neutral {
            return None;
        }

        // 必须有相关市场
        let market = item.market.clone()?;

        // 计算预期收益
        let expected_return = self.calc_expected_return(sentiment.confidence);

        if expected_return < self.config.min_expected_return {
            return None;
        }

        Some(Signal {
            market,
            direction: sentiment.direction,
            confidence: sentiment.confidence,
            expected_return,
        })
    }

    fn calc_expected_return(&self, confidence: f64) -> f64 {
        // 简化模型：置信度越高，预期收益越高
        // 最高可达 200%
        confidence * 2.0
    }
}
