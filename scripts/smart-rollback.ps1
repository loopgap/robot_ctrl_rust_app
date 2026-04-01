#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Roll back a failed release tag workflow.
.DESCRIPTION
    Supports deleting release by tag, deleting local/remote tag, and optional
    revert of the latest bump commit.
#>

param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^v\d+\.\d+\.\d+([-.].+)?$')]
    [string]$Tag,

    [string]$Owner = "loopgap",
    [string]$Repo = "robot_ctrl_rust_app",

    [switch]$DeleteRelease,
    [switch]$DeleteRemoteTag,
    [switch]$DeleteLocalTag,
    [switch]$RevertLastCommit,
    [switch]$PushRevert,
    [switch]$NoVerify,
    [switch]$SkipProcessCleanup,
    [switch]$SkipWorkspaceGuard,
    [switch]$SkipIndexRefresh
)

$ErrorActionPreference = "Stop"

function Invoke-Git {
    param(
        [Parameter(Mandatory = $true)][string]$Command,
        [string]$ErrorMessage = "Git command failed"
    )

    Invoke-Expression $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$ErrorMessage (exit=$LASTEXITCODE): $Command"
    }
}

function Remove-GitHubReleaseByTag {
    param(
        [Parameter(Mandatory = $true)][string]$Owner,
        [Parameter(Mandatory = $true)][string]$Repo,
        [Parameter(Mandatory = $true)][string]$Tag
    )

    if (-not $env:GITHUB_TOKEN) {
        throw "DeleteRelease requires GITHUB_TOKEN"
    }

    $headers = @{
        Authorization = "Bearer $($env:GITHUB_TOKEN)"
        Accept = "application/vnd.github+json"
        "X-GitHub-Api-Version" = "2022-11-28"
    }

    $uri = "https://api.github.com/repos/$Owner/$Repo/releases/tags/$Tag"
    try {
        $release = Invoke-RestMethod -Method Get -Uri $uri -Headers $headers
    } catch {
        Write-Host "Release for tag $Tag not found. Skip delete." -ForegroundColor Yellow
        return
    }

    Invoke-RestMethod -Method Delete -Uri "https://api.github.com/repos/$Owner/$Repo/releases/$($release.id)" -Headers $headers
    Write-Host "Deleted GitHub release for $Tag" -ForegroundColor Green
}

function Invoke-ProcessCleanup {
    param(
        [Parameter(Mandatory = $true)][string]$ScriptsDir,
        [Parameter(Mandatory = $true)][string]$RepoRoot,
        [switch]$Skip
    )

    if ($Skip) {
        return
    }

    $cleanupScript = Join-Path $ScriptsDir "cleanup-process-files.ps1"
    if (-not (Test-Path $cleanupScript)) {
        throw "Missing process cleanup script: $cleanupScript"
    }

    Invoke-ChildPwshScript -ScriptPath $cleanupScript -Arguments @("-Mode", "apply", "-RepoRoot", $RepoRoot) -ErrorMessage "Process cleanup failed"
}

function Invoke-WorkspaceGuard {
    param(
        [Parameter(Mandatory = $true)][string]$ScriptsDir,
        [Parameter(Mandatory = $true)][string]$RepoRoot,
        [switch]$Skip
    )

    if ($Skip) {
        return
    }

    $guardScript = Join-Path $ScriptsDir "enforce-workspace-structure.ps1"
    if (-not (Test-Path $guardScript)) {
        throw "Missing workspace guard script: $guardScript"
    }

    Invoke-ChildPwshScript -ScriptPath $guardScript -Arguments @("-Mode", "audit", "-RepoRoot", $RepoRoot, "-Strict") -ErrorMessage "Workspace structure guard failed"
}

function Update-ReleaseIndex {
    param(
        [Parameter(Mandatory = $true)][string]$ScriptsDir,
        [Parameter(Mandatory = $true)][string]$RepoRoot,
        [switch]$Skip
    )

    if ($Skip) {
        return
    }

    $indexScript = Join-Path $ScriptsDir "update-release-index.ps1"
    if (-not (Test-Path $indexScript)) {
        throw "Missing release index script: $indexScript"
    }

    Invoke-ChildPwshScript -ScriptPath $indexScript -Arguments @("-RepoRoot", $RepoRoot) -ErrorMessage "Failed to update release index"
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$scriptsDir = $PSScriptRoot
Set-Location $repoRoot

$pwsh = Get-Command pwsh -ErrorAction SilentlyContinue
if (-not $pwsh) {
    throw "pwsh (PowerShell 7+) is required"
}

function Invoke-ChildPwshScript {
    param(
        [Parameter(Mandatory = $true)][string]$ScriptPath,
        [string[]]$Arguments = @(),
        [Parameter(Mandatory = $true)][string]$ErrorMessage
    )

    & $pwsh.Source -NoProfile -File $ScriptPath @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw $ErrorMessage
    }
}

$verifyFlag = if ($NoVerify) { " --no-verify" } else { "" }

try {
    Invoke-ProcessCleanup -ScriptsDir $scriptsDir -RepoRoot $repoRoot -Skip:$SkipProcessCleanup
    Invoke-WorkspaceGuard -ScriptsDir $scriptsDir -RepoRoot $repoRoot -Skip:$SkipWorkspaceGuard

    if ($DeleteRelease) {
        Remove-GitHubReleaseByTag -Owner $Owner -Repo $Repo -Tag $Tag
    }

    if ($DeleteRemoteTag) {
        Invoke-Git -Command "git push$verifyFlag origin :refs/tags/$Tag" -ErrorMessage "Failed to delete remote tag"
        Write-Host "Deleted remote tag: $Tag" -ForegroundColor Green
    }

    if ($DeleteLocalTag) {
        Invoke-Git -Command "git tag -d $Tag" -ErrorMessage "Failed to delete local tag"
        Write-Host "Deleted local tag: $Tag" -ForegroundColor Green
    }

    if ($RevertLastCommit) {
        $msg = (git log -1 --pretty=%s).Trim()
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to inspect latest commit"
        }

        if ($msg -notmatch '^chore\(release\): bump version to ') {
            throw "Latest commit is not a release bump commit: $msg"
        }

        Invoke-Git -Command "git revert --no-edit HEAD" -ErrorMessage "Failed to revert bump commit"
        Write-Host "Reverted last release bump commit" -ForegroundColor Green

        if ($PushRevert) {
            Invoke-Git -Command "git push$verifyFlag origin HEAD" -ErrorMessage "Failed to push revert commit"
            Write-Host "Pushed revert commit to origin" -ForegroundColor Green
        }
    }

    Update-ReleaseIndex -ScriptsDir $scriptsDir -RepoRoot $repoRoot -Skip:$SkipIndexRefresh

    Write-Host "Rollback operation completed." -ForegroundColor Green
}
finally {
    Invoke-ProcessCleanup -ScriptsDir $scriptsDir -RepoRoot $repoRoot -Skip:$SkipProcessCleanup
    Invoke-WorkspaceGuard -ScriptsDir $scriptsDir -RepoRoot $repoRoot -Skip:$SkipWorkspaceGuard
}
