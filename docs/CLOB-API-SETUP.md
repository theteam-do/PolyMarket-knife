# Polymarket CLOB API 密钥配置指南

## 📋 步骤 1: 访问 Polymarket

打开浏览器访问：
```
https://polymarket.com
```

## 📋 步骤 2: 连接钱包

### 2.1 导入测试钱包到 MetaMask

1. 打开 MetaMask 钱包
2. 点击右上角头像 → **"导入账户"**
3. 选择 **"私钥"** 选项
4. 输入测试私钥：
   ```
   0x18f0d0ca93a73f451cf42ea17bf4cae1286fd352f81f1a965650ea49fb5951e7
   ```
5. 点击 **"导入"**

### 2.2 连接 Polymarket

1. 访问 https://polymarket.com
2. 点击右上角 **"Connect Wallet"**
3. 选择 **MetaMask**
4. 在 MetaMask 中确认连接
5. 确保切换到 **Polygon 主网**

## 📋 步骤 3: 注册 CLOB API 密钥

### 3.1 访问 CLOB 门户

```
https://clob.polymarket.com
```

### 3.2 登录 CLOB

1. 点击 **"Sign In"** 或 **"Connect Wallet"**
2. 选择 MetaMask 钱包
3. 签名登录消息（不需要 gas 费）

### 3.3 创建 API 密钥

1. 进入 **"API Keys"** 或 **"Developer"** 页面
   - 可能在右上角用户菜单中
2. 点击 **"Create New API Key"**
3. 填写信息：
   - **名称**: `PolyMarket-Knife-VPS`
   - **权限**: 勾选 `Trade` (交易权限)
   - **IP 白名单** (可选但推荐): 添加 `139.180.207.66`
4. 点击 **"Create"** 或 **"Generate"**

### 3.4 保存密钥

⚠️ **重要：API Secret 只会显示一次！立即复制保存！**

保存以下信息：
```
API Key: abc123def456... (示例)
API Secret: xyz789uvw012... (示例)
```

## 📋 步骤 4: 配置到项目

### 方式 A: 更新配置文件（推荐）

1. 在本地编辑配置文件：
```bash
nano config/market-maker-mainnet-test.toml
```

2. 添加 API 密钥：
```toml
[clob]
host = "https://clob.polymarket.com"
api_key = "你的 API_KEY"
api_secret = "你的 API_SECRET"
```

3. 上传到 VPS：
```bash
scp -o IdentitiesOnly=yes -i ~/works/agent-keys/agent \
    config/market-maker-mainnet-test.toml \
    root@139.180.207.66:/root/works/PolyMarket-knife/config/
```

### 方式 B: 使用环境变量

1. SSH 连接到 VPS：
```bash
ssh root@139.180.207.66 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent
```

2. 添加到 ~/.bashrc：
```bash
echo 'export CLOB_API_KEY="你的 API_KEY"' >> ~/.bashrc
echo 'export CLOB_API_SECRET="你的 API_SECRET"' >> ~/.bashrc
source ~/.bashrc
```

## 📋 步骤 5: 验证配置

### 5.1 测试连接

```bash
ssh root@139.180.207.66 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent

cd /root/works/PolyMarket-knife
export POLYMARKET_PRIVATE_KEY='0x18f0d0ca93a73f451cf42ea17bf4cae1286fd352f81f1a965650ea49fb5951e7'
timeout 30 ./market-maker config/market-maker-mainnet-test.toml 2>&1 | head -50
```

### 5.2 预期输出

✅ 成功：
```
INFO Market Maker starting up...
INFO Executor initialized with order_size: $1
INFO Market Maker initialized
INFO Monitoring 1 markets
INFO Fetching orderbook for token: xxx
INFO Orderbook response token=xxx ts=xxx bids=xx asks=xx
```

❌ 失败（认证错误）：
```
ERROR Authentication failed: {"error":"Unauthorized/Invalid api key"}
```

## 🔧 故障排查

### 问题 1: 找不到 API Keys 页面

**解决方案**:
- 尝试直接访问：https://clob.polymarket.com/api-keys
- 或在 CLOB 首页寻找 "Developer"、"API"、"Settings" 等菜单

### 问题 2: 无法连接钱包

**解决方案**:
- 确保 MetaMask 已解锁
- 切换到 Polygon 主网（不是测试网）
- 刷新页面重试

### 问题 3: API 密钥创建失败

**解决方案**:
- 检查钱包是否有 POLY 代币（可能需要少量用于签名）
- 尝试清除浏览器缓存
- 使用无痕模式重试

### 问题 4: 认证仍然失败

**解决方案**:
- 确认 API Key 和 Secret 没有复制错误（无多余空格）
- 检查 IP 白名单是否包含 VPS IP
- 删除旧密钥，创建新密钥重试

## 📊 获取活跃市场 ID

运行脚本获取有流动性的市场：

```bash
./scripts/fetch-active-markets.sh 10
```

选择一个市场，更新配置：

```toml
[strategy]
market_ids = ["75467129615908319583031474642658885479135630431889036121812713428992454630178"]
```

## 📞 获取帮助

如果遇到问题：
1. 检查 Polymarket Discord: https://discord.gg/polymarket
2. 查看文档：https://docs.polymarket.com
3. 联系支持：support@polymarket.com
