# 发布操作手册

## 目标

确保每次发布均通过同一条受控路径：版本推进、Tag、CI/CD 构建、资产校验。

## 前置条件

1. 当前分支为 main 或 master。
2. 本地工作区干净。
3. 本地质量门禁通过：

```powershell
.\make.ps1 check
```

## 标准发布流程

1. 生成版本提交与 Tag：

```powershell
.\scripts\smart-bump.ps1 -Part patch
```

2. 推送分支与 Tag（可按需要跳过本地 hook）：

```powershell
.\scripts\smart-bump.ps1 -Part patch -Push -NoVerify
```

3. 等待 Release 工作流完成，确认资产：
- robot_control_rust.exe
- RobotControlSuite_Setup.exe
- checksums-sha256.txt

## 质量门禁说明

Release 工作流会在发布前执行：

1. Tag 规则校验。
2. fmt、clippy、test、doc 全量检查。
3. Windows 构建 + smoke test（参数启动、超时、退出码）。
4. 资产存在性校验。

## 发布后验收

1. 校验 Release 页面包含 3 个必需资产。
2. 下载 checksums-sha256.txt 对 exe 与 setup 做 SHA256 校验。
3. 记录发布链接和版本号到变更日志。
