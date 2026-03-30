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
    [switch]$NoVerify
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

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$verifyFlag = if ($NoVerify) { " --no-verify" } else { "" }

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

Write-Host "Rollback operation completed." -ForegroundColor Green
