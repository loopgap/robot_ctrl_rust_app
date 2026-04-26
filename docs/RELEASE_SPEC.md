# Release 工件规范 / Release Artifacts Specification

> 本文档定义了 Robot Control Suite 发布的标准工件格式、命名规范和质量门禁。
> 所有 Release 必须遵循此规范。

---

## 1. 版本命名规范 / Version Naming

| 格式 | 示例 | 说明 |
|------|------|------|
| `vMAJOR.MINOR.PATCH` | `v0.2.1` | 正式版本 |
| `vMAJOR.MINOR.PATCH-PRERELEASE` | `v0.2.1-beta.1` | 预发布版本 |

**语义化版本 (SemVer):**
- **MAJOR**: 不兼容的 API 变更
- **MINOR**: 向后兼容的功能新增
- **PATCH**: 向后兼容的问题修复

---

## 2. 工件列表 / Artifact List

每个 Release 必须包含以下工件：

### 2.1 Windows 工件

| 工件类型 | 命名格式 | 说明 |
|----------|----------|------|
| **NSIS 安装包** | `robot_control_suite_{VERSION}_windows_x64-setup.exe` | 带安装向导的安装包 |
| **便携版 ZIP** | `robot_control_suite_{VERSION}_windows_x64_portable.zip` | 无需安装的便携版本 |

### 2.2 Linux 工件

| 工件类型 | 命名格式 | 说明 |
|----------|----------|------|
| **DEB 安装包** | `robot_control_suite_{VERSION}_amd64.deb` | Debian/Ubuntu 系统安装包 |

### 2.3 校验文件

| 工件类型 | 命名格式 | 说明 |
|----------|----------|------|
| **SHA256 校验** | `checksums-sha256.txt` | 包含所有工件的 SHA256 校验和 |

---

## 3. 工件命名规则 / Naming Convention

```
robot_control_suite_{VERSION}_{PLATFORM}_arttype.{ext}

VERSION:   v0.2.1 (不含 v 前缀用于文件名)
PLATFORM:  windows_x64 | amd64 (Linux)
ARTYPE:    setup | portable | deb
EXT:       exe | zip | deb | txt
```

**示例:**
```
robot_control_suite_0.2.1_windows_x64-setup.exe
robot_control_suite_0.2.1_windows_x64_portable.zip
robot_control_suite_0.2.1_amd64.deb
checksums-sha256.txt
```

---

## 4. Quality Gates / 质量门禁

### 4.1 发布前检查

| 检查项 | 命令/工具 | 通过标准 |
|--------|-----------|----------|
| 代码格式 | `cargo fmt --check` | 0 errors |
| 静态分析 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 warnings |
| 单元测试 | `cargo test --workspace` | 全部通过 |
| 文档构建 | `cargo doc --workspace --no-deps` | 0 warnings |
| 安全审计 | `cargo audit` | 0 vulnerabilities |
| 依赖策略 | `cargo deny check` | 0 errors |
| Release Notes | `rusktask release-notes validate` | 结构完整 |

### 4.2 发布后验证

| 检查项 | 说明 |
|--------|------|
| 工件完整性 | 所有预期工件均已上传 |
| 校验和验证 | SHA256 与 checksums-sha256.txt 匹配 |
| 安装包运行 | Windows installer 可正常安装/卸载 |

---

## 5. GitHub Release 配置

### 5.1 Tag 格式
```
v{MAJOR}.{MINOR}.{PATCH}
```
示例: `v0.2.1`

### 5.2 Tag 必须来自 main 分支
Release workflow 的 `verify-tag` job 会验证：
```bash
git merge-base --is-ancestor HEAD origin/main
```

### 5.3 Release Notes
- 文件路径: `release_notes/RELEASE_NOTES_{TAG}.md`
- 必须包含: Highlights, Fixes, Verification
- 必须通过: `rusktask release-notes validate --mode release`

---

## 6. 发布流程 / Release Flow

```powershell
# 1. 确保 main 分支最新
git checkout main && git pull origin main

# 2. 合并 develop 到 main (如有变更)
git merge develop

# 3. 运行本地预检
./scripts/ubuntu/task.sh preflight  # Linux
.\scripts\windows\task.ps1 preflight  # Windows

# 4. 更新版本和 Release Notes
#    编辑 Cargo.toml 中的版本号
#    创建/更新 release_notes/RELEASE_NOTES_vX.Y.Z.md

# 5. 提交并打 tag
git add -A && git commit -m "chore(release): bump version to vX.Y.Z"
git tag vX.Y.Z && git push origin main --tags

# 6. GitHub Actions 自动构建和发布
#    - verify-tag: 验证 tag 格式和分支祖先
#    - quality-gate: 运行所有质量检查
#    - build-windows: 构建 Windows 工件
#    - build-linux: 构建 Linux 工件
#    - publish-release: 上传工件到 GitHub Release
```

---

## 7. 工件验收清单 / Artifact Verification Checklist

### Windows
- [ ] `robot_control_suite_{VERSION}_windows_x64-setup.exe` 存在
- [ ] `robot_control_suite_{VERSION}_windows_x64_portable.zip` 存在

### Linux
- [ ] `robot_control_suite_{VERSION}_amd64.deb` 存在

### 校验
- [ ] `checksums-sha256.txt` 存在
- [ ] 所有工件的 SHA256 校验和已记录

### Release
- [ ] GitHub Release 已创建
- [ ] Release Notes 已正确显示
- [ ] 所有工件已上传

---

## 8. 历史版本参考

| 版本 | 发布日期 | 工件 |
|------|----------|------|
| v0.2.0 | - | windows-setup.exe, linux.deb |
| v0.1.8 | - | windows-setup.exe, linux.deb |
| v0.1.7 | - | windows-setup.exe, linux.deb |

---

*本文档由 Robot Control Suite 团队维护，最后更新: 2026-04-26*
