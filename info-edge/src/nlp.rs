//! NLP 引擎 - 情感分析和关键词匹配

use crate::collector::NewsItem;
use crate::config::SourcesConfig;

#[derive(Debug)]
pub struct SentimentResult {
    pub confidence: f64,
    pub direction: Direction,
    pub matched_keywords: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Yes,
    No,
    Neutral,
}

pub struct NLPEngine {
    keywords: Vec<String>,
    positive_words: Vec<String>,
    negative_words: Vec<String>,
}

impl NLPEngine {
    pub fn new(sources: &SourcesConfig) -> Self {
        Self {
            keywords: sources.keywords.clone(),
            positive_words: vec![
                "win", "success", "approve", "yes", "agree", "pass", "confirm", "positive",
                "bullish", "up", "rise", "gain",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
            negative_words: vec![
                "lose", "fail", "reject", "no", "disagree", "deny", "negative", "bearish", "down",
                "fall", "drop",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        }
    }

    pub fn analyze(&self, item: &NewsItem) -> SentimentResult {
        let text = format!("{} {}", item.title, item.content).to_lowercase();

        // 关键词匹配
        let matched_keywords = self.match_keywords(&text);

        // 情感分析
        let sentiment_score = self.simple_sentiment(&text);

        // 时效性评分
        let recency_score = self.calc_recency(item.timestamp);

        // 计算总体置信度
        let keyword_score = if matched_keywords.is_empty() {
            0.0
        } else {
            (matched_keywords.len() as f64 / self.keywords.len() as f64).min(1.0)
        };

        let confidence = (keyword_score + sentiment_score.abs() + recency_score) / 3.0;

        let direction = if sentiment_score > 0.1 {
            Direction::Yes
        } else if sentiment_score < -0.1 {
            Direction::No
        } else {
            Direction::Neutral
        };

        SentimentResult {
            confidence: confidence.min(1.0),
            direction,
            matched_keywords,
        }
    }

    fn match_keywords(&self, text: &str) -> Vec<String> {
        self.keywords
            .iter()
            .filter(|kw| text.contains(&kw.to_lowercase()))
            .cloned()
            .collect()
    }

    fn simple_sentiment(&self, text: &str) -> f64 {
        let pos_count = self
            .positive_words
            .iter()
            .filter(|w| text.contains(w.as_str()))
            .count();

        let neg_count = self
            .negative_words
            .iter()
            .filter(|w| text.contains(w.as_str()))
            .count();

        if pos_count + neg_count == 0 {
            return 0.0;
        }

        (pos_count as f64 - neg_count as f64) / (pos_count + neg_count) as f64
    }

    fn calc_recency(&self, timestamp: u64) -> f64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let age_secs = now.saturating_sub(timestamp);

        // 越新越高分
        if age_secs < 60 {
            1.0
        } else if age_secs < 300 {
            0.8
        } else if age_secs < 3600 {
            0.5
        } else if age_secs < 86400 {
            0.2
        } else {
            0.0
        }
    }
}
