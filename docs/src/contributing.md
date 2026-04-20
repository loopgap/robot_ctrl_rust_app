# 贡献指南

> 欢迎贡献 Rust Serial 工作区！

## 如何贡献

### 贡献类型

我们欢迎以下类型的贡献：

| 类型 | 说明 |
|------|------|
| Bug 修复 | 修复已知问题 |
| 新功能 | 添加新功能 |
| 文档改进 | 完善文档内容 |
| 代码优化 | 提升性能或可读性 |
| 测试用例 | 增加测试覆盖率 |

## 开发流程

### 1. Fork 仓库

点击 GitHub 页面右上角的 **Fork** 按钮。

### 2. 克隆你的 Fork

```powershell
git clone https://github.com/YOUR_USERNAME/robot_ctrl_rust_app.git
cd robot_ctrl_rust_app
```

### 3. 添加上游仓库

```powershell
git remote add upstream https://github.com/loopgap/robot_ctrl_rust_app.git
```

### 4. 切换到开发分支

```powershell
git checkout develop
git pull --ff-only origin develop
```

### 5. 进行开发

```powershell
# 确保在 develop 上开发
git checkout develop

# 进行代码修改
# ... 修改代码 ...

# 运行测试确保没有破坏现有功能
cargo test

# 运行代码检查
cargo clippy
cargo fmt
```

### 6. 提交代码

```powershell
# 添加修改的文件
git add .

# 提交 (遵循 Conventional Commits 格式)
git commit -m "feat(controller): 添加新的 PID 参数自动整定功能"
```

### 7. 同步上游变更

```powershell
# 切换到 main 分支
git checkout main

# 获取上游最新代码
git fetch upstream

# 合并到 main
git merge upstream/main

# 将 main 的变更合并到 develop（如需同步）
git checkout develop
git merge main
```

### 8. 推送分支

```powershell
# 推送 develop
git push origin develop
```

### 9. 创建 Pull Request

1. 访问原仓库页面
2. 点击 **Compare & pull request**
3. 填写 PR 描述
4. 提交 PR

## 分支使用规范

- 仅允许 `develop` 与 `main` 两个分支。
- 日常开发与集成在 `develop`，发布相关操作在 `main`。
- 不允许创建 `feature/*`、`fix/*`、`release/*`、`master` 等其他分支。

## 提交信息规范

### 格式

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### 类型 (Type)

| 类型 | 说明 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `docs` | 文档变更 |
| `style` | 代码格式（不影响功能）|
| `refactor` | 重构 |
| `perf` | 性能优化 |
| `test` | 测试相关 |
| `build` | 构建相关 |
| `ci` | CI/CD 相关 |
| `chore` | 杂项 |
| `revert` | 回退 |

### 示例

```
feat(controller): 添加神经网络参数调优功能

实现基于梯度的自动参数整定算法，支持：
- 梯度下降优化
- 自适应学习率
- 参数边界约束

Closes #123
```

## 代码规范

### Rust 代码规范

- 遵循 Rust 官方代码风格
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码
- 添加适当的文档注释

### 文档规范

- 使用中文编写文档（与现有文档保持一致）
- Markdown 格式正确
- 包含代码示例时确保可运行

### 测试规范

- 新功能必须包含测试
- Bug 修复必须包含回归测试
- 测试覆盖率不应降低

## 代码审查

### 审查清单

提交 PR 前请确保：

- [ ] 代码符合 Rust 编码规范
- [ ] 已运行 `cargo fmt` 和 `cargo clippy`
- [ ] 所有测试通过
- [ ] 新功能有适当的文档
- [ ] 提交信息符合规范

### 审查流程

1. **自动检查**: CI 会自动运行以下检查：
   - 代码格式 (`cargo fmt --check`)
   - Clippy 分析
   - 测试
   - 安全审计

2. **人工审查**: 维护者会审查代码并提供反馈

3. **合并**: 审查通过后，代码将被合并到 main 分支

## 问题反馈

### Bug 报告

报告 Bug 时请包含：

- 清晰的标题和描述
- 复现步骤
- 预期行为和实际行为
- 环境信息（操作系统、Rust 版本等）
- 相关日志或截图

### 功能请求

提交功能请求时：

- 清晰描述功能需求
- 解释为什么需要此功能
- 提供可能的使用场景
- 可以附上伪代码或设计草图

## 许可证

通过贡献代码，你同意将你的贡献以 MIT 许可证发布。

## 联系方式

- GitHub Issues: [问题追踪](../issues)
- GitHub Discussions: [讨论区](../discussions)

## 感谢贡献

感谢所有贡献者的努力！

<a href="https://github.com/loopgap/robot_ctrl_rust_app/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=loopgap/robot_ctrl_rust_app" />
</a>

## 相关文档

- [开发与工作流](workflow.md) - 项目开发流程
- [代码规范](../.github/CONTRIBUTING.md) - 详细代码规范
- [智能排障](troubleshooting.md) - 常见问题