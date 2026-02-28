//! 认证模块

use polymarket_client_sdk::auth::{Credentials, LocalSigner, Signer};
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::clob::state::Authenticated;
use polymarket_client_sdk::auth::Normal;
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use std::str::FromStr;
use tracing::info;

use crate::error::{Error, Result};

/// 认证配置
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// 私钥
    pub private_key: String,
    /// CLOB API 地址
    pub clob_host: String,
    /// 链 ID (默认 Polygon 主网)
    pub chain_id: u64,
}

impl AuthConfig {
    /// 创建新的认证配置
    pub fn new(private_key: &str, clob_host: &str) -> Self {
        Self {
            private_key: private_key.to_string(),
            clob_host: clob_host.to_string(),
            chain_id: POLYGON,
        }
    }

    /// 设置链 ID
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = chain_id;
        self
    }

    /// 创建认证客户端
    pub async fn create_client(&self) -> Result<Client<Authenticated<Normal>>> {
        info!("Authenticating with CLOB host: {}", self.clob_host);

        let signer = LocalSigner::from_str(&self.private_key)
            .map_err(|e| Error::Auth(format!("Failed to parse private key: {}", e)))?
            .with_chain_id(Some(self.chain_id));

        let client: Client<Authenticated<Normal>> = Client::new(&self.clob_host, Config::default())
            .map_err(|e| Error::Auth(format!("Failed to create client: {}", e)))?
            .authentication_builder(&signer)
            .authenticate()
            .await
            .map_err(|e| Error::Auth(format!("Failed to authenticate: {}", e)))?;

        info!("Authentication successful");
        Ok(client)
    }
}

/// 从环境变量创建认证配置
impl Default for AuthConfig {
    fn default() -> Self {
        let private_key = std::env::var(PRIVATE_KEY_VAR)
            .unwrap_or_else(|_| String::new());
        let clob_host = std::env::var("CLOB_HOST")
            .unwrap_or_else(|_| "https://clob.polymarket.com".to_string());

        Self::new(&private_key, &clob_host)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_new() {
        let config = AuthConfig::new("0x1234", "https://clob.polymarket.com");
        assert_eq!(config.private_key, "0x1234");
        assert_eq!(config.clob_host, "https://clob.polymarket.com");
        assert_eq!(config.chain_id, POLYGON);
    }

    #[test]
    fn test_auth_config_with_chain_id() {
        let config = AuthConfig::new("0x1234", "https://clob.polymarket.com")
            .with_chain_id(80002); // Amoy testnet
        assert_eq!(config.chain_id, 80002);
    }
}
