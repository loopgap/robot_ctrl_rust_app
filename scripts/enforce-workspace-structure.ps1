#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Enforce workspace layout and reject misplaced files.
.DESCRIPTION
    Validates root-level structure and blocks staged process artifacts.
#>

param(
    [ValidateSet("audit", "apply")]
    [string]$Mode = "audit",

    [string]$RepoRoot = "",
    [string]$ConfigPath = "",
    [switch]$Strict,
    [switch]$UseStagedPaths
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = Split-Path -Parent $PSScriptRoot
}

if ([string]::IsNullOrWhiteSpace($ConfigPath)) {
    $ConfigPath = Join-Path $PSScriptRoot "workspace-governance.json"
}

Set-Location $RepoRoot

if (-not (Test-Path $ConfigPath)) {
    throw "Missing workspace governance config: $ConfigPath"
}

$config = Get-Content -Path $ConfigPath -Raw -Encoding UTF8 | ConvertFrom-Json -Depth 10

function Get-StringArray($value) {
    if ($null -eq $value) {
        return @()
    }
    if ($value -is [System.Array]) {
        return @($value | ForEach-Object { [string]$_ })
    }
    return @([string]$value)
}

$allowedRootEntries = @(Get-StringArray -value $config.workspace.allowedRootEntries)
$allowedRootRegex = @(Get-StringArray -value $config.workspace.allowedRootRegex)
$blockedPathRegex = @(Get-StringArray -value $config.workspace.blockedPathRegex)
$blockedFixedRelativePaths = @(Get-StringArray -value $config.workspace.blockedFixedRelativePaths)
$blockedGlobPatterns = @(Get-StringArray -value $config.workspace.blockedGlobPatterns)

if ($allowedRootEntries.Count -eq 0 -or $blockedPathRegex.Count -eq 0) {
    throw "workspace-governance workspace policy is incomplete"
}

function Normalize-Path([string]$path) {
    return ($path -replace '\\', '/').Trim()
}

function Is-AllowedRoot([string]$entry) {
    if ($allowedRootEntries -contains $entry) {
        return $true
    }
    foreach ($rx in $allowedRootRegex) {
        if ($entry -match $rx) {
            return $true
        }
    }
    return $false
}

function Is-BlockedPath([string]$relativePath) {
    foreach ($rx in $blockedPathRegex) {
        if ($relativePath -match $rx) {
            return $true
        }
    }
    return $false
}

function Get-BlockedWorkspacePaths([string]$RootPath) {
    $paths = New-Object System.Collections.Generic.List[string]

    foreach ($rel in $blockedFixedRelativePaths) {
        $full = Join-Path $RootPath $rel
        if (Test-Path $full) {
            $paths.Add((Normalize-Path -path $rel))
        }
    }

    foreach ($pattern in $blockedGlobPatterns) {
        $globPath = Join-Path $RootPath $pattern
        $items = Get-ChildItem -Path $globPath -Force -ErrorAction SilentlyContinue
        foreach ($item in $items) {
            $relative = $item.FullName.Substring($RootPath.Length).TrimStart('\\', '/')
            $paths.Add((Normalize-Path -path $relative))
        }
    }

    return @($paths | Sort-Object -Unique)
}

$unexpectedRootEntries = New-Object System.Collections.Generic.List[string]
$blockedWorkspacePaths = New-Object System.Collections.Generic.List[string]
$blockedStagedPaths = New-Object System.Collections.Generic.List[string]

$rootEntries = Get-ChildItem -Path $RepoRoot -Force -ErrorAction Stop | ForEach-Object { $_.Name }
foreach ($entry in $rootEntries) {
    if (-not (Is-AllowedRoot -entry $entry)) {
        $unexpectedRootEntries.Add($entry)
    }
}

$blockedWorkspacePaths = @((Get-BlockedWorkspacePaths -RootPath $RepoRoot))

if ($UseStagedPaths) {
    $staged = git diff --cached --name-only
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to read staged paths"
    }

    foreach ($path in @($staged | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })) {
        $rel = Normalize-Path -path $path
        if (Is-BlockedPath -relativePath $rel) {
            $blockedStagedPaths.Add($rel)
        }
    }
}

$unexpectedRootEntries = @($unexpectedRootEntries | Sort-Object -Unique)
$blockedWorkspacePaths = @($blockedWorkspacePaths | Sort-Object -Unique)
$blockedStagedPaths = @($blockedStagedPaths | Sort-Object -Unique)

Write-Host "Workspace structure summary" -ForegroundColor Cyan
Write-Host "- Mode: $Mode"
Write-Host "- Unexpected root entries: $($unexpectedRootEntries.Count)"
Write-Host "- Blocked workspace paths: $($blockedWorkspacePaths.Count)"
Write-Host "- Blocked staged paths: $($blockedStagedPaths.Count)"

if ($unexpectedRootEntries.Count -gt 0) {
    Write-Host "Unexpected root entries:" -ForegroundColor Yellow
    $unexpectedRootEntries | ForEach-Object { Write-Host "  $_" -ForegroundColor Yellow }
}

if ($blockedWorkspacePaths.Count -gt 0) {
    Write-Host "Blocked workspace paths:" -ForegroundColor Yellow
    $blockedWorkspacePaths | ForEach-Object { Write-Host "  $_" -ForegroundColor Yellow }
}

if ($blockedStagedPaths.Count -gt 0) {
    Write-Host "Blocked staged paths:" -ForegroundColor Yellow
    $blockedStagedPaths | ForEach-Object { Write-Host "  $_" -ForegroundColor Yellow }
}

if ($Mode -eq "apply") {
    $cleanupScript = Join-Path $PSScriptRoot "cleanup-process-files.ps1"
    if (-not (Test-Path $cleanupScript)) {
        throw "Missing cleanup script: $cleanupScript"
    }

    & $cleanupScript -Mode apply -RepoRoot $RepoRoot -ConfigPath $ConfigPath
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to clean process files"
    }

    # Re-evaluate blocked workspace paths after cleanup.
    $blockedWorkspacePaths = @((Get-BlockedWorkspacePaths -RootPath $RepoRoot))
}

$issueCount = $unexpectedRootEntries.Count + $blockedWorkspacePaths.Count + $blockedStagedPaths.Count
if ($Strict -and $issueCount -gt 0) {
    exit 2
}

Write-Host "enforce-workspace-structure $Mode completed." -ForegroundColor Green
