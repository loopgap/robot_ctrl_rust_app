# 回滚操作手册

## 适用场景

1. Tag 已推送，但 Release 构建失败。
2. 发布资产不完整或校验失败。
3. 发布版本号错误，需要撤销。

## 快速回滚流程

### 1) 删除失败 Release（可选）

需要先设置 GITHUB_TOKEN。

```powershell
$env:GITHUB_TOKEN = "<token>"
```

### 2) 删除错误 Tag（本地 + 远端）

```powershell
.\scripts\smart-rollback.ps1 -Tag vX.Y.Z -DeleteRemoteTag -DeleteLocalTag -NoVerify
```

### 3) 回退最近版本提交（可选）

```powershell
.\scripts\smart-rollback.ps1 -Tag vX.Y.Z -RevertLastCommit -PushRevert -NoVerify
```

### 4) 一次性回滚（常用）

```powershell
.\scripts\smart-rollback.ps1 -Tag vX.Y.Z -DeleteRelease -DeleteRemoteTag -DeleteLocalTag -RevertLastCommit -PushRevert -NoVerify
```

## 回滚后检查

1. 远端 Tag 不存在。
2. Release 页面无对应版本或仅保留 draft 记录。
3. main/master 分支版本号恢复到预期。
4. 重新执行 .\make.ps1 check 后再发版。

## 注意事项

1. 回滚脚本不会强制改写历史，不使用 reset --hard。
2. 若已通知外部用户下载，请额外发出撤回公告。
3. 对已发布资产建议保留审计记录。
