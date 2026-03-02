#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    智能PR工具 - 自动化Pull Request流程
.DESCRIPTION
    创建PR、生成PR描述、检查PR准备情况、自动合并检测
#>

param(
    [switch]$Create,
    [switch]$Check,
    [switch]$Merge,
    [switch]$Draft,
    [string]$Title = "",
    [string]$Body = "",
    [string]$Base = "main",
    [string]$Head = "",
    [switch]$AutoFill,
    [switch]$Help
)

if ($Help) {
    @"
智能PR工具

用法:
  .\pr-helper.ps1 [选项]

选项:
  -Create          创建新的PR
  -Check           检查PR准备情况
  -Merge           尝试合并PR
  -Draft           创建为草稿PR
  -Title "标题"    PR标题
  -Body "内容"     PR描述
  -Base "分支"     目标分支 (默认: main)
  -Head "分支"     源分支 (默认: 当前分支)
  -AutoFill        自动生成PR描述
  -Help            显示帮助

示例:
  .\pr-helper.ps1 -Check                    # 检查PR准备情况
  .\pr-helper.ps1 -Create -Title "修复Bug"   # 创建PR
  .\pr-helper.ps1 -Create -AutoFill          # 自动生成描述创建PR
  .\pr-helper.ps1 -Merge                     # 合并当前PR
"@ | Write-Host
    exit 0
}

$ErrorActionPreference = "Stop"
$script:ExitCode = 0

# 导入通用模块
$ScriptDir = $PSScriptRoot
Import-Module "$ScriptDir\common.psm1" -Force

Write-Header "🔀 智能PR工具"

# 获取当前分支
$currentBranch = git rev-parse --abbrev-ref HEAD 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Error "无法获取当前分支"
    exit 1
}

if ([string]::IsNullOrEmpty($Head)) {
    $Head = $currentBranch
}

Write-Info "当前分支: $Head"
Write-Info "目标分支: $Base"

# 功能1: 检查PR准备情况
function Test-PRReadiness {
    Write-Header "PR准备情况检查"
    
    $issues = @()
    $warnings = @()
    
    # 1. 检查是否在目标分支上
    if ($Head -eq $Base) {
        $issues += "不能在目标分支 '$Base' 上创建PR"
    }
    
    # 2. 检查分支是否已推送到远程
    Write-Step "检查远程分支..."
    $remoteBranch = git ls-remote --heads origin $Head 2>$null
    if ([string]::IsNullOrEmpty($remoteBranch)) {
        $issues += "分支 '$Head' 未推送到远程，请先执行: git push -u origin $Head"
    } else {
        Write-Success "分支已推送到远程"
    }
    
    # 3. 检查是否有未提交的更改
    Write-Step "检查工作区..."
    $status = git status --porcelain
    if ($status) {
        $warnings += "工作区有未提交的更改"
        Write-Warning "工作区不干净"
    } else {
        Write-Success "工作区干净"
    }
    
    # 4. 检查与目标分支的差异
    Write-Step "检查分支差异..."
    git fetch origin $Base --quiet 2>$null
    $diff = git diff --stat origin/$Base...$Head 2>$null
    if ([string]::IsNullOrEmpty($diff)) {
        $issues += "与目标分支没有差异，无需创建PR"
    } else {
        $commitCount = (git rev-list --count origin/$Base...$Head 2>$null)
        $fileCount = ($diff -split "`n" | Where-Object { $_ -match "^\s*\d+" }).Count
        Write-Info "提交数: $commitCount"
        Write-Info "修改文件数: $fileCount"
        Write-Success "有代码变更需要合并"
    }
    
    # 5. 检查是否有冲突
    Write-Step "检查合并冲突..."
    $mergeBase = git merge-base HEAD origin/$Base 2>$null
    $canMerge = git merge-tree $mergeBase HEAD origin/$Base 2>$null | Where-Object { $_ -match "<<<<<<" }
    if ($canMerge) {
        $issues += "存在合并冲突，请先解决"
        Write-Error "检测到合并冲突"
    } else {
        Write-Success "无合并冲突"
    }
    
    # 6. 运行审查脚本
    Write-Step "运行代码审查..."
    & "$ScriptDir\review.ps1" -Quick
    if ($LASTEXITCODE -ne 0) {
        $issues += "代码审查未通过"
    }
    
    # 7. 检查最近的提交信息
    Write-Step "检查提交历史..."
    $commits = git log --oneline origin/$Base..$Head 2>$null
    if ($commits) {
        Write-Info "最近的提交:"
        $commits -split "`n" | Select-Object -First 5 | ForEach-Object { Write-Host "  $_" }
    }
    
    # 总结
    Write-Header "检查结果"
    
    if ($issues.Count -eq 0 -and $warnings.Count -eq 0) {
        Write-Success "✓ PR准备就绪！"
        return $true
    } else {
        if ($issues.Count -gt 0) {
            Write-Error "发现以下问题，需要修复:"
            $issues | ForEach-Object { Write-Host "  ✗ $_" }
        }
        if ($warnings.Count -gt 0) {
            Write-Warning "警告:"
            $warnings | ForEach-Object { Write-Host "  ⚠ $_" }
        }
        return $false
    }
}

# 功能2: 自动生成PR描述
function Get-AutoPRDescription {
    Write-Step "自动生成PR描述..."
    
    # 获取提交信息
    $commits = git log --pretty=format:"- %s" origin/$Base..$Head 2>$null
    
    # 获取变更文件
    $files = git diff --name-only origin/$Base...$Head 2>$null
    $fileList = $files -split "`n" | Where-Object { $_ -ne "" }
    
    # 分类文件
    $categories = @{
        "新功能" = @()
        "修复" = @()
        "文档" = @()
        "其他" = @()
    }
    
    foreach ($file in $fileList) {
        if ($file -match "^docs/|\.md$") {
            $categories["文档"] += $file
        } elseif ($commits -match "feat|add|new" -and $file -notmatch "test") {
            $categories["新功能"] += $file
        } elseif ($commits -match "fix|bug|repair") {
            $categories["修复"] += $file
        } else {
            $categories["其他"] += $file
        }
    }
    
    # 生成描述
    $description = @"
## 变更摘要

### 提交历史
$commits

### 变更文件
"@
    
    foreach ($category in $categories.Keys) {
        if ($categories[$category].Count -gt 0) {
            $description += "`n#### $category`n"
            $categories[$category] | ForEach-Object { $description += "- $_`n" }
        }
    }
    
    $description += @"

### 检查清单
- [ ] 代码审查通过
- [ ] 测试通过
- [ ] 文档已更新
- [ ] 无合并冲突

### 相关Issue
<!-- 关联的Issue编号，如: Fixes #123 -->
"@
    
    return $description
}

# 功能3: 创建PR
function New-PullRequest {
    param([string]$PRTitle, [string]$PRBody)
    
    Write-Header "创建Pull Request"
    
    # 先检查准备情况
    if (-not (Test-PRReadiness)) {
        Write-Error "PR准备检查未通过，无法创建PR"
        return $false
    }
    
    # 如果没有标题，生成默认标题
    if ([string]::IsNullOrEmpty($PRTitle)) {
        $firstCommit = (git log --pretty=format:"%s" -1 origin/$Base..$Head 2>$null)
        $PRTitle = $firstCommit
        Write-Info "使用默认标题: $PRTitle"
    }
    
    # 如果没有描述且要求自动填充
    if ([string]::IsNullOrEmpty($PRBody) -and $AutoFill) {
        $PRBody = Get-AutoPRDescription
    }
    
    # 检查是否有gh CLI
    $ghCheck = Invoke-CommandWithOutput "where.exe" "gh"
    if ($ghCheck.ExitCode -eq 0) {
        Write-Step "使用GitHub CLI创建PR..."
        
        $ghArgs = "pr create --base `"$Base`" --head `"$Head`" --title `"$PRTitle`""
        
        if ($Draft) {
            $ghArgs += " --draft"
        }
        
        if (-not [string]::IsNullOrEmpty($PRBody)) {
            # 将描述写入临时文件
            $tempFile = [System.IO.Path]::GetTempFileName()
            $PRBody | Out-File -FilePath $tempFile -Encoding UTF8
            $ghArgs += " --body-file `"$tempFile`""
        }
        
        $result = Invoke-CommandWithOutput "gh" $ghArgs
        
        if ($result.ExitCode -eq 0) {
            Write-Success "PR创建成功！"
            Write-Host $result.StdOut
            return $true
        } else {
            Write-Error "PR创建失败"
            Write-Host $result.StdErr
            return $false
        }
    } else {
        # 没有gh CLI，输出手动创建指南
        Write-Warning "未找到GitHub CLI (gh)，请手动创建PR"
        Write-Host ""
        Write-Host "${CYAN}手动创建PR步骤:${RESET}"
        Write-Host "1. 打开仓库页面"
        Write-Host "2. 点击 'New Pull Request'"
        Write-Host "3. 选择 base: $Base <- compare: $Head"
        Write-Host ""
        Write-Host "${CYAN}建议的PR标题:${RESET}"
        Write-Host $PRTitle
        Write-Host ""
        Write-Host "${CYAN}PR描述模板:${RESET}"
        if ([string]::IsNullOrEmpty($PRBody)) {
            $PRBody = Get-AutoPRDescription
        }
        Write-Host $PRBody
        return $false
    }
}

# 功能4: 合并PR
function Merge-PullRequest {
    Write-Header "合并Pull Request"
    
    # 检查当前分支是否有PR
    $ghCheck = Invoke-CommandWithOutput "where.exe" "gh"
    if ($ghCheck.ExitCode -eq 0) {
        Write-Step "检查PR状态..."
        $prInfo = Invoke-CommandWithOutput "gh" "pr view --json state,mergeStateStatus,title"
        
        if ($prInfo.ExitCode -eq 0) {
            Write-Info "找到PR: $($prInfo.StdOut)"
            
            # 检查是否可以合并
            $result = Invoke-CommandWithOutput "gh" "pr merge --squash --delete-branch"
            if ($result.ExitCode -eq 0) {
                Write-Success "PR合并成功！"
                Write-Host $result.StdOut
                return $true
            } else {
                Write-Error "PR合并失败"
                Write-Host $result.StdErr
                return $false
            }
        } else {
            Write-Error "当前分支没有关联的PR"
            return $false
        }
    } else {
        Write-Warning "未找到GitHub CLI，请手动合并PR"
        return $false
    }
}

# 主逻辑
if ($Check) {
    Test-PRReadiness
    exit $script:ExitCode
}
elseif ($Create) {
    $success = New-PullRequest -PRTitle $Title -PRBody $Body
    exit ($success ? 0 : 1)
}
elseif ($Merge) {
    $success = Merge-PullRequest
    exit ($success ? 0 : 1)
}
else {
    # 默认执行检查
    Test-PRReadiness
    exit $script:ExitCode
}
