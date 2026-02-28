# 贡献指南

感谢你考虑为 PolyMarket Knife 做出贡献！🎉

本指南旨在帮助你了解如何参与项目开发。

## 📋 目录

- [行为准则](#行为准则)
- [我能贡献什么](#我能贡献什么)
- [开发环境设置](#开发环境设置)
- [提交流程](#提交流程)
- [代码规范](#代码规范)
- [测试要求](#测试要求)
- [Pull Request 流程](#pull-request-流程)

---

## 行为准则

本项目采用[贡献者公约](https://www.contributor-covenant.org/)行为准则。

**我们的承诺**：
- 营造开放、友好的环境
- 尊重不同观点和经验
- 优雅地接受建设性批评
- 关注对社区最有利的事情
- 对其他社区成员表示同理心

**不可接受的行为**：
- 使用性化的语言或图像
- 人身攻击或侮辱性评论
- 公开或私下骚扰
- 未经许可发布他人信息
- 其他不道德或不专业的行为

---

## 我能贡献什么

### 报告 Bug 🐛

发现 Bug？请创建 Issue 并包含：
- 清晰的标题和描述
- 复现步骤
- 预期行为 vs 实际行为
- 环境信息（Rust 版本、操作系统等）
- 相关日志或截图

### 提出功能建议 💡

有新想法？请创建 Issue 并包含：
- 功能描述
- 使用场景
- 预期收益
- 可能的实现方案

### 提交代码 👨‍💻

欢迎各种代码贡献：
- Bug 修复
- 新功能实现
- 性能优化
- 文档改进
- 测试用例
- 代码重构

### 其他贡献 📝

- 改进文档
- 翻译本地化
- 分享使用经验
- 帮助解答问题

---

## 开发环境设置

### 1. 前置要求

- **Rust**: 1.75 或更高版本
- **Git**: 2.0 或更高版本
- **操作系统**: Linux/macOS/Windows

### 2. 克隆项目

```bash
git clone git@github.com:theteam-do/PolyMarket-knife.git
cd PolyMarket-knife
```

### 3. 安装依赖

```bash
# Rust 依赖会自动下载
cargo check
```

### 4. 编译项目

```bash
# 编译所有程序
cargo build --release

# 或编译单个程序
cargo build --release -p market-maker
```

### 5. 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定包的测试
cargo test -p poly-client
```

### 6. 代码检查

```bash
# 格式化检查
cargo fmt --check

# Clippy 检查
cargo clippy -- -D warnings
```

---

## 提交流程

### 1. Fork 仓库

在 GitHub 上 Fork 本项目到你的账户。

### 2. 创建分支

```bash
# 从 main 分支创建新分支
git checkout -b feature/your-feature-name

# 或使用 bugfix 前缀
git checkout -b bugfix/fix-issue-123
```

**分支命名规范**：
- `feature/xxx` - 新功能
- `bugfix/xxx` - Bug 修复
- `docs/xxx` - 文档更新
- `refactor/xxx` - 代码重构
- `test/xxx` - 测试相关
- `perf/xxx` - 性能优化

### 3. 进行修改

按照[代码规范](#代码规范)编写代码和测试。

### 4. 提交更改

```bash
# 添加更改的文件
git add <files>

# 提交（使用 Conventional Commits 格式）
git commit -m "type(scope): description"
```

### 5. 推送到远程

```bash
git push origin feature/your-feature-name
```

### 6. 创建 Pull Request

在 GitHub 上创建 Pull Request 到 `main` 分支。

---

## 代码规范

### Rust 代码风格

遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)：

```rust
// ✅ 好的命名
let order_book = OrderBook::new();
let max_position = calculate_max_position();

// ❌ 避免的命名
let ob = OrderBook::new();
let calc_max_pos = calculate_max_position();
```

### 代码格式化

使用 `rustfmt` 自动格式化：

```bash
# 格式化所有代码
cargo fmt

# 检查格式化
cargo fmt --check
```

### Clippy 检查

确保代码通过 Clippy 检查：

```bash
# 运行 Clippy
cargo clippy -- -D warnings
```

### Conventional Commits

提交信息遵循 [Conventional Commits](https://www.conventionalcommits.org/)：

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Type 类型**：
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 代码重构
- `test`: 测试相关
- `chore`: 构建/工具相关

**示例**：
```bash
# 新功能
git commit -m "feat(market-maker): add dynamic spread adjustment"

# Bug 修复
git commit -m "fix(arbitrage): correct profit calculation"

# 文档更新
git commit -m "docs: update API integration guide"

# 重构
git commit -m "refactor(poly-client): simplify authentication flow"
```

---

## 测试要求

### 单元测试

所有新功能必须包含单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature() {
        // Arrange
        let input = ...;
        
        // Act
        let result = new_feature(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

### 测试覆盖率

目标：核心逻辑覆盖率 > 80%

```bash
# 安装覆盖率工具
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html

# 查看报告
open tarpaulin-report.html
```

### 运行所有测试

```bash
# 运行测试
cargo test --all

# 显示测试输出
cargo test -- --nocapture

# 运行特定测试
cargo test test_name
```

---

## Pull Request 流程

### PR 模板

创建 PR 时请填写模板：

```markdown
## 描述
简要描述此 PR 的目的。

## 相关 Issue
Fixes #123

## 改动说明
- 添加了 xxx 功能
- 修复了 xxx Bug
- 重构了 xxx 模块

## 测试
- [ ] 已添加单元测试
- [ ] 所有测试通过
- [ ] 已手动测试

## 检查清单
- [ ] 代码通过 clippy 检查
- [ ] 代码已格式化
- [ ] 文档已更新
- [ ] 测试覆盖率符合要求
```

### 审核流程

1. **CI 检查** - 自动运行测试和 Clippy
2. **代码审核** - 至少 1 位维护者审核
3. **修改建议** - 根据反馈进行修改
4. **合并** - 审核通过后合并到 main 分支

### 合并策略

- 使用 **Squash and Merge** 保持提交历史清晰
- 确保 CI 全部通过
- 至少获得 1 个 approval

---

## 开发架构

### 项目结构

```
PolyMarket-knife/
├── poly-client/          # API 客户端库
├── market-maker/         # 做市策略
├── arbitrage/            # 套利策略
├── follow-trade/         # 跟单策略
├── volatility-hunter/    # 波动狩猎
├── info-edge/            # 信息差
└── order-attack/         # 订单攻击
```

### 核心依赖

- `tokio` - 异步运行时
- `serde` - 序列化
- `reqwest` - HTTP 客户端
- `rust_decimal` - 精度计算
- `tracing` - 日志

### 添加新功能

1. 在 `poly-client` 中添加 API 封装（如需要）
2. 在对应策略目录实现功能
3. 添加单元测试
4. 更新文档

---

## 发布流程

### 版本号规范

遵循 [Semantic Versioning](https://semver.org/)：

- `MAJOR.MINOR.PATCH` (e.g., 1.2.3)
- `MAJOR`: 不兼容的 API 变更
- `MINOR`: 向后兼容的功能新增
- `PATCH`: 向后兼容的 Bug 修复

### 发布步骤

1. 更新版本号（Cargo.toml）
2. 更新 CHANGELOG.md
3. 创建 Git Tag
4. 推送 Tag
5. 在 GitHub 创建 Release

---

## 常见问题

### Q: 如何开始第一个贡献？

A: 查看 [Good First Issues](https://github.com/theteam-do/PolyMarket-knife/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)

### Q: 测试失败怎么办？

A: 仔细阅读错误信息，本地复现问题，必要时添加调试日志。

### Q: 代码审核多久有反馈？

A: 通常在 1-3 个工作日内，请耐心等待。

### Q: 可以在 PR 中请求功能建议吗？

A: 可以！我们鼓励在 PR 讨论中交流想法。

---

## 联系方式

- **GitHub Issues**: [提问/讨论](https://github.com/theteam-do/PolyMarket-knife/issues)
- **Email**: developer@polymarket-knife.dev

---

## 致谢

感谢所有为本项目做出贡献的开发者！🙏

你的每一次贡献都让 PolyMarket Knife 变得更好！

---

**最后更新**: 2026-03-01
