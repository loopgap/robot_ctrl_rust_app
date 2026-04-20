# 发布操作手册

## 目标
 
确保每次发布均通过同一条受控路径：版本推进、Tag、CI/CD 构建、资产校验。

## 前置条件

1. 当前分支为 main。
2. 本地工作区干净。
3. 本地质量门禁通过：

```powershell
.\scripts\task.ps1 workflow-seal
.\scripts\task.ps1 check
```

4. 发布状态一致性审计通过（tag/发布说明/归档）：

```powershell
.\scripts\task.ps1 release-sync
```

## 标准发布流程

1. 生成版本提交与 Tag：

```powershell
.\scripts\task.ps1 smart-bump -BumpPart patch
```

2. 推送分支与 Tag（可按需要跳过本地 hook）：

```powershell
.\scripts\task.ps1 smart-bump -BumpPart patch -BumpPush -BumpNoVerify
```

3. 发布前对说明文件执行严格校验：

```powershell
.\scripts\task.ps1 release-notes-validate -ReleaseNotesFile .\release_notes\RELEASE_NOTES_vX.Y.Z.md -ReleaseNotesMode release
```

4. 等待 Release 工作流完成，确认资产：
- robot_control_suite_*_windows_x64-setup.exe
- robot_control_suite_*_amd64.deb
- checksums-sha256.txt

5. 确认远端 Release 正文与本地 `release_notes/RELEASE_NOTES_vX.Y.Z.md` 一致（正文以该文件为准）。

## 目录约定

1. 发布说明统一维护在 `release_notes/RELEASE_NOTES_vX.Y.Z.md`。
2. 发布索引统一维护在 `release_notes/RELEASE_INDEX.md`（记录版本、Tag、本地/远端 Tag 状态、本地归档状态）。
3. 历史已发布二进制资产归档在 `release_notes/archive_assets/vX.Y.Z/`。
4. 根目录 `release_artifacts/` 与 `smoke_logs/` 属于发布流程临时产物目录，不入库。

## 索引维护

标准升版脚本会自动更新发布索引；也可手动重建：

```powershell
.\scripts\task.ps1 release-index
```

如需批量清理无效本地迭代残留并归一化：

```powershell
.\scripts\task.ps1 release-sync-apply
```

每次发布前后建议执行一次过程文件清理与目录守卫：

```powershell
.\scripts\task.ps1 workflow-seal
```

治理策略来源：`scripts/workspace-governance.json`，用于统一约束目录结构与过程文件路径。

## 手动发布（可选）

默认会从统一目录读取发布说明和资产：

```powershell
.\scripts\task.ps1 release-publish -ReleaseTag vX.Y.Z
```

如需覆盖路径：

```powershell
.\scripts\task.ps1 release-publish \
	-ReleaseTag vX.Y.Z \
	-ReleaseBodyFile release_notes/RELEASE_NOTES_vX.Y.Z.md \
	-ReleaseAsset release_artifacts/robot_control_rust_windows_x64_portable.zip \
	-ReleaseAsset release_artifacts/rust_tools_suite_windows_x64_portable.zip \
	-ReleaseAsset release_artifacts/RobotControlSuite_Setup.exe \
	-ReleaseAsset release_artifacts/checksums-sha256.txt
```

## 质量门禁说明

Release 工作流会在发布前执行：

1. Tag 规则校验。
2. fmt、clippy、test、doc 全量检查。
3. Windows 构建 + smoke test（参数启动、超时、退出码）。
4. 资产存在性校验。
5. Release 正文文件存在性与非空校验。
6. Release 正文结构与完成度校验（无占位文本、Verification 勾选完成）。

## 发布后验收

1. 校验 Release 页面包含必需资产（Windows 安装包、Linux deb、checksums）。
2. 下载 checksums-sha256.txt 并对 .exe/.deb 做 SHA256 校验。
3. 记录发布链接和版本号到变更日志。

## v0.2.1 专项验证

1. `v0.2.1` Tag 需通过分支祖先校验（可追溯到 `origin/main`）。
2. `release_notes/RELEASE_NOTES_v0.2.1.md` 必须存在且通过结构校验。
3. 校验 `checksums-sha256.txt` 包含 .exe 与 .deb 两类资产散列值。
