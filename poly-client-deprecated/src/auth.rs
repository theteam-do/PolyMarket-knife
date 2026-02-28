//! 认证模块 - Polymarket API 签名

use anyhow::{Context, Result};
use ethers::signers::LocalWallet;
use ethers::signers::Signer;
use hex;
use sha3::{Digest, Keccak256};
use std::time::{SystemTime, UNIX_EPOCH};

/// 认证配置
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub api_key: String,
    pub api_secret: String,
    pub signer: LocalWallet,
}

impl AuthConfig {
    /// 从私钥派生 API 凭证
    pub fn from_private_key(private_key: &str, _host: &str) -> Result<Self> {
        let wallet = private_key
            .parse::<LocalWallet>()
            .context("Failed to parse private key")?;

        // 简化实现：使用地址作为 API key
        let api_key = format!("0x{}", hex::encode(wallet.address().as_bytes()));
        let api_secret = hex::encode(vec![0u8; 32]);

        Ok(Self {
            api_key,
            api_secret,
            signer: wallet,
        })
    }

    pub fn new(api_key: String, api_secret: String, signer: LocalWallet) -> Self {
        Self {
            api_key,
            api_secret,
            signer,
        }
    }
}

/// 签名中间件
#[derive(Debug)]
pub struct PolySigner {
    pub api_key: String,
    pub api_secret: String,
    pub wallet: LocalWallet,
}

impl PolySigner {
    pub fn new(config: &AuthConfig) -> Self {
        Self {
            api_key: config.api_key.clone(),
            api_secret: config.api_secret.clone(),
            wallet: config.signer.clone(),
        }
    }

    /// 为请求添加签名头
    pub fn sign_request(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> Vec<(String, String)> {
        let timestamp = current_timestamp_ms();
        let message = self.build_message(method, path, timestamp, body);

        let signature = self.sign_message(&message);

        vec![
            ("POLY-API-KEY".to_string(), self.api_key.clone()),
            ("POLY-API-SIGNATURE".to_string(), hex::encode(&signature)),
            ("POLY-API-TIMESTAMP".to_string(), timestamp.to_string()),
        ]
    }

    fn build_message(
        &self,
        method: &str,
        path: &str,
        timestamp: u64,
        body: Option<&str>,
    ) -> String {
        let body_hash = if let Some(b) = body {
            hex::encode(Keccak256::digest(b.as_bytes()))
        } else {
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()
        };

        format!("{}{}{}{}", timestamp, method, path, body_hash)
    }

    fn sign_message(&self, message: &str) -> Vec<u8> {
        use k256::ecdsa::signature::Signer as _;
        use k256::ecdsa::Signature;
        let message_hash = Keccak256::digest(message.as_bytes());
        let signing_key =
            k256::ecdsa::SigningKey::from_bytes(&self.wallet.signer().to_bytes()).unwrap();
        let signature: Signature = signing_key.sign(&message_hash);
        signature.to_bytes().to_vec()
    }
}

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// 认证中间件
#[derive(Debug)]
pub struct AuthMiddleware {
    signer: PolySigner,
}

impl AuthMiddleware {
    pub fn new(config: &AuthConfig) -> Self {
        Self {
            signer: PolySigner::new(config),
        }
    }

    pub fn add_headers(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> Vec<(String, String)> {
        self.signer.sign_request(method, path, body)
    }
}

impl Clone for AuthMiddleware {
    fn clone(&self) -> Self {
        Self {
            signer: PolySigner::new(&AuthConfig {
                api_key: self.signer.api_key.clone(),
                api_secret: self.signer.api_secret.clone(),
                signer: self.signer.wallet.clone(),
            }),
        }
    }
}
