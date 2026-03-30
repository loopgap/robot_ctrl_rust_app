# 自动化审查工具

本项目配置了完整的自动化审查工具链，确保代码质量和Git工作流规范。

## 快速开始

### 1. 安装Git Hooks

```powershell
.\scripts\install-hooks.ps1
```

这将在你的本地Git仓库中安装以下hooks：
- **pre-commit**: 提交前基础检查
- **pre-push**: 推送前严格检查
- **commit-msg**: 提交信息格式验证

### 2. 运行审查工具

```powershell
# 完整审查
.\scripts\review.ps1

# 快速检查（仅格式和Clippy）
.\scripts\review.ps1 -Quick

# 自动修复格式问题
.\scripts\review.ps1 -Fix

# 推送前完整检查
.\scripts\review.ps1 -BeforePush

# 仅检查指定项目
.\scripts\review.ps1 -Project robot_control_rust
```

### 3. 版本与发布

```powershell
# 仅创建版本提交与 tag（不推送）
.\scripts\smart-bump.ps1 -Part patch

# 创建后立即推送分支和 tag
.\scripts\smart-bump.ps1 -Part patch -Push
```

`smart-bump.ps1` 默认只允许在 `main/master` 上执行，并会阻止重复 tag。

## Git Hooks说明

### Pre-commit（提交前）

在每次执行 `git commit` 时自动运行：

1. **Git工作流验证**
   - 检查分支命名规范
   - 检查暂存区文件（大文件、敏感信息）

2. **Rust代码快速检查**
   - 代码格式检查 (rustfmt)
   - 静态分析 (clippy)

**执行时间**: 约5-15秒

### Pre-push（推送前）⭐ 更严格

在每次执行 `git push` 时自动运行：

1. **Git工作流验证（推送模式）**
   - 检查分支保护规则
   - 检查远程同步状态
   - 阻止直接推送到main/master分支

2. **完整Rust代码审查**
   - 代码格式检查
   - Clippy静态分析（包含pedantic和nursery规则）
   - 单元测试和集成测试
   - 构建检查
   - 安全审计 (cargo-audit)

3. **发布构建测试**
   - 验证release模式构建

4. **文档检查**
   - 验证文档构建

**执行时间**: 约30秒-2分钟（取决于项目大小）

### Commit-msg（提交信息）

验证提交信息格式是否符合规范：

**支持的格式**:
```
# Conventional Commits
type(scope): description
feat(controller): 添加PID参数调节功能
fix(connection): 修复串口连接超时问题

# 详细格式
[模块] 描述
[控制算法] 优化PID计算性能
[UI] 添加深色模式支持
```

**支持的类型**: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert

## 审查流程图

```
开发工作流:
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
                              否 ◄────────────┼────────────► 是
                                              │
                                    ┌─────────▼─────────┐
                                    │ 提交成功          │
                                    └─────────┬─────────┘
                                              │
                                    ┌─────────▼─────────┐
                                    │ git push          │
                                    └─────────┬─────────┘
                                              │
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
                              否 ◄────────────┼────────────► 是
                                              │
                                    ┌─────────▼─────────┐
                                    │ 推送成功 ✓        │
                                    └───────────────────┘
```

## 分支策略

### 受保护分支

以下分支不允许直接推送：
- `main`
- `master`
- `release/*`

**请使用Pull Request进行代码合并！**

### 分支命名规范

```
feature/功能描述     # 新功能
fix/修复描述         # Bug修复
docs/文档描述        # 文档更新
refactor/重构描述    # 代码重构
test/测试描述        # 测试相关
chore/杂项描述       # 构建/工具等
```

## 配置说明

审查配置存储在 `scripts/review-config.json` 中，可以根据项目需求进行调整：

- **hooks**: 启用/禁用特定hooks
- **rust**: Rust工具链配置
- **git**: Git工作流规则

## 故障排除

### 检查失败怎么办？

1. **格式问题**: 运行 `.\scripts\review.ps1 -Fix` 自动修复
2. **Clippy警告**: 根据提示修改代码
3. **测试失败**: 修复测试用例或代码逻辑
4. **提交信息**: 按照规范格式重写提交信息

### 跳过检查（不推荐）

在紧急情况下可以使用 `--no-verify` 跳过检查：

```bash
git commit -m "紧急修复" --no-verify
git push --no-verify
```

**注意**: 跳过检查可能导致代码质量问题，请谨慎使用！

### 卸载Hooks

```powershell
.\scripts\install-hooks.ps1 -Uninstall
```

## 工具依赖

### 必需
- Rust工具链 (cargo, rustfmt, clippy)
- PowerShell 7.0+

### 可选
- cargo-audit: 安全审计
- cargo-deny: 依赖检查

安装可选工具：
```bash
cargo install cargo-audit
cargo install cargo-deny
```
