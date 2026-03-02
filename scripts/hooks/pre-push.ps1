#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Pre-push Hook - 推送前严格检查
.DESCRIPTION
    在推送到远程仓库前执行完整的测试和审查流程
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

Write-Header "Pre-push 严格检查"
Write-Host "${YELLOW}此检查比pre-commit更严格，确保代码质量${RESET}"

# 1. Git工作流验证（推送模式）
Write-Step "执行Git工作流验证（推送模式）..."
& "$ScriptsDir\git-check.ps1" -PrePush
if ($LASTEXITCODE -ne 0) {
    $script:ExitCode = 1
}

# 2. 完整的Rust代码审查
Write-Step "执行完整代码审查..."
& "$ScriptsDir\rust-review.ps1"
if ($LASTEXITCODE -ne 0) {
    $script:ExitCode = 1
}

# 3. 发布构建测试
Write-Step "测试发布构建..."
$projects = Get-ProjectDirs

foreach ($project in $projects) {
    Write-Info "构建项目: $project"
    $result = Invoke-CommandWithOutput "cargo" "build --release" $project
    
    if ($result.ExitCode -ne 0) {
        Write-Error "项目 $project 发布构建失败"
        $script:ExitCode = 1
    } else {
        Write-Success "项目 $project 发布构建成功"
    }
}

# 4. 检查文档
Write-Step "检查文档构建..."
foreach ($project in $projects) {
    $result = Invoke-CommandWithOutput "cargo" "doc --no-deps" $project
    if ($result.ExitCode -ne 0) {
        Write-Warning "项目 $project 文档构建有警告"
    }
}

# 5. 检查未提交的更改
Write-Step "检查工作区状态..."
$status = git status --porcelain
if ($status) {
    Write-Warning "工作区有未提交的更改:"
    Write-Host $status
    Write-Host ""
    Write-Host "${YELLOW}建议提交所有更改后再推送${RESET}"
}

# 总结
Write-Header "Pre-push 总结"
if ($script:ExitCode -eq 0) {
    Write-Success "所有严格检查通过！"
    Write-Host ""
    Write-Host "${GREEN}${BOLD}代码质量优秀，可以安全推送${RESET}"
} else {
    Write-Error "严格检查未通过，推送已阻止"
    Write-Host ""
    Write-Host "${YELLOW}请修复所有问题后再尝试推送${RESET}"
    Write-Host "${CYAN}提示: 使用 .\scripts\rust-review.ps1 -Fix 自动修复格式问题${RESET}"
}

exit $script:ExitCode
