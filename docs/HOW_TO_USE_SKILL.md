# 如何使用 vps-debugger Skill

## ✅ Skill 已安装

**位置**: `~/.config/opencode/skills/vps-debugger/SKILL.md`

**状态**: 已添加 YAML front matter 元数据，opencode 应能自动识别

## 🚀 加载方式

### 方式 1: 自动加载（推荐）

在 opencode 中，当你提到以下关键词时，会自动加载此 skill：

- "VPS"
- "部署"
- "调试"
- "主网测试"
- "polymarket"
- "poly-client"

**示例对话**:
```
你：帮我部署到 VPS
AI: [自动加载 vps-debugger skill 并提供部署指导]

你：如何在 VPS 上运行主网测试？
AI: [使用 skill 中的配置提供详细步骤]

你：VPS 连接不上了
AI: [使用 skill 中的诊断流程帮你排查]
```

### 方式 2: 手动指定

如果自动加载不工作，可以在对话中明确提到 skill 名称：

```
你：使用 vps-debugger skill 帮我检查 VPS 连接
```

### 方式 3: 重启 opencode

有时需要重启 opencode 才能加载新技能：

```bash
# 退出并重新打开 opencode
# 或者刷新 opencode 窗口
```

## 📋 Skill 包含的配置

### VPS 服务器
- **主机**: 95.179.239.239
- **用户**: root
- **SSH 密钥**: ~/works/agent-keys/agent

### Polymarket 钱包
- **地址**: 0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6
- **网络**: Polygon 主网
- **测试限制**: 最大 1 USDC 订单

### 工作流程
1. 本地编译（不在 VPS 编译）
2. 上传二进制到 VPS
3. VPS 运行测试
4. 下载日志分析
5. 本地修复 bug

## 🛠️ 常用命令

### 部署
```bash
./scripts/deploy-to-vps.sh
```

### 测试
```bash
./scripts/mainnet-test.sh
```

### 诊断
```bash
./scripts/vps-debug.sh
```

### 连接
```bash
ssh root@95.179.239.239 -o IdentitiesOnly=yes -i ~/works/agent-keys/agent
```

## ❓ 如果 skill 仍未加载

### 检查方法 1: 验证文件存在
```bash
ls -la ~/.config/opencode/skills/vps-debugger/
# 应该看到 SKILL.md 文件
```

### 检查方法 2: 验证元数据
```bash
head -15 ~/.config/opencode/skills/vps-debugger/SKILL.md
# 应该看到 YAML front matter (--- 开头)
```

### 检查方法 3: 重新打开 opencode
关闭并重新打开 opencode 窗口

### 检查方法 4: 手动触发
在对话中说：
```
"加载 vps-debugger skill"
```

## 📖 相关文档

- [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md) - 完整部署指南
- [MAINNET_TEST_GUIDE.md](MAINNET_TEST_GUIDE.md) - 主网测试指南
- [QUICK_REFERENCE.md](../../QUICK_REFERENCE.md) - 快速参考

---

**更新时间**: 2026-03-03
**Skill 版本**: 1.0
