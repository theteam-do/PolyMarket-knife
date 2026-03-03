// 使用 polymarket-client-sdk 创建 API 密钥
// 运行：cargo run --bin create-api-key

use polymarket_client_sdk::ClobClient;
use alloy::signers::local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量读取私钥
    let private_key = std::env::var("POLYMARKET_PRIVATE_KEY")
        .expect("POLYMARKET_PRIVATE_KEY must be set");
    
    // 创建签名器
    let signer = PrivateKeySigner::from_hex(&private_key)?;
    let address = signer.address();
    
    println!("🔑 Wallet Address: {}", address);
    
    // 创建 CLOB 客户端
    let client = ClobClient::new(
        "https://clob.polymarket.com",
        137, // Polygon mainnet
        signer,
    );
    
    // 创建或派生 API 密钥
    println!("📝 Creating/Deriving API credentials...");
    let credentials = client.create_or_derive_api_creds().await?;
    
    println!("\n✅ API Credentials Generated!\n");
    println!("┌─────────────────────────────────────────────┐");
    println!("│ API Key:       {:<36} │", credentials.api_key);
    println!("│ API Secret:    {:<36} │", credentials.secret);
    println!("│ Passphrase:    {:<36} │", credentials.passphrase);
    println!("└─────────────────────────────────────────────┘");
    
    println!("\n⚠️  IMPORTANT: Save these credentials securely!");
    println!("   The secret will only be shown once!\n");
    
    println!("📋 下一步:");
    println!("   1. 复制上面的 API Key, Secret, Passphrase");
    println!("   2. 更新配置文件 config/market-maker-mainnet-test.toml:");
    println!();
    println!("      [clob]");
    println!("      api_key = \"{}\"", credentials.api_key);
    println!("      api_secret = \"{}\"", credentials.secret);
    println!();
    println!("   或使用环境变量:");
    println!("      export CLOB_API_KEY=\"{}\"", credentials.api_key);
    println!("      export CLOB_API_SECRET=\"{}\"", credentials.secret);
    println!();
    
    Ok(())
}
