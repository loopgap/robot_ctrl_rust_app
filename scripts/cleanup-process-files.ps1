#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Audit or clean known transient process files/directories.
.DESCRIPTION
    Removes local runtime artifacts that should not persist in the workspace,
    such as smoke logs and temporary release staging directories.
#>

param(
    [ValidateSet("audit", "apply")]
    [string]$Mode = "apply",

    [string]$RepoRoot = "",
    [string]$ConfigPath = "",
    [switch]$Strict
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

$fixedRelativePaths = @(
    Get-StringArray -value $config.cleanup.fixedRelativePaths
)

$globPatterns = @(
    Get-StringArray -value $config.cleanup.globPatterns
)

$protectedRelativePaths = @(
    Get-StringArray -value $config.cleanup.protectedRelativePaths
)

if ($fixedRelativePaths.Count -eq 0) {
    throw "workspace-governance cleanup.fixedRelativePaths cannot be empty"
}

$protectedFullPaths = @($protectedRelativePaths | ForEach-Object {
    [IO.Path]::GetFullPath((Join-Path $RepoRoot $_))
})

function Is-ProtectedPath([string]$fullPath) {
    foreach ($protected in $protectedFullPaths) {
        if ($fullPath.StartsWith($protected, [System.StringComparison]::OrdinalIgnoreCase)) {
            return $true
        }
    }
    return $false
}

function Get-Candidates {
    $candidates = New-Object System.Collections.Generic.List[string]

    foreach ($rel in $fixedRelativePaths) {
        $full = Join-Path $RepoRoot $rel
        if (Test-Path $full) {
            $resolved = [IO.Path]::GetFullPath((Resolve-Path -LiteralPath $full).Path)
            $candidates.Add($resolved)
        }
    }

    foreach ($pattern in $globPatterns) {
        $globPath = Join-Path $RepoRoot $pattern
        $items = Get-ChildItem -Path $globPath -Force -ErrorAction SilentlyContinue
        foreach ($item in $items) {
            $resolved = [IO.Path]::GetFullPath((Resolve-Path -LiteralPath $item.FullName).Path)
            $candidates.Add($resolved)
        }
    }

    return @($candidates | Sort-Object -Unique)
}

$found = @(Get-Candidates)

Write-Host "Process file cleanup summary" -ForegroundColor Cyan
Write-Host "- Mode: $Mode"
Write-Host "- Candidates found: $($found.Count)"

if ($found.Count -gt 0) {
    foreach ($path in $found) {
        Write-Host "  $path" -ForegroundColor Yellow
    }
}

if ($Mode -eq "apply") {
    foreach ($path in $found) {
        if ($path -eq [IO.Path]::GetFullPath($RepoRoot)) {
            throw "Refuse to delete repo root: $path"
        }
        if (Is-ProtectedPath -fullPath $path) {
            throw "Refuse to delete protected path: $path"
        }
        Remove-Item -Path $path -Recurse -Force -ErrorAction Stop
        Write-Host "Removed: $path" -ForegroundColor Green
    }
}

$remaining = if ($Mode -eq "apply") { @(Get-Candidates) } else { $found }

if ($Strict -and $remaining.Count -gt 0) {
    exit 2
}

Write-Host "cleanup-process-files $Mode completed." -ForegroundColor Green
