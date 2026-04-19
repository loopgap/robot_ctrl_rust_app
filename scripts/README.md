# 自动化任务入口

`scripts/` 目录已完成一轮去 PowerShell 化迁移：

- 质量门禁、工作区治理、发布状态治理、Git Hooks、打包与发布命令统一迁移到 `scripts/go/rusktask`。
- 已删除历史兼容脚本（如 `review.ps1`、`install-hooks.ps1`、`sync-release-state.ps1`、`workflow-seal.ps1` 等）。
- 版本升级、发布回滚、PR 辅助也已迁移为 Go 子命令（`smart-bump`、`smart-rollback`、`pr-helper`）。

## 推荐使用方式

### 1) 根目录统一入口（Windows）

```powershell
.\scripts\task.ps1 go-install-hooks
.\scripts\task.ps1 go-review
.\scripts\task.ps1 workflow-seal
.\scripts\task.ps1 release-notes-validate -ReleaseNotesFile .\release_notes\RELEASE_NOTES_vX.Y.Z.md -ReleaseNotesMode release
.\scripts\task.ps1 docs-bundle -DocsCreateZip
.\scripts\task.ps1 smart-bump -BumpPart patch
.\scripts\task.ps1 smart-rollback -RollbackTag vX.Y.Z -RollbackDeleteRemoteTag -RollbackDeleteLocalTag
.\scripts\task.ps1 pr-helper -PrCheck
.\scripts\task.ps1 build-release-slim
.\scripts\task.ps1 package-windows-installer -PackageVersion X.Y.Z
.\scripts\task.ps1 package-windows-assets -PackageVersion X.Y.Z -PackageOutputDir release_artifacts
.\scripts\task.ps1 package-windows-portable-installer -PackageVersion X.Y.Z
.\scripts\task.ps1 release-publish -ReleaseTag vX.Y.Z
```

### 2) 直接调用 Go 编排器

```powershell
cd .\scripts\go\rusktask

go run . review --before-push
go run . install-hooks
go run . install-hooks --uninstall
go run . release-sync --mode audit --strict
go run . workflow-seal --mode audit
go run . release-notes validate --file ..\..\release_notes\RELEASE_NOTES_vX.Y.Z.md --mode release
go run . docs-bundle --create-zip
go run . smart-bump --part patch
go run . smart-rollback --tag vX.Y.Z --delete-remote-tag --delete-local-tag
go run . pr-helper --check
go run . build-release-slim
go run . release-publish --tag vX.Y.Z
```

PowerShell 仅保留根入口 `make.ps1`，用于 Windows 下的统一任务调用体验。
