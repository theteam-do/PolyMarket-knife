//! 订单签名器 - 使用 k256

use anyhow::{Context, Result};
use k256::ecdsa::{SigningKey, Signature};
use k256::ecdsa::signature::Signer as _;
use sha3::{Digest, Keccak256};

/// 订单签名器
pub struct OrderSigner {
    signer: SigningKey,
    address: String,
}

impl OrderSigner {
    /// 从私钥字节创建签名器
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        let signer = SigningKey::from_bytes(bytes.into())
            .context("Failed to create signing key")?;
        
        // 从私钥推导地址
        let verifying_key = signer.verifying_key();
        let uncompressed = verifying_key.to_encoded_point(false);
        let bytes = &uncompressed.as_bytes()[1..]; // 去掉 0x04 前缀
        
        let hash = Keccak256::digest(bytes);
        let address = format!("0x{}", hex::encode(&hash[12..]));
        
        Ok(Self { signer, address })
    }

    /// 从十六进制私钥字符串创建
    pub fn from_hex(hex_key: &str) -> Result<Self> {
        let pk = hex_key.strip_prefix("0x").unwrap_or(hex_key);
        let bytes: [u8; 32] = hex::decode(pk)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid private key length"))?;
        
        Self::from_bytes(&bytes)
    }

    /// 获取签名者地址
    pub fn address(&self) -> &str {
        &self.address
    }

    /// 签名订单
    pub fn sign_order(&self, order_hash: &str) -> Result<String> {
        // 计算订单哈希
        let hash = Keccak256::digest(order_hash.as_bytes());
        
        // 签名
        let signature: Signature = self.signer.sign(&hash);
        
        // 返回十六进制签名
        Ok(format!("0x{}", hex::encode(signature.to_bytes())))
    }

    /// 生成订单哈希
    pub fn hash_order(
        &self,
        token_id: &str,
        price: &str,
        size: &str,
        side: &str,
        nonce: u64,
        expiration: u64,
    ) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}|{}",
            token_id, price, size, side, nonce, expiration, self.address
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signer_creation() {
        // 使用测试私钥 (Anvil 默认)
        let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let signer = OrderSigner::from_hex(private_key).unwrap();
        
        assert_eq!(signer.address().len(), 42); // 0x + 40 hex chars
        // 地址可能大小写不同，比较时忽略大小写
        assert_eq!(signer.address().to_lowercase(), "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266");
    }

    #[test]
    fn test_sign_order() {
        let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let signer = OrderSigner::from_hex(private_key).unwrap();
        
        let order_hash = "test_order_hash";
        let signature = signer.sign_order(order_hash).unwrap();
        
        assert!(signature.starts_with("0x"));
        // 签名长度：0x + 128 hex chars (64 bytes) + 1 (v value not included in to_bytes)
        assert!(signature.len() >= 130); // At least 0x + 128 chars
    }

    #[test]
    fn test_hash_order() {
        let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let signer = OrderSigner::from_hex(private_key).unwrap();
        
        let hash = signer.hash_order(
            "123456",
            "0.50",
            "100",
            "BUY",
            1234567890,
            0,
        );
        
        assert!(hash.contains("123456"));
        assert!(hash.contains("0.50"));
        assert!(hash.contains("BUY"));
    }
}
