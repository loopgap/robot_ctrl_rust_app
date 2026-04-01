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

function Get-ScriptExitCode {
    if ($null -eq $LASTEXITCODE) {
        if ($?) {
            return 0
        }
        return 1
    }
    return [int]$LASTEXITCODE
}

Write-Header "Pre-push Check"

# 0) 工作区过程文件与路径策略检查
$global:LASTEXITCODE = 0
& "$ScriptsDir\cleanup-process-files.ps1" -Mode audit -Strict
$exitCode = Get-ScriptExitCode
if ($exitCode -ne 0) {
    Write-Error "发现过程文件残留，已阻止 push。请先执行: ./make.ps1 workspace-cleanup"
    exit $exitCode
}

$global:LASTEXITCODE = 0
& "$ScriptsDir\enforce-workspace-structure.ps1" -Mode audit -Strict -UseStagedPaths
$exitCode = Get-ScriptExitCode
if ($exitCode -ne 0) {
    Write-Error "发现不合规目录或暂存路径，已阻止 push。"
    exit $exitCode
}

# 1) Git 工作流校验（包含远程同步状态）
$global:LASTEXITCODE = 0
& "$ScriptsDir\git-check.ps1" -PrePush
$exitCode = Get-ScriptExitCode
if ($exitCode -ne 0) {
    Write-Error "Git 工作流检查失败，已阻止 push"
    exit $exitCode
}

# 2) 完整 Rust 审查（推送前模式）
$global:LASTEXITCODE = 0
& "$ScriptsDir\review.ps1" -BeforePush
$exitCode = Get-ScriptExitCode
if ($exitCode -ne 0) {
    Write-Error "Rust 推送前检查失败，已阻止 push"
    exit $exitCode
}

Write-Success "Pre-push checks passed"
exit 0
