# 开发与工作流说明

> 规范化的 Git 工作流与自动化工具链

## Git 工作流规范

### 分支命名规范

| 前缀 | 用途 | 示例 |
|------|------|------|
| `feature/` | 新功能开发 | `feature/pid-tuning-ui` |
| `fix/` | Bug 修复 | `fix/serial-timeout` |
| `docs/` | 文档更新 | `docs/update-api` |
| `refactor/` | 代码重构 | `refactor/connection-manager` |
| `test/` | 测试相关 | `test/add-integration-tests` |
| `chore/` | 构建/工具/杂项 | `chore/update-deps` |

### 受保护分支

以下分支不允许直接推送，必须通过 Pull Request：

- `main`
- `master`
- `release/*`

### 提交信息格式

遵循 Conventional Commits 规范：

```
<type>(<scope>): <description>

feat(controller): 添加 PID 参数调节功能
fix(connection): 修复串口连接超时问题
docs(readme): 更新安装说明
```

**支持的 Type**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档变更
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `build`: 构建相关
- `ci`: CI/CD 相关
- `chore`: 杂项
- `revert`: 回退

## 自动化工具链

### 工具概览

| 工具 | 用途 | 配置文件 |
|------|------|----------|
| `rusktask review` | 代码审查入口 | `review-config.json` |
| `make.ps1 preflight` | 预检入口 | - |
| `preflight.sh` | Linux/macOS 预检 | - |
| `rusktask install-hooks` | Git Hooks 安装 | `review-config.json` |
| `rusktask smart-bump` | 版本智能升级 | - |

### Git Hooks

安装后自动在以下时机执行检查：

#### Pre-commit (提交前检查)

**执行时间**: 约 5-15 秒

检查项目：
- Git 工作流验证（分支命名、暂存区文件）
- `cargo fmt` 代码格式检查
- `cargo clippy --fix` 快速静态分析

#### Pre-push (推送前检查) ⭐ 更严格

**执行时间**: 约 30 秒 - 2 分钟

检查项目：
- 分支保护规则验证
- 远程同步状态检查
- 完整 Rust 代码审查（格式、Clippy、测试、构建）
- `cargo audit` 安全审计
- Release 模式构建验证

#### Commit-msg (提交信息验证)

验证提交信息是否符合 Conventional Commits 规范。

## 本地开发脚本

### 安装 Git Hooks

```powershell
# 安装
.\scripts\task.ps1 go-install-hooks

# 卸载
cd .\scripts\go\rusktask; go run . install-hooks --uninstall
```

### 代码审查

```powershell
# 完整审查
cd .\scripts\go\rusktask; go run . review

# 快速检查（仅格式和 Clippy）
go run . review --quick

# 自动修复格式问题
go run . review --quick --fix

# 推送前完整检查
go run . review --before-push

# 检查指定项目
go run . review --project robot_control_rust
```

### 预检脚本

```powershell
# Windows
.\scripts\task.ps1 preflight

# Linux/macOS
./scripts/task preflight
```

### 版本智能升级

```powershell
# 生成版本提交和 tag
.\scripts\task.ps1 smart-bump -BumpPart patch

# 推送分支和 tag（按需跳过本地 pre-push）
.\scripts\task.ps1 smart-bump -BumpPart patch -BumpPush -BumpNoVerify

# 失败回滚
.\scripts\task.ps1 smart-rollback -RollbackTag vX.Y.Z -RollbackDeleteRemoteTag -RollbackDeleteLocalTag -RollbackRevertLastCommit -RollbackPushRevert -RollbackNoVerify
```

支持：
- Semantic Versioning 自动升号
- Release notes 草稿生成
- Annotated tag 生成

## CI/CD 流水线

### GitHub Actions 工作流

| 工作流 | 触发条件 | 核心能力 |
|--------|----------|----------|
| **CI** | PR / push 到 main/develop | 格式检查、Clippy、测试、文档阻断 |
| **Security Audit** | 每周一 / 依赖变更 / 手动触发 | cargo-audit 与 cargo-deny 门禁 |
| **Release** | push tag v* | Tag 校验、质量门禁、Windows 资产发布（exe/setup/checksums） |

### CI 工作流详情

```yaml
# 触发条件
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]
```

**检查步骤**:
1. 格式检查 (`cargo fmt --check`)
2. Clippy 静态分析（`--all-targets -- -D warnings`）
3. 单元测试 (`cargo test`)
4. 文档检查 (`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`)

### Release 工作流详情

```yaml
# 触发条件
on:
  push:
    tags:
      - 'v*'
```

**构建目标**:
- Windows x64 (`robot_control_rust_windows_x64_portable.zip`、`rust_tools_suite_windows_x64_portable.zip`、`RobotControlSuite_Setup.exe`)
- Linux x64 (`rust_tools_suite_linux_amd64.deb`)
- 完整性校验 (`checksums-sha256.txt`)

## 故障排除

### 检查失败怎么办？

| 问题类型 | 解决方案 |
|----------|----------|
| 格式问题 | `cd .\scripts\go\rusktask; go run . review --quick --fix` |
| Clippy 警告 | 根据提示修改代码 |
| 测试失败 | 修复测试用例或代码逻辑 |
| 提交信息格式 | 按规范重写提交信息 |
| 推送被拒绝 | 使用 PR 合并到受保护分支 |

### 跳过检查（紧急情况）

```bash
# 跳过 Git Hooks
git commit -m "紧急修复" --no-verify
git push --no-verify
```

**注意**: 跳过检查可能导致代码质量问题，请谨慎使用！

## 开发流程图

```
┌─────────────────────────────────────────────────────────────┐
│                      开发工作流                              │
└─────────────────────────────────────────────────────────────┘

1. 创建功能分支
   git checkout -b feature/your-feature

2. 开发与提交
   ┌─────────────┐    ┌──────────────┐    ┌─────────────┐
   │  代码修改   │ -> │ git add      │ -> │ git commit  │
   └─────────────┘    └──────────────┘    └──────┬──────┘
                                                  │
                                        ┌─────────▼─────────┐
                                        │   pre-commit      │
                                        │  ├─ Git检查       │
                                        │  ├─ 格式检查      │
                                        │  └─ Clippy快速    │
                                        └─────────┬─────────┘
                                                  │
                                        ┌─────────▼─────────┐
                                        │  检查通过?        │
                                        └─────────┬─────────┘
                                                  │
                              否 ◄───────────────┼────────────► 是
                                              │
                                    ┌─────────▼─────────┐
                                    │ 提交成功          │
                                    └─────────┬─────────┘
                                              │
3. 推送分支
                                    ┌─────────▼─────────┐
                                    │ git push          │
                                    └─────────┬─────────┘
                                              │
4. 创建 Pull Request
                                    ┌─────────▼─────────┐
                                    │   pre-push ⭐     │
                                    │  ├─ Git工作流     │
                                    │  ├─ 完整代码审查  │
                                    │  ├─ 运行所有测试  │
                                    │  ├─ 安全审计      │
                                    │  └─ 发布构建      │
                                    └─────────┬─────────┘
                                              │
                                    ┌─────────▼─────────┐
                                    │ 检查通过?        │
                                    └─────────┬─────────┘
                                              │
                          否 ◄───────────────┼────────────► 是
                                              │
5. Code Review 与合并
                                    ┌─────────▼─────────┐
                                    │ PR Review & Merge  │
                                    └───────────────────┘
```

## 相关文档

- [快速入门](getting-started.md) - 环境准备与首次运行
- [智能排障百科](troubleshooting.md) - 常见问题与解决方案