#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Git工作流验证脚本 - 检查提交规范和分支策略
.DESCRIPTION
    验证Git提交信息和分支策略是否符合团队规范
#>

param(
    [switch]$PrePush,
    [string]$CommitMsgFile = ""
)

$ErrorActionPreference = "Stop"
$script:ExitCode = 0

Import-Module "$PSScriptRoot\common.psm1" -Force

Write-Header "Git工作流验证"

# 提交信息规范
$COMMIT_TYPES = @("feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert")
$COMMIT_PATTERN = "^($(($COMMIT_TYPES -join '|')))(\(.+\))?!?: .{1,100}$"
$COMMIT_DETAIL_PATTERN = "^\[.+\] .+"

function Test-CommitMessage {
    param([string]$Message)
    
    # 去除注释和空行
    $cleanMsg = ($Message -split "`n" | Where-Object { $_ -notmatch "^#" -and $_.Trim() -ne "" }) -join "`n"
    $firstLine = ($cleanMsg -split "`n")[0]
    
    Write-Step "检查提交信息格式..."
    
    # 检查是否匹配Conventional Commits规范
    if ($firstLine -match $COMMIT_PATTERN) {
        Write-Success "提交信息格式正确: $($matches[1])"
        return $true
    }
    
    # 检查是否匹配详细格式 [模块] 描述
    if ($firstLine -match $COMMIT_DETAIL_PATTERN) {
        Write-Success "提交信息格式正确 (详细格式)"
        return $true
    }
    
    # 检查是否是合并提交
    if ($firstLine -match "^Merge (branch|pull request|remote-tracking branch)") {
        Write-Success "合并提交，跳过格式检查"
        return $true
    }
    
    Write-Error "提交信息格式不符合规范"
    Write-Host ""
    Write-Host "${YELLOW}当前提交信息:${RESET}"
    Write-Host "  $firstLine"
    Write-Host ""
    Write-Host "${CYAN}支持的格式:${RESET}"
    Write-Host "  1. Conventional Commits:"
    Write-Host "     type(scope): description"
    Write-Host "     示例: feat(controller): 添加PID参数调节功能"
    Write-Host ""
    Write-Host "  2. 详细格式:"
    Write-Host "     [模块] 描述"
    Write-Host "     示例: [控制算法] 优化PID计算性能"
    Write-Host ""
    Write-Host "${CYAN}支持的类型:${RESET}"
    Write-Host "  $($COMMIT_TYPES -join ', ')"
    
    return $false
}

function Test-BranchProtection {
    Write-Step "检查分支保护规则..."
    
    $branch = git rev-parse --abbrev-ref HEAD 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Error "无法获取当前分支"
        return $false
    }
    
    Write-Info "当前分支: $branch"
    
    # 受保护分支
    $protectedBranches = @("main", "master", "release/*")
    
    foreach ($pattern in $protectedBranches) {
        if ($branch -like $pattern) {
            if ($PrePush) {
                Write-Error "不允许直接推送到受保护分支: $branch"
                Write-Host "${YELLOW}请使用Pull Request进行代码合并${RESET}"
                return $false
            } else {
                Write-Warning "当前在受保护分支上工作，建议创建功能分支"
            }
        }
    }
    
    # 检查分支命名规范
    $validPatterns = @(
        "^feature/",
        "^fix/",
        "^docs/",
        "^refactor/",
        "^test/",
        "^chore/",
        "^main$",
        "^master$",
        "^develop$",
        "^release/"
    )
    
    $isValid = $false
    foreach ($pattern in $validPatterns) {
        if ($branch -match $pattern) {
            $isValid = $true
            break
        }
    }
    
    if (-not $isValid) {
        Write-Warning "分支名 '$branch' 不符合命名规范"
        Write-Host "${CYAN}建议的分支命名:${RESET}"
        Write-Host "  feature/功能描述"
        Write-Host "  fix/修复描述"
        Write-Host "  docs/文档描述"
        Write-Host "  refactor/重构描述"
    } else {
        Write-Success "分支命名符合规范"
    }
    
    return $true
}

function Test-StagedFiles {
    Write-Step "检查暂存区文件..."
    
    $staged = git diff --cached --name-only 2>$null
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($staged)) {
        Write-Warning "暂存区为空"
        return $true
    }
    
    $files = $staged -split "`n" | Where-Object { $_ -ne "" }
    Write-Info "暂存区文件数: $($files.Count)"
    
    # 检查大文件（使用git ls-files获取实际大小）
    $largeFiles = @()
    foreach ($file in $files) {
        try {
            $output = git ls-files -s $file 2>$null
            if ($output) {
                $size = ($output -split "\s+")[3]
                if ($size -and [int]$size -gt 1048576) {  # 1MB = 1048576 bytes
                    $sizeMB = [math]::Round([int]$size / 1048576, 2)
                    $largeFiles += "$file (${sizeMB}MB)"
                }
            }
        } catch {
            # 如果无法获取大小，跳过检查
        }
    }
    
    if ($largeFiles.Count -gt 0) {
        Write-Error "发现大文件（建议不超过1MB）:"
        $largeFiles | ForEach-Object { Write-Host "  $_" }
        return $false
    }
    
    # 检查敏感文件
    $sensitivePatterns = @("*.pem", "*.key", "*.p12", "*.env", "secrets*", "password*", "credential*")
    $sensitiveFiles = @()
    foreach ($file in $files) {
        $fileName = Split-Path $file -Leaf
        foreach ($pattern in $sensitivePatterns) {
            if ($fileName -like $pattern) {
                $sensitiveFiles += $file
                break
            }
        }
    }
    
    if ($sensitiveFiles.Count -gt 0) {
        Write-Error "发现可能包含敏感信息的文件:"
        $sensitiveFiles | ForEach-Object { Write-Host "  $_" }
        Write-Host "${YELLOW}请确认这些文件不包含密码、密钥等敏感信息${RESET}"
        return $false
    }
    
    Write-Success "暂存区文件检查通过"
    return $true
}

function Test-RemoteSync {
    Write-Step "检查远程同步状态..."
    
    # 获取远程分支信息
    git fetch origin --quiet 2>$null
    
    $branch = git rev-parse --abbrev-ref HEAD
    $localCommit = git rev-parse HEAD
    $remoteCommit = git rev-parse "origin/$branch" 2>$null
    
    if ($LASTEXITCODE -ne 0) {
        Write-Warning "远程分支 origin/$branch 不存在"
        return $true
    }
    
    $baseCommit = git merge-base HEAD "origin/$branch" 2>$null
    
    if ($localCommit -eq $remoteCommit) {
        Write-Success "本地与远程同步"
    } elseif ($baseCommit -eq $localCommit) {
        Write-Error "本地分支落后于远程，请先拉取更新"
        return $false
    } elseif ($baseCommit -eq $remoteCommit) {
        Write-Info "本地分支领先于远程，可以推送"
    } else {
        Write-Error "本地与远程有分歧，需要合并"
        return $false
    }
    
    return $true
}

# 主逻辑
if ($CommitMsgFile -and (Test-Path $CommitMsgFile)) {
    # 提交信息验证模式
    $msg = Get-Content $CommitMsgFile -Raw
    if (-not (Test-CommitMessage $msg)) {
        $script:ExitCode = 1
    }
} else {
    # 完整验证模式
    if (-not (Test-BranchProtection)) {
        $script:ExitCode = 1
    }
    
    if (-not (Test-StagedFiles)) {
        $script:ExitCode = 1
    }
    
    if ($PrePush) {
        if (-not (Test-RemoteSync)) {
            $script:ExitCode = 1
        }
    }
}

# 总结
Write-Header "验证总结"
if ($script:ExitCode -eq 0) {
    Write-Success "Git工作流验证通过！"
} else {
    Write-Error "Git工作流验证未通过"
}

exit $script:ExitCode
