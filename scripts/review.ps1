#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    综合审查工具入口 - 完整的CI/CD本地模拟
.DESCRIPTION
    在本地模拟完整的CI/CD流程，确保代码符合所有质量标准
.EXAMPLE
    .\review.ps1                    # 执行完整审查
    .\review.ps1 -Quick             # 快速检查（仅格式和Clippy）
    .\review.ps1 -Fix               # 自动修复格式问题
    .\review.ps1 -BeforePush        # 推送前完整检查
    .\review.ps1 -Project robot_control_rust  # 仅检查指定项目
#>

param(
    [switch]$Quick,
    [switch]$Fix,
    [switch]$BeforePush,
    [string[]]$Project,
    [switch]$SkipTests,
    [switch]$SkipAudit,
    [switch]$Help
)

if ($Help) {
    @"
Rust项目自动化审查工具

用法:
  .\review.ps1 [选项]

选项:
  -Quick          快速模式：仅检查格式和Clippy
  -Fix            自动修复格式问题
  -BeforePush     推送前完整检查（包含所有测试和构建）
  -Project <name> 仅检查指定项目（可多次使用）
  -SkipTests      跳过测试
  -SkipAudit      跳过安全审计
  -Help           显示此帮助信息

示例:
  .\review.ps1                          # 完整审查
  .\review.ps1 -Quick                   # 快速检查
  .\review.ps1 -Fix                     # 自动修复格式
  .\review.ps1 -BeforePush              # 推送前检查
  .\review.ps1 -Project robot_control_rust  # 仅检查主项目
"@ | Write-Host
    exit 0
}

$ErrorActionPreference = "Stop"
$StartTime = Get-Date

# 获取脚本所在目录
$ScriptDir = $PSScriptRoot
$RepoRoot = Split-Path -Parent $ScriptDir

# 导入通用模块
Import-Module "$ScriptDir\common.psm1" -Force

Write-Header "Rust项目自动化审查工具"
Write-Host "${CYAN}仓库路径: $RepoRoot${RESET}"
Write-Host "${CYAN}开始时间: $($StartTime.ToString('yyyy-MM-dd HH:mm:ss'))${RESET}"
Write-Host ""

# 确定运行模式
$Mode = "完整审查"
if ($Quick) { $Mode = "快速检查" }
if ($BeforePush) { $Mode = "推送前检查" }
if ($Fix) { $Mode = "自动修复模式" }

Write-Host "${BOLD}运行模式: $Mode${RESET}"
Write-Host ""

# 设置参数
$rustReviewArgs = @()
if ($Fix) { $rustReviewArgs += "-Fix" }
if ($SkipTests) { $rustReviewArgs += "-SkipTests" }
if ($SkipAudit) { $rustReviewArgs += "-SkipAudit" }
if ($Project) { 
    foreach ($p in $Project) {
        $rustReviewArgs += "-Projects"
        $rustReviewArgs += $p
    }
}

$script:ExitCode = 0

# 执行审查流程
try {
    Push-Location $RepoRoot
    
    if ($Quick) {
        # 快速模式
        Write-Header "快速检查"
        
        # 仅格式和Clippy
        Write-Step "检查代码格式..."
        $fmtCheck = if ($Fix) { "fmt" } else { "fmt -- --check" }
        $result = Invoke-CommandWithOutput "cargo" $fmtCheck
        if ($result.ExitCode -ne 0) {
            if ($Fix) {
                Write-Success "代码已自动格式化"
            } else {
                Write-Error "代码格式不符合规范"
                $script:ExitCode = 1
            }
        } else {
            Write-Success "代码格式检查通过"
        }
        
        Write-Step "执行Clippy检查..."
        $result = Invoke-CommandWithOutput "cargo" "clippy -- -D warnings"
        if ($result.ExitCode -ne 0) {
            Write-Error "Clippy发现警告或错误"
            $script:ExitCode = 1
        } else {
            Write-Success "Clippy检查通过"
        }
    }
    else {
        # 完整审查
        Write-Header "Git工作流验证"
        & "$ScriptDir\git-check.ps1" -PrePush:$BeforePush
        if ($LASTEXITCODE -ne 0) {
            $script:ExitCode = 1
        }
        
        Write-Header "Rust代码审查"
        & "$ScriptDir\rust-review.ps1" @rustReviewArgs
        if ($LASTEXITCODE -ne 0) {
            $script:ExitCode = 1
        }
        
        if ($BeforePush) {
            Write-Header "发布构建测试"
            $projects = if ($Project) { $Project } else { Get-ProjectDirs }
            
            foreach ($proj in $projects) {
                Write-Step "构建项目: $proj"
                $result = Invoke-CommandWithOutput "cargo" "build --release" $proj
                if ($result.ExitCode -ne 0) {
                    Write-Error "项目 $proj 发布构建失败"
                    $script:ExitCode = 1
                } else {
                    Write-Success "项目 $proj 发布构建成功"
                }
            }
        }
    }
    
    # 统计时间
    $EndTime = Get-Date
    $Duration = $EndTime - $StartTime
    
    Write-Header "审查完成"
    Write-Host "${CYAN}结束时间: $($EndTime.ToString('yyyy-MM-dd HH:mm:ss'))${RESET}"
    Write-Host "${CYAN}耗时: $($Duration.Minutes)分 $($Duration.Seconds)秒${RESET}"
    Write-Host ""
    
    if ($script:ExitCode -eq 0) {
        Write-Host "${GREEN}${BOLD}✓ 所有检查通过！${RESET}"
        if ($BeforePush) {
            Write-Host "${GREEN}代码质量优秀，可以安全推送${RESET}"
        }
    } else {
        Write-Host "${RED}${BOLD}✗ 检查未通过${RESET}"
        Write-Host ""
        Write-Host "${YELLOW}请修复上述问题后重试${RESET}"
        if (-not $Fix) {
            Write-Host "${CYAN}提示: 使用 -Fix 参数自动修复格式问题${RESET}"
        }
    }
}
finally {
    Pop-Location
}

exit $script:ExitCode
