<#
.SYNOPSIS
智能语义化版本递增与 Changelog 生成脚本 (Smart Bump & Changelog Generator)

.DESCRIPTION
该脚本抓取上一个 Tag 以来的所有 Git 提交记录，基于 Conventional Commits 协议：
- feat: -> Minor (小版本更新)
- fix:, refactor:, perf: -> Patch (补丁更新)
- BREAKING CHANGE: 或 feat!: -> Major (大版本更新)
自动更新本工作区中所有 Cargo.toml 的版本号，并根据提交生成自然语言式的 CHANGELOG.md 内容。

.EXAMPLE
.\scripts\smart-bump.ps1 -Project robot_control_rust
#>

param(
    [Parameter(Mandatory=$false)]
    [string]$Project = "All", # 可选: robot_control_rust, rust_micro_tools, rust_indie_tools, 或 All
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

function Get-Projects {
    $projects = @()
    if ($Project -eq "All" -or $Project -eq "robot_control_rust") {
        if (Test-Path ".\robot_control_rust\Cargo.toml") { $projects += "robot_control_rust" }
    }
    if ($Project -eq "All" -or $Project -eq "rust_micro_tools") {
        if (Test-Path ".\rust_micro_tools\Cargo.toml") { $projects += "rust_micro_tools" }
    }
    if ($Project -eq "All" -or $Project -eq "rust_indie_tools") {
        $indies = Get-ChildItem -Path ".\rust_indie_tools" -Directory
        foreach ($indie in $indies) {
            if (Test-Path "$($indie.FullName)\Cargo.toml") {
                $projects += "rust_indie_tools\$($indie.Name)"
            }
        }
    }
    return $projects
}

function Get-NextVersion {
    param([string]$CurrentVersion, [string]$BumpType)
    $parts = $CurrentVersion.Split('.')
    $major = [int]$parts[0]
    $minor = [int]$parts[1]
    $patch = [int]$parts[2]

    if ($BumpType -eq "major") {
        $major++
        $minor = 0
        $patch = 0
    } elseif ($BumpType -eq "minor") {
        $minor++
        $patch = 0
    } else {
        $patch++
    }
    return "$major.$minor.$patch"
}

$ProjectsToBump = Get-Projects

foreach ($proj in $ProjectsToBump) {
    Write-Host "Analyzing Project: $proj" -ForegroundColor Cyan
    $tomlPath = ".\$proj\Cargo.toml"
    $tomlContent = Get-Content $tomlPath -Raw
    
    # 匹配 version = "x.y.z"
    if ($tomlContent -match 'version\s*=\s*"(\d+\.\d+\.\d+)"') {
        $currentVersion = $Matches[1]
    } else {
        Write-Host "No valid version found in $tomlPath" -ForegroundColor Yellow
        continue
    }

    # 获取最后一次 tag 
    # 这里为了简单化，假设统一使用 <proj>-vX.Y.Z 的 tag 格式，如果是全局的，简化为全局 tag
    $lastTag = git describe --tags --abbrev=0 --match "$proj-v*" 2>$null
    if (-not $lastTag) {
        $lastTag = git describe --tags --abbrev=0 2>$null
    }

    $commitMsgs = @()
    if ($lastTag) {
        Write-Host "Found last tag: $lastTag. Extracting commits..." -ForegroundColor DarkGray
        $commits = git log "$lastTag..HEAD" --oneline -- "$proj"
        if ($commits) { $commitMsgs = $commits -split "`n" }
    } else {
        Write-Host "No previous tag found. Using all commits." -ForegroundColor DarkGray
        $commits = git log --oneline -- "$proj"
        if ($commits) { $commitMsgs = $commits -split "`n" }
    }

    if ($commitMsgs.Count -eq 0) {
        Write-Host "No new commits for $proj since last tag. Skipping." -ForegroundColor Yellow
        continue
    }

    $bumpType = "patch"
    $features = @()
    $fixes = @()
    $breaking = @()

    foreach ($msg in $commitMsgs) {
        if ($msg -match "BREAKING CHANGE" -or $msg -match "^[a-z]+\!:") {
            $bumpType = "major"
            $breaking += $msg
        } elseif ($msg -match "^feat(\(.*\))?:") {
            if ($bumpType -ne "major") { $bumpType = "minor" }
            $features += $msg
        } elseif ($msg -match "^(?:fix|refactor|perf)(\(.*\))?:") {
            $fixes += $msg
        }
    }

    $nextVersion = Get-NextVersion -CurrentVersion $currentVersion -BumpType $bumpType
    Write-Host "Version Bump: $currentVersion -> $nextVersion ($bumpType)" -ForegroundColor Green

    # 构建 Changelog
    $dateStr = Get-Date -Format "yyyy-MM-dd"
    $changelogEntry = "`n## [$nextVersion] - $dateStr`n"
    if ($breaking.Count -gt 0) {
        $changelogEntry += "`n### ⚠️ BREAKING CHANGES`n"
        $changelogEntry += ($breaking | ForEach-Object { "- $_" }) -join "`n"
        $changelogEntry += "`n"
    }
    if ($features.Count -gt 0) {
        $changelogEntry += "`n### ✨ Features`n"
        $changelogEntry += ($features | ForEach-Object { "- $_" }) -join "`n"
        $changelogEntry += "`n"
    }
    if ($fixes.Count -gt 0) {
        $changelogEntry += "`n### 🐛 Bug Fixes & Refactoring`n"
        $changelogEntry += ($fixes | ForEach-Object { "- $_" }) -join "`n"
        $changelogEntry += "`n"
    }

    if (-not $DryRun) {
        # 修改 Cargo.toml
        $newTomlContent = $tomlContent -replace 'version\s*=\s*"\d+\.\d+\.\d+"', "version = `"$nextVersion`""
        Set-Content -Path $tomlPath -Value $newTomlContent -Encoding UTF8

        # 写入 CHANGELOG
        $changelogPath = ".\$proj\CHANGELOG.md"
        if (Test-Path $changelogPath) {
            $oldLog = Get-Content $changelogPath -Raw
            # 简单插入到第一行标题后面
            if ($oldLog -match "(?s)(# Changelog.*?`n)(.*)") {
                $newLog = $Matches[1] + $changelogEntry + "`n" + $Matches[2]
                Set-Content -Path $changelogPath -Value $newLog -Encoding UTF8
            } else {
                Set-Content -Path $changelogPath -Value "# Changelog`n$changelogEntry`n$oldLog" -Encoding UTF8
            }
        } else {
            Set-Content -Path $changelogPath -Value "# Changelog`n$changelogEntry" -Encoding UTF8
        }
        
        Write-Host "Updated Cargo.toml and CHANGELOG.md for $proj" -ForegroundColor Green
    }
}
