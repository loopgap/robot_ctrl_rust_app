#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Pre-commit Hook - 提交前检查
.DESCRIPTION
    在提交前执行代码格式化和基本验证
#>

$ErrorActionPreference = "Stop"
$script:ExitCode = 0

# 获取脚本所在目录 (scripts/hooks/)
$HookDir = Split-Path -Parent $MyInvocation.MyCommand.Path
# 获取scripts目录
$ScriptsDir = Split-Path -Parent $HookDir
# 获取仓库根目录
$RepoRoot = Split-Path -Parent $ScriptsDir

# 导入通用模块
Import-Module "$ScriptsDir\common.psm1" -Force

Write-Header "Pre-commit 检查"

# 1. Git工作流验证
Write-Step "执行Git工作流验证..."
& "$ScriptsDir\git-check.ps1"
if ($LASTEXITCODE -ne 0) {
    $script:ExitCode = 1
}

# 2. Rust代码快速检查（仅检查修改的文件）
Write-Step "检查Rust代码格式..."

# 获取暂存区的Rust文件
$stagedRsFiles = git diff --cached --name-only --diff-filter=ACM | Where-Object { $_ -match "\.rs$" }

if ($stagedRsFiles) {
    Write-Info "发现 $($stagedRsFiles.Count) 个Rust文件需要检查"
    
    # 检查rustfmt
    $result = Invoke-CommandWithOutput "cargo" "fmt -- --check"
    if ($result.ExitCode -ne 0) {
        Write-Error "代码格式不符合规范"
        Write-Host "${YELLOW}运行 'cargo fmt' 修复格式问题${RESET}"
        $script:ExitCode = 1
    } else {
        Write-Success "代码格式检查通过"
    }
    
    # 快速Clippy检查
    Write-Step "执行快速Clippy检查..."
    $result = Invoke-CommandWithOutput "cargo" "clippy -- -D warnings"
    if ($result.ExitCode -ne 0) {
        Write-Error "Clippy发现警告或错误"
        $script:ExitCode = 1
    } else {
        Write-Success "Clippy检查通过"
    }
} else {
    Write-Info "没有Rust文件需要检查"
}

# 总结
Write-Header "Pre-commit 总结"
if ($script:ExitCode -eq 0) {
    Write-Success "所有检查通过，准备提交"
} else {
    Write-Error "检查未通过，提交已阻止"
    Write-Host ""
    Write-Host "${YELLOW}修复问题后重新尝试提交${RESET}"
}

exit $script:ExitCode
