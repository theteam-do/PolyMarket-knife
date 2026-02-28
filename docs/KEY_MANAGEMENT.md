# 密钥管理指南

## ⚠️ 安全警告

**私钥安全至关重要！**

- ❌ **永远不要**提交私钥到 Git
- ❌ **永远不要**在日志中打印私钥
- ❌ **永远不要**通过不安全的渠道传输私钥
- ✅ **始终**使用安全的密钥管理方案

---

## 密钥管理方案

### 方案 A: 环境变量 (开发环境)

```bash
# .env (不要提交到 Git)
POLYMARKET_PRIVATE_KEY="0x..."
CLOB_API_KEY="..."
CLOB_API_SECRET="..."

# 加载环境变量
source .env
export POLYMARKET_PRIVATE_KEY
```

**优点**:
- ✅ 简单
- ✅ 快速设置

**缺点**:
- ❌ 不够安全
- ❌ 难以轮换
- ❌ 不适合生产

---

### 方案 B: AWS Secrets Manager (推荐)

```bash
# 存储密钥
aws secretsmanager create-secret \
  --name polymarket/private-key \
  --secret-string "0x..."

# 读取密钥
export POLYMARKET_PRIVATE_KEY=$(aws secretsmanager get-secret-value \
  --secret-id polymarket/private-key \
  --query SecretString \
  --output text)
```

**优点**:
- ✅ 安全加密存储
- ✅ 自动轮换
- ✅ 访问审计
- ✅ IAM 集成

**缺点**:
- ⚠️ AWS 依赖
- ⚠️ 成本

---

### 方案 C: HashiCorp Vault (企业级)

```bash
# 启动 Vault
vault server -dev

# 存储密钥
vault kv put secret/polymarket private_key="0x..."

# 读取密钥
export POLYMARKET_PRIVATE_KEY=$(vault kv get -field=private_key secret/polymarket)
```

**优点**:
- ✅ 最高安全性
- ✅ 动态密钥
- ✅ 详细审计
- ✅ 多云支持

**缺点**:
- ⚠️ 运维复杂
- ⚠️ 需要专门团队

---

## 密钥生成

### 使用脚本生成

```bash
# 生成新密钥
./scripts/setup-keys.sh generate

# 输出:
# Private Key: 0x...
# ⚠️  Use web3 tools to derive address
```

### 手动生成

```bash
# 使用 openssl
openssl rand -hex 32

# 使用 Python
python3 -c "import secrets; print('0x' + secrets.token_hex(32))"
```

---

## 密钥验证

```bash
# 验证密钥格式
./scripts/setup-keys.sh validate

# 检查:
# ✅ 格式正确 (0x + 64 hex)
# ✅ 非空
# ✅ 非测试值
```

---

## 密钥轮换

### 流程

1. **生成新密钥**
   ```bash
   ./scripts/setup-keys.sh generate
   ```

2. **更新 Secrets Manager**
   ```bash
   aws secretsmanager update-secret \
     --secret-id polymarket/private-key \
     --secret-string "0x..."
   ```

3. **更新应用程序**
   ```bash
   # 重启服务以读取新密钥
   systemctl restart market-maker
   ```

4. **验证新密钥**
   ```bash
   curl http://localhost:8080/health
   ```

5. **撤销旧密钥** (在 Polymarket 平台)

---

## 安全最佳实践

### 1. 最小权限原则

- 为不同环境使用不同密钥
- 限制 API 密钥权限
- 定期审查访问日志

### 2. 加密存储

- 静态加密 (at rest)
- 传输加密 (in transit)
- 内存加密 (in memory)

### 3. 访问控制

- IAM 角色
- 多因素认证
- 访问审计

### 4. 监控告警

- 异常访问检测
- 密钥使用监控
- 自动告警

### 5. 应急计划

- 密钥泄露响应流程
- 快速撤销机制
- 备份恢复方案

---

## 检查清单

### 部署前

- [ ] 私钥已安全存储
- [ ] 环境变量已设置
- [ ] 密钥格式已验证
- [ ] 访问权限已配置
- [ ] 监控已启用

### 运行中

- [ ] 定期检查密钥使用
- [ ] 审查访问日志
- [ ] 监控异常活动
- [ ] 定期轮换密钥

### 应急情况

- [ ] 密钥泄露响应流程
- [ ] 快速撤销机制
- [ ] 备用密钥准备
- [ ] 通知相关方

---

## 故障排查

### Q: 私钥格式错误？

**A**: 确保格式为 `0x` 后跟 64 个十六进制字符

```bash
# 正确: 0x1234567890abcdef... (66 字符)
# 错误: 1234567890abcdef... (缺少 0x)
# 错误: 0x1234... (太短)
```

### Q: 无法访问密钥？

**A**: 检查权限和环境变量

```bash
# 检查环境变量
echo $POLYMARKET_PRIVATE_KEY

# 检查 AWS 权限
aws sts get-caller-identity

# 检查 Vault 访问
vault kv get secret/polymarket
```

### Q: 密钥泄露怎么办？

**A**: 立即执行以下步骤

1. **撤销密钥** - Polymarket 平台
2. **生成新密钥** - ./scripts/setup-keys.sh generate
3. **更新配置** - 所有环境
4. **审查日志** - 检查异常活动
5. **通知相关方** - 安全团队

---

**密钥安全是生产部署的第一要务！**

