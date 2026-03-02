#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Rust代码审查脚本 - 执行格式化、Clippy检查、测试和安全审计
.DESCRIPTION
    对所有Rust项目执行完整的代码审查流程
#>

param(
    [switch]$Fix,
    [switch]$SkipTests,
    [switch]$SkipAudit,
    [string[]]$Projects = @()
)

$ErrorActionPreference = "Stop"
$script:ExitCode = 0

# 导入通用模块
Import-Module "$PSScriptRoot\common.psm1" -Force

Write-Header "Rust代码审查工具"

# 获取项目列表
if ($Projects.Count -eq 0) {
    $Projects = Get-ProjectDirs
}

if ($Projects.Count -eq 0) {
    Write-Error "未找到任何Rust项目"
    exit 1
}

Write-Info "发现 $($Projects.Count) 个项目:"
$Projects | ForEach-Object { Write-Info "  - $_" }

# 检查必要工具
Write-Header "检查工具链"
$tools = @("cargo", "rustfmt", "cargo-clippy")
if (-not $SkipAudit) { $tools += "cargo-audit" }

foreach ($tool in $tools) {
    $result = Invoke-CommandWithOutput "where.exe" $tool
    if ($result.ExitCode -eq 0) {
        Write-Success "$tool 已安装"
    } else {
        Write-Error "$tool 未安装"
        $script:ExitCode = 1
    }
}

if ($script:ExitCode -ne 0) {
    Write-Host ""
    Write-Error "缺少必要工具，请安装: rustup component add rustfmt clippy"
    if (-not $SkipAudit) {
        Write-Info "安装 cargo-audit: cargo install cargo-audit"
    }
    exit 1
}

# 对每个项目执行审查
foreach ($project in $Projects) {
    Write-Header "审查项目: $project"
    $projectPath = Resolve-Path $project
    
    # 1. 代码格式化检查
    Write-Step "检查代码格式 (rustfmt)..."
    $fmtArgs = if ($Fix) { "fmt" } else { "fmt -- --check" }
    $result = Invoke-CommandWithOutput "cargo" $fmtArgs $projectPath
    
    if ($result.ExitCode -eq 0) {
        Write-Success "代码格式检查通过"
    } else {
        if ($Fix) {
            Write-Success "代码已自动格式化"
        } else {
            Write-Error "代码格式不符合规范"
            Write-Host $result.StdErr
            $script:ExitCode = 1
        }
    }
    
    # 2. Clippy静态分析
    Write-Step "执行Clippy静态分析..."
    $clippyArgs = "clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo"
    $result = Invoke-CommandWithOutput "cargo" $clippyArgs $projectPath
    
    if ($result.ExitCode -eq 0) {
        Write-Success "Clippy检查通过"
    } else {
        Write-Error "Clippy发现警告或错误"
        Write-Host $result.StdOut
        Write-Host $result.StdErr
        $script:ExitCode = 1
    }
    
    # 3. 运行测试
    if (-not $SkipTests) {
        Write-Step "运行单元测试和集成测试..."
        $result = Invoke-CommandWithOutput "cargo" "test --all-features" $projectPath
        
        if ($result.ExitCode -eq 0) {
            # 解析测试结果
            $passed = ($result.StdOut | Select-String "test result: ok").Matches.Count
            $failed = ($result.StdOut | Select-String "test result: FAILED").Matches.Count
            
            if ($failed -eq 0) {
                Write-Success "所有测试通过"
            } else {
                Write-Error "有测试失败"
                $script:ExitCode = 1
            }
        } else {
            Write-Error "测试执行失败"
            Write-Host $result.StdOut
            Write-Host $result.StdErr
            $script:ExitCode = 1
        }
    } else {
        Write-Warning "跳过测试"
    }
    
    # 4. 构建检查
    Write-Step "检查构建..."
    $result = Invoke-CommandWithOutput "cargo" "check --all-features" $projectPath
    
    if ($result.ExitCode -eq 0) {
        Write-Success "构建检查通过"
    } else {
        Write-Error "构建失败"
        Write-Host $result.StdErr
        $script:ExitCode = 1
    }
    
    # 5. 安全审计
    if (-not $SkipAudit) {
        Write-Step "执行安全审计 (cargo-audit)..."
        $result = Invoke-CommandWithOutput "cargo" "audit" $projectPath
        
        if ($result.ExitCode -eq 0) {
            Write-Success "安全审计通过"
        } else {
            Write-Error "发现安全漏洞"
            Write-Host $result.StdOut
            $script:ExitCode = 1
        }
    }
}

# 总结
Write-Header "审查总结"
if ($script:ExitCode -eq 0) {
    Write-Success "所有检查通过！"
    Write-Host ""
    Write-Host "${GREEN}${BOLD}代码质量良好，可以提交${RESET}"
} else {
    Write-Error "审查未通过，请修复上述问题"
    Write-Host ""
    Write-Host "${YELLOW}提示: 使用 -Fix 参数自动修复格式问题${RESET}"
}

exit $script:ExitCode
