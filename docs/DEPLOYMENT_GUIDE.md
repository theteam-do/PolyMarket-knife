# 本地编译 + VPS 运行 部署指南

## 工作流程

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│  本地开发   │ ──▶ │  本地编译    │ ──▶ │  上传 VPS   │
│  (修改代码) │     │ (release)    │     │ (二进制)    │
└─────────────┘     └──────────────┘     └─────────────┘
                                              │
                                              ▼
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│  问题修复   │ ◀── │  分析日志    │ ◀── │  VPS 测试   │
│  (重新编译) │     │  (本地查看)  │     │ (主网验证)  │
└─────────────┘     └──────────────┘     └─────────────┘
```

## 优势

| 对比项 | VPS 编译 | 本地编译 |
|--------|----------|----------|
| 编译时间 | 15-30 分钟 | 5-10 分钟 |
| CPU 占用 | 100% (影响运行) | 本地资源 |
| 内存占用 | 可能 OOM | 本地资源 |
| 迭代速度 | 慢 | 快 |
| 真实部署 | ❌ | ✅ |

## 快速开始

### 一键部署并测试

```bash
# 1. 本地编译并上传到 VPS
./scripts/deploy-to-vps.sh

# 2. 运行主网测试
./scripts/mainnet-test.sh
```

## 详细步骤

### 步骤 1: 本地编译

```bash
# 清理旧构建
cargo clean

# 编译 release 版本
cargo build --release

# 验证二进制文件
ls -lh target/release/
```

**编译时间** (参考 M1/M2 Mac):
- market-maker: ~2 分钟
- arbitrage: ~2 分钟
- follow-trade: ~2 分钟
- volatility-hunter: ~2 分钟
- 总计：~8-10 分钟

### 步骤 2: 上传到 VPS

```bash
# 使用部署脚本（推荐）
./scripts/deploy-to-vps.sh

# 或手动上传
scp -i ~/works/agent-keys/agent \
    target/release/market-maker \
    root@95.179.239.239:/home/de/works/PolyMarket-knife/target/release/

scp -i ~/works/agent-keys/agent \
    target/release/arbitrage \
    root@95.179.239.239:/home/de/works/PolyMarket-knife/target/release/

# 上传配置文件
scp -i ~/works/agent-keys/agent \
    config/*-mainnet-test.toml \
    root@95.179.239.239:/home/de/works/PolyMarket-knife/config/
```

### 步骤 3: VPS 验证

```bash
# SSH 连接
ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent

# 验证二进制文件
cd /home/de/works/PolyMarket-knife
ls -lh target/release/

# 测试运行
./target/release/market-maker --help
```

### 步骤 4: 运行测试

```bash
# 运行主网测试
./scripts/mainnet-test.sh

# 或手动运行
ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent "
    cd /home/de/works/PolyMarket-knife
    export POLYMARKET_PRIVATE_KEY='0x18f0d0ca93a73f451cf42ea17bf4cae1286fd352f81f1a965650ea49fb5951e7'
    timeout 120 ./target/release/market-maker --config config/market-maker-mainnet-test.toml
"
```

### 步骤 5: 查看日志

```bash
# 实时查看 VPS 日志
ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent \
    "tail -f /home/de/works/PolyMarket-knife/logs/*.log"

# 下载日志到本地
scp -i ~/works/agent-keys/agent \
    root@95.179.239.239:/home/de/works/PolyMarket-knife/logs/*.log \
    ./logs/

# 本地分析
grep -E "ERROR|order.*filled" logs/*.log | tail -20
```

## 常见问题修复流程

### 发现 Bug

```bash
# 1. 本地修复代码
vim src/xxx.rs

# 2. 重新编译
cargo build --release

# 3. 重新部署
./scripts/deploy-to-vps.sh

# 4. 验证修复
./scripts/mainnet-test.sh
```

### 配置问题

```bash
# 1. 本地修改配置
vim config/market-maker-mainnet-test.toml

# 2. 上传配置
scp -i ~/works/agent-keys/agent \
    config/market-maker-mainnet-test.toml \
    root@95.179.239.239:/home/de/works/PolyMarket-knife/config/

# 3. 重新运行
./scripts/mainnet-test.sh
```

## 部署脚本说明

### deploy-to-vps.sh

**功能**:
1. 本地编译所有策略
2. 备份 VPS 上的旧版本
3. 上传新二进制文件
4. 上传配置文件
5. 验证部署
6. 可选运行测试

**使用方法**:
```bash
# 使用默认配置
./scripts/deploy-to-vps.sh

# 自定义 VPS
export VPS_HOST="your_vps_ip"
export VPS_USER="root"
export VPS_KEY="~/path/to/key"
./scripts/deploy-to-vps.sh
```

### mainnet-test.sh

**功能**:
1. 检查 VPS 连接
2. 检查二进制文件
3. 查询钱包余额
4. 选择策略测试
5. 运行测试并下载日志
6. 分析测试结果

**使用方法**:
```bash
# 运行测试
./scripts/mainnet-test.sh

# 选择策略:
# 1) Market Maker
# 2) Arbitrage
# 3) Follow Trade
# 4) Volatility Hunter
# 5) 全部顺序测试
```

## 日志管理

### 日志位置

| 位置 | 路径 |
|------|------|
| 本地日志 | `./logs/` |
| VPS 日志 | `/home/de/works/PolyMarket-knife/logs/` |
| VPS 临时日志 | `/tmp/*_test.log` |

### 日志分析

```bash
# 统计订单数量
grep -c "order created" logs/*.log

# 统计成交数量
grep -c "order filled" logs/*.log

# 统计错误
grep -c "ERROR" logs/*.log

# 查看盈亏
grep "PnL" logs/*.log | tail -10

# 查看延迟
grep "latency\|ms" logs/*.log | tail -20
```

## 版本管理

### 备份策略

部署脚本会自动备份旧版本：
```bash
# VPS 备份目录
/home/de/works/PolyMarket-knife/backup/release_YYYYMMDD_HHMMSS/
```

### 回滚

```bash
# SSH 到 VPS
ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent

# 进入备份目录
cd /home/de/works/PolyMarket-knife/backup/

# 查看备份
ls -la

# 恢复备份
cp release_YYYYMMDD_HHMMSS/* /home/de/works/PolyMarket-knife/target/release/
```

## 性能优化

### 本地编译优化

```bash
# 使用所有 CPU 核心
export CARGO_BUILD_JOBS=$(nproc)

# 启用 LTO (更慢但更优化)
# 在 Cargo.toml 中：
# [profile.release]
# lto = true

# 使用 sccache 加速编译
cargo install sccache
export RUSTC_WRAPPER=sccache
cargo build --release
```

### 上传优化

```bash
# 使用 rsync 增量上传
rsync -avz -e "ssh -i ~/works/agent-keys/agent" \
    target/release/ \
    root@95.179.239.239:/home/de/works/PolyMarket-knife/target/release/
```

## 安全检查清单

部署前检查：
- [ ] 本地测试通过
- [ ] 配置文件正确
- [ ] 私钥设置正确
- [ ] 风控参数合理

部署后检查：
- [ ] 二进制文件权限正确
- [ ] 配置文件已上传
- [ ] 程序可以启动
- [ ] 日志正常输出

测试后检查：
- [ ] 订单正常创建
- [ ] 订单正常成交
- [ ] 无异常错误
- [ ] 盈亏符合预期
- [ ] 钱包余额正确

## 自动化 CI/CD (可选)

### GitHub Actions 示例

```yaml
name: Deploy to VPS

on:
  push:
    branches: [ main ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Build
        run: cargo build --release
      
      - name: Deploy to VPS
        uses: easingthemes/ssh-deploy@v3
        with:
          SSH_PRIVATE_KEY: ${{ secrets.VPS_KEY }}
          REMOTE_HOST: 95.179.239.239
          REMOTE_USER: root
          SOURCE: target/release/
          TARGET: /home/de/works/PolyMarket-knife/target/release/
```

---

**最后更新**: 2026-03-03
**VPS**: 95.179.239.239
**测试钱包**: 0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6
