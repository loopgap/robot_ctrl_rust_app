#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Seal workspace workflow with cleanup, guard and release-state checks.
.DESCRIPTION
    Provides one entrypoint to keep workspace layout stable and release
    metadata consistent. Audit mode is non-destructive. Apply mode can
    normalize local release state when switches are provided.
#>

param(
    [ValidateSet("audit", "apply")]
    [string]$Mode = "audit",

    [string]$RepoRoot = "",
    [switch]$PruneLocalTagsNotOnRemote,
    [switch]$CleanOrphanNotes,
    [switch]$SkipRemote
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = Split-Path -Parent $PSScriptRoot
}

Set-Location $RepoRoot

$cleanupScript = Join-Path $PSScriptRoot "cleanup-process-files.ps1"
$guardScript = Join-Path $PSScriptRoot "enforce-workspace-structure.ps1"
$syncScript = Join-Path $PSScriptRoot "sync-release-state.ps1"
$indexScript = Join-Path $PSScriptRoot "update-release-index.ps1"

foreach ($script in @($cleanupScript, $guardScript, $syncScript, $indexScript)) {
    if (-not (Test-Path $script)) {
        throw "Missing required script: $script"
    }
}

$pwsh = Get-Command pwsh -ErrorAction SilentlyContinue
if (-not $pwsh) {
    throw "pwsh (PowerShell 7+) is required"
}

function Invoke-ChildScript {
    param(
        [Parameter(Mandatory = $true)][string]$ScriptPath,
        [string[]]$Arguments = @()
    )

    & $pwsh.Source -NoProfile -File $ScriptPath @Arguments
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

Write-Host "Workflow seal started. Mode: $Mode" -ForegroundColor Cyan

if ($Mode -eq "audit") {
    Invoke-ChildScript -ScriptPath $cleanupScript -Arguments @("-Mode", "audit", "-RepoRoot", $RepoRoot, "-Strict")

    Invoke-ChildScript -ScriptPath $guardScript -Arguments @("-Mode", "audit", "-RepoRoot", $RepoRoot, "-Strict")

    $syncArgs = @("-Mode", "audit", "-RepoRoot", $RepoRoot, "-Strict")
    if ($SkipRemote) { $syncArgs += "-SkipRemote" }
    Invoke-ChildScript -ScriptPath $syncScript -Arguments $syncArgs

    Invoke-ChildScript -ScriptPath $indexScript -Arguments @("-RepoRoot", $RepoRoot)

    Write-Host "Workflow seal audit completed." -ForegroundColor Green
    exit 0
}

Invoke-ChildScript -ScriptPath $cleanupScript -Arguments @("-Mode", "apply", "-RepoRoot", $RepoRoot)

Invoke-ChildScript -ScriptPath $guardScript -Arguments @("-Mode", "apply", "-RepoRoot", $RepoRoot, "-Strict")

$syncApplyArgs = @("-Mode", "apply", "-RepoRoot", $RepoRoot)
if ($PruneLocalTagsNotOnRemote) { $syncApplyArgs += "-PruneLocalTagsNotOnRemote" }
if ($CleanOrphanNotes) { $syncApplyArgs += "-CleanOrphanNotes" }
if ($SkipRemote) { $syncApplyArgs += "-SkipRemote" }
Invoke-ChildScript -ScriptPath $syncScript -Arguments $syncApplyArgs

Invoke-ChildScript -ScriptPath $cleanupScript -Arguments @("-Mode", "apply", "-RepoRoot", $RepoRoot)

Invoke-ChildScript -ScriptPath $guardScript -Arguments @("-Mode", "audit", "-RepoRoot", $RepoRoot, "-Strict")

Invoke-ChildScript -ScriptPath $indexScript -Arguments @("-RepoRoot", $RepoRoot)

Write-Host "Workflow seal apply completed." -ForegroundColor Green
exit 0
