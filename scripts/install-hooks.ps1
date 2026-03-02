#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Git Hooks安装脚本
.DESCRIPTION
    安装Git hooks到本地仓库，启用自动化审查流程
#>

param(
    [switch]$Uninstall,
    [switch]$Force
)

$ErrorActionPreference = "Stop"

# 颜色定义
$ESC = [char]27
$GREEN = "${ESC}[32m"
$YELLOW = "${ESC}[33m"
$CYAN = "${ESC}[36m"
$RESET = "${ESC}[0m"
$BOLD = "${ESC}[1m"

Write-Host ""
Write-Host "${BOLD}${CYAN}══════════════════════════════════════════════════════════════${RESET}"
Write-Host "${BOLD}${CYAN}  Git Hooks 安装工具${RESET}"
Write-Host "${BOLD}${CYAN}══════════════════════════════════════════════════════════════${RESET}"
Write-Host ""

# 获取路径
$ScriptDir = $PSScriptRoot
$RepoRoot = Split-Path -Parent $ScriptDir
$GitHooksDir = Join-Path $RepoRoot ".git\hooks"
$SourceHooksDir = Join-Path $ScriptDir "hooks"

# 检查Git仓库
if (-not (Test-Path "$RepoRoot\.git")) {
    Write-Error "错误: 当前目录不是Git仓库"
    exit 1
}

# 确保hooks目录存在
if (-not (Test-Path $GitHooksDir)) {
    New-Item -ItemType Directory -Path $GitHooksDir -Force | Out-Null
}

# 定义要安装的hooks
$Hooks = @(
    @{ Name = "pre-commit"; Description = "提交前检查" },
    @{ Name = "pre-push"; Description = "推送前严格检查" },
    @{ Name = "commit-msg"; Description = "提交信息验证" }
)

if ($Uninstall) {
    Write-Host "${YELLOW}卸载Git Hooks...${RESET}"
    
    foreach ($hook in $Hooks) {
        $targetPath = Join-Path $GitHooksDir $hook.Name
        $backupPath = "$targetPath.backup"
        
        if (Test-Path $targetPath) {
            Remove-Item $targetPath -Force
            Write-Host "  已移除: $($hook.Name)"
        }
        
        # 恢复备份
        if (Test-Path $backupPath) {
            Move-Item $backupPath $targetPath
            Write-Host "  已恢复备份: $($hook.Name)"
        }
    }
    
    Write-Host ""
    Write-Host "${GREEN}Git Hooks已卸载${RESET}"
    exit 0
}

Write-Host "${CYAN}安装Git Hooks...${RESET}"
Write-Host ""

foreach ($hook in $Hooks) {
    $sourcePath = Join-Path $SourceHooksDir "$($hook.Name).ps1"
    $targetPath = Join-Path $GitHooksDir $hook.Name
    $backupPath = "$targetPath.backup"
    
    # 检查源文件
    if (-not (Test-Path $sourcePath)) {
        Write-Warning "跳过: 未找到源文件 $($hook.Name).ps1"
        continue
    }
    
    # 备份现有hook
    if (Test-Path $targetPath) {
        if (-not $Force) {
            # 检查是否是我们安装的
            $content = Get-Content $targetPath -Raw -ErrorAction SilentlyContinue
            if ($content -match "自动化审查") {
                Write-Host "  更新: $($hook.Name) - $($hook.Description)"
            } else {
                Move-Item $targetPath $backupPath -Force
                Write-Host "  备份: $($hook.Name) -> $($hook.Name).backup"
            }
        } else {
            Move-Item $targetPath $backupPath -Force
            Write-Host "  强制备份: $($hook.Name) -> $($hook.Name).backup"
        }
    } else {
        Write-Host "  安装: $($hook.Name) - $($hook.Description)"
    }
    
    # 创建hook包装脚本
    $hookContent = @"
#!/bin/sh
# Git Hook: $($hook.Name)
# 由自动化审查工具生成

# Windows上使用PowerShell执行
if command -v pwsh.exe >/dev/null 2>&1; then
    exec pwsh.exe -NoProfile -ExecutionPolicy Bypass -File "$sourcePath" `$@
elif command -v powershell.exe >/dev/null 2>&1; then
    exec powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$sourcePath" `$@
else
    echo "错误: 未找到PowerShell"
    exit 1
fi
"@
    
    # 同时创建PowerShell版本（用于Windows直接执行）
    $psHookContent = @"
#!/usr/bin/env pwsh
# Git Hook: $($hook.Name)
# 由自动化审查工具生成

`$ErrorActionPreference = "Stop"
& "$sourcePath" `@args
exit `$LASTEXITCODE
"@
    
    # 写入Shell版本（Git使用）
    $hookContent | Out-File -FilePath $targetPath -Encoding UTF8 -NoNewline
    
    # 写入PowerShell版本
    $psHookPath = "$targetPath.ps1"
    $psHookContent | Out-File -FilePath $psHookPath -Encoding UTF8
}

Write-Host ""
Write-Host "${GREEN}Git Hooks安装完成！${RESET}"
Write-Host ""
Write-Host "${CYAN}已启用的Hooks:${RESET}"
foreach ($hook in $Hooks) {
    Write-Host "  ✓ $($hook.Name) - $($hook.Description)"
}
Write-Host ""
Write-Host "${YELLOW}注意:${RESET}"
Write-Host "  • 提交代码时将自动执行pre-commit检查"
Write-Host "  • 推送代码时将自动执行pre-push严格检查"
Write-Host "  • 可以使用 --no-verify 跳过检查（不推荐）"
Write-Host "  • 运行 .\scripts\review.ps1 -Help 查看更多选项"
Write-Host ""
Write-Host "${CYAN}要卸载hooks，请运行: .\scripts\install-hooks.ps1 -Uninstall${RESET}"
Write-Host ""
