#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Pre-push Hook - 推送前严格检查
.DESCRIPTION
    在推送前执行 Git 工作流检查和完整 Rust 审查，阻止问题进入远程。
#>

$ErrorActionPreference = "Stop"

# 获取脚本所在目录 (scripts/hooks/)
$HookDir = Split-Path -Parent $MyInvocation.MyCommand.Path
# 获取 scripts 目录
$ScriptsDir = Split-Path -Parent $HookDir

# 导入通用模块
Import-Module "$ScriptsDir\common.psm1" -Force

Write-Header "Pre-push Check"

# 1) Git 工作流校验（包含远程同步状态）
& "$ScriptsDir\git-check.ps1" -PrePush
if ($LASTEXITCODE -ne 0) {
    Write-Error "Git 工作流检查失败，已阻止 push"
    exit $LASTEXITCODE
}

# 2) 完整 Rust 审查（推送前模式）
& "$ScriptsDir\review.ps1" -BeforePush
if ($LASTEXITCODE -ne 0) {
    Write-Error "Rust 推送前检查失败，已阻止 push"
    exit $LASTEXITCODE
}

Write-Success "Pre-push checks passed"
exit 0
