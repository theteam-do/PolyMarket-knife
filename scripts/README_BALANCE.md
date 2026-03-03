# 钱包余额查询脚本

快速查询 Polymarket 测试钱包的 MATIC 和 USDC 余额。

## 📖 使用方法

### 1. 查询默认测试钱包

```bash
cd /home/de/works/PolyMarket-knife
python3 scripts/check-balance.py
```

### 2. 查询指定钱包

```bash
python3 scripts/check-balance.py 0x你的钱包地址
```

### 3. 查看所有 RPC 端点

```bash
python3 scripts/check-balance.py --list
```

## 📦 依赖安装

```bash
pip3 install web3
```

## 🎯 输出示例

```
正在查询钱包余额...

正在连接 Polygon 主网...
✅ 已连接到：https://rpc.sentio.xyz/matic...

======================================================================
💰 Polymarket 测试钱包余额报告
======================================================================

钱包地址：0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6
网络：Polygon 主网

MATIC 余额：20.0000 MATIC
  美元价值：≈ $9.00 (按 $0.45/MATIC)
  可支持交易：≈ 2000 笔 (按 0.01 MATIC/笔)

USDC 余额：5.00 USDC
  美元价值：≈ $5.00

======================================================================
📊 资金评估
======================================================================
✅ MATIC: 余额充足，可用于测试
⚠️  USDC: 余额不足 (5.00), 建议充值 100+ USDC

======================================================================
⚠️  MATIC 充足，但 USDC 不足 - 可进行 Gas 测试，无法实际交易
======================================================================

💡 充值信息:
  钱包地址：0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6
  网络：Polygon 主网
  建议充值：10+ MATIC, 100+ USDC
```

## 🔧 配置说明

### 默认钱包

脚本默认查询以下测试钱包：

```
地址：0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6
网络：Polygon 主网
```

如需修改，编辑脚本中的 `DEFAULT_WALLET` 常量。

### RPC 端点

脚本内置了 12 个 Polygon RPC 端点，会自动尝试连接直到成功：

1. https://rpc.sentio.xyz/matic ⭐ (最快)
2. https://rpc.owlracle.info/poly/...
3. https://polygon-public.nodies.app
4. https://gateway.tenderly.co/public/polygon
5. https://1rpc.io/matic
6. https://polygon-bor-rpc.publicnode.com
7. https://polygon-bor.publicnode.com
8. https://go.getblock.io/...
9. https://api.zan.top/polygon-mainnet
10. https://poly.api.pocket.network
11. https://rpc.ankr.com/polygon
12. https://api-polygon-mainnet-full.n.dwellir.com/...

### MATIC 价格

当前设置为 $0.45/MATIC，可根据市场价格更新脚本中的 `MATIC_PRICE_USD` 常量。

## 💡 使用场景

### 1. 主网测试前检查

```bash
# 测试前确认资金充足
python3 scripts/check-balance.py
```

### 2. 监控多个钱包

```bash
# 查询不同钱包
python3 scripts/check-balance.py 0x钱包 1
python3 scripts/check-balance.py 0x钱包 2
```

### 3. 集成到部署脚本

```bash
#!/bin/bash
# deploy-and-test.sh

# 部署代码
./scripts/deploy-to-vps.sh

# 检查余额
python3 scripts/check-balance.py

# 如果余额充足，运行测试
if [ $? -eq 0 ]; then
    ./scripts/mainnet-test.sh
fi
```

## ⚠️ 注意事项

1. **隐私安全**: 不要将私钥放入脚本或版本控制
2. **RPC 限制**: 公共 RPC 有速率限制，频繁查询可能被限流
3. **价格更新**: MATIC 价格是固定的，实际价格可能波动
4. **网络选择**: 确保使用 Polygon 主网，不是测试网

## 🚀 快速命令

```bash
# 添加到 bash 别名
echo "alias poly-balance='cd /home/de/works/PolyMarket-knife && python3 scripts/check-balance.py'" >> ~/.bashrc
source ~/.bashrc

# 使用别名
poly-balance
```

## 📝 故障排查

### 问题：无法连接到任何 RPC

**解决**:
1. 检查网络连接
2. 运行 `python3 scripts/check-balance.py --list` 查看所有端点
3. 尝试使用代理或 VPN

### 问题：USDC 查询失败

**解决**:
1. 确认使用的是 Polygon 主网 USDC 合约
2. 检查钱包是否真的持有 USDC
3. 尝试更换 RPC 端点

### 问题：web3 库未安装

**解决**:
```bash
pip3 install web3
```

## 🔗 相关资源

- [Polygon 官方文档](https://docs.polygon.technology/)
- [Chainlist](https://chainlist.org/chain/137) - 更多 RPC 端点
- [Polygonscan](https://polygonscan.com/) - 区块浏览器

---

**最后更新**: 2026-03-03
