//! 错误处理

use thiserror::Error;

/// 适配器错误类型
#[derive(Error, Debug)]
pub enum Error {
    /// SDK 错误
    #[error("SDK error: {0}")]
    Sdk(#[from] polymarket_client_sdk::error::Error),

    /// 认证错误
    #[error("Authentication error: {0}")]
    Auth(String),

    /// 类型转换错误
    #[error("Conversion error: {from} -> {to}: {reason}")]
    Conversion {
        from: String,
        to: String,
        reason: String,
    },

    /// 订单错误
    #[error("Order error: {0}")]
    Order(String),

    /// 序列化错误
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// 十进制解析错误
    #[error("Decimal parse error: {0}")]
    DecimalParse(#[from] rust_decimal::Error),

    /// U256 解析错误
    #[error("U256 parse error: {0}")]
    U256Parse(#[from] alloy::primitives::ruint::ParseError),

    /// UUID 解析错误
    #[error("UUID parse error: {0}")]
    UuidParse(#[from] uuid::Error),

    /// 其他错误
    #[error("Other error: {0}")]
    Other(String),
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, Error>;

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Other(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Auth("Invalid credentials".to_string());
        assert_eq!(err.to_string(), "Authentication error: Invalid credentials");
    }

    #[test]
    fn test_conversion_error() {
        let err = Error::Conversion {
            from: "String".to_string(),
            to: "U256".to_string(),
            reason: "Invalid format".to_string(),
        };
        assert!(err.to_string().contains("String -> U256"));
    }
}
