#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Commit-msg Hook - 提交信息检查
.DESCRIPTION
    验证提交信息格式是否符合规范
#>

param(
    [Parameter(Position=0)]
    [string]$CommitMsgFile
)

if (-not $CommitMsgFile) {
    Write-Host "错误: 未提供提交信息文件路径"
    exit 1
}

# 获取脚本所在目录 (scripts/hooks/)
$HookDir = Split-Path -Parent $MyInvocation.MyCommand.Path
# 获取scripts目录
$ScriptsDir = Split-Path -Parent $HookDir

# 导入通用模块
Import-Module "$ScriptsDir\common.psm1" -Force

Write-Header "Commit Message 检查"

# 执行提交信息验证
& "$ScriptsDir\git-check.ps1" -CommitMsgFile $CommitMsgFile

exit $LASTEXITCODE
