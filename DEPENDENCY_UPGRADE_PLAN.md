# 依赖升级计划

## 安全漏洞依赖

### 1. protobuf (RUSTSEC-2024-0437)

**当前版本**: 2.28.0  
**目标版本**: >=3.7.2  
**影响**: 递归漏洞可能导致崩溃  
**依赖链**: `monitor -> prometheus -> protobuf`

#### 升级步骤

```bash
# 1. 升级 prometheus 到最新版本
cargo update -p prometheus

# 如果失败，修改 Cargo.toml
# monitor/Cargo.toml
prometheus = "0.14"
```

**状态**: ⚠️ 需要 prometheus 0.14 支持

---

### 2. ring (RUSTSEC-2025-0009)

**当前版本**: 0.16.20  
**目标版本**: >=0.17.12  
**影响**: AES 函数可能 panic  
**依赖链**: `follow-trade -> ethers -> jsonwebtoken -> ring`

#### 升级步骤

**选项 A: 升级 ethers 到 v3 (推荐)**

```toml
# follow-trade/Cargo.toml
# 将 ethers 升级到 v3 (如果可用)
ethers = "3.0"
```

**选项 B: 迁移到 alloy (长期方案)**

```toml
# 移除 ethers，完全使用 alloy
# follow-trade/Cargo.toml
alloy = { version = "1.6", features = ["providers", "transports"] }
```

**状态**: ⚠️ ethers v2 锁定 jsonwebtoken v8，需要升级 ethers

---

### 3. backoff (RUSTSEC-2025-0012)

**当前版本**: 0.4.0  
**状态**: 已停止维护  
**依赖链**: `polymarket-client-sdk -> backoff`

#### 升级步骤

```toml
# rs-clob-client/Cargo.toml
# 替换为 tokio 内置重试或 backon
# 选项 1: 使用 tokio::time::retry (实验性)
# 选项 2: 使用 backon crate
backon = "1.0"
```

**状态**: 📝 需要修改 rs-clob-client 代码

---

## 执行计划

### 第 1 周 - 评估和测试

- [ ] 创建升级分支
- [ ] 评估每个升级的影响范围
- [ ] 编写回归测试

### 第 2 周 - 执行升级

- [ ] 升级 prometheus
- [ ] 升级 ethers 或迁移到 alloy
- [ ] 替换 backoff

### 第 3 周 - 测试和验证

- [ ] 运行完整测试套件
- [ ] 性能基准测试
- [ ] 安全审计

### 第 4 周 - 部署

- [ ] 逐步部署到测试环境
- [ ] 监控错误率
- [ ] 生产环境部署

---

## 临时缓解措施

在升级完成前，可以采取以下措施降低风险：

### 1. 监控和告警

```rust
// 添加 panic hook 捕获异常
std::panic::set_hook(Box::new(|panic_info| {
    tracing::error!("Panic occurred: {:?}", panic_info);
    // 发送告警
}));
```

### 2. 限制输入大小

```rust
// 防止递归攻击
const MAX_RECURSION_DEPTH: usize = 100;
```

### 3. 定期重启

```bash
# 设置 systemd 自动重启
[Service]
Restart=always
RestartSec=300
```

---

## 参考资源

- [RUSTSEC-2024-0437](https://rustsec.org/advisories/RUSTSEC-2024-0437)
- [RUSTSEC-2025-0009](https://rustsec.org/advisories/RUSTSEC-2025-0009)
- [RUSTSEC-2025-0012](https://rustsec.org/advisories/RUSTSEC-2025-0012)
- [ethers-rs GitHub](https://github.com/gakonst/ethers-rs)
- [alloy GitHub](https://github.com/alloy-rs/alloy)

---

**创建时间**: 2026-03-05  
**最后更新**: 2026-03-05  
**负责人**: PolyMarket Knife Team
