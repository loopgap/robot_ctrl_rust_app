#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Audit or normalize local release metadata state.
.DESCRIPTION
    Checks consistency among git tags, release notes files, and local archive
    layout. In apply mode it can prune local tags not found on remote and
    remove orphan release notes, then rebuild release index.
#>

param(
    [ValidateSet("audit", "apply")]
    [string]$Mode = "audit",

    [switch]$PruneLocalTagsNotOnRemote,
    [switch]$CleanOrphanNotes,
    [switch]$SkipRemote,
    [switch]$Strict,
    [string]$RepoRoot = ""
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = Split-Path -Parent $PSScriptRoot
}

Set-Location $RepoRoot

function Get-LocalSemverTags {
    $raw = git tag --list "v*"
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to read local tags"
    }

    return @($raw |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
        ForEach-Object { $_.Trim() } |
        Where-Object { $_ -match '^v\d+\.\d+\.\d+([-.].+)?$' } |
        Sort-Object -Unique)
}

function Get-RemoteSemverTags {
    if ($SkipRemote) {
        return @()
    }

    git fetch --tags --prune --quiet
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Warning: failed to fetch remote tags, skip remote sync check." -ForegroundColor Yellow
        return @()
    }

    $raw = git ls-remote --tags origin "v*"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Warning: failed to list remote tags, skip remote sync check." -ForegroundColor Yellow
        return @()
    }

    return @($raw |
        ForEach-Object {
            $parts = ($_ -split "`t")
            if ($parts.Count -lt 2) {
                return
            }
            $ref = $parts[1]
            if ($ref -match '^refs/tags/(v\d+\.\d+\.\d+([-.].+)?)\^?\{?\}?$') {
                $matches[1]
            }
        } |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
        Sort-Object -Unique)
}

function Get-ReleaseNoteMap([string]$releaseNotesDir) {
    $map = @{}
    $files = Get-ChildItem -Path $releaseNotesDir -Filter "RELEASE_NOTES_v*.md" -File -ErrorAction SilentlyContinue
    foreach ($file in $files) {
        $m = [regex]::Match($file.Name, '^RELEASE_NOTES_(v\d+\.\d+\.\d+(?:[-.].+)?)\.md$')
        if (-not $m.Success) {
            continue
        }
        $tag = $m.Groups[1].Value
        $map[$tag] = $file.FullName
    }
    return $map
}

function Remove-LocalTag([string]$tag) {
    git tag -d $tag | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to delete local tag: $tag"
    }
}

$releaseNotesDir = Join-Path $RepoRoot "release_notes"
if (-not (Test-Path $releaseNotesDir)) {
    throw "release_notes directory not found: $releaseNotesDir"
}

$remoteTags = Get-RemoteSemverTags
$localTags = Get-LocalSemverTags
$noteMap = Get-ReleaseNoteMap -releaseNotesDir $releaseNotesDir
$noteTags = @($noteMap.Keys | Sort-Object -Unique)

$localOnlyTags = @()
if ($remoteTags.Count -gt 0) {
    $localOnlyTags = @($localTags | Where-Object { $remoteTags -notcontains $_ })
}

$orphanNotes = @($noteTags | Where-Object { $localTags -notcontains $_ })
$orphanTags = @($localTags | Where-Object { $noteTags -notcontains $_ })

Write-Host "Release state summary" -ForegroundColor Cyan
Write-Host "- Local semver tags: $($localTags.Count)"
Write-Host "- Remote semver tags: $($remoteTags.Count)"
Write-Host "- Release notes files: $($noteTags.Count)"
Write-Host "- Local-only tags (not on remote): $($localOnlyTags.Count)"
Write-Host "- Orphan notes (no local tag): $($orphanNotes.Count)"
Write-Host "- Orphan tags (no release note): $($orphanTags.Count)"

if ($localOnlyTags.Count -gt 0) {
    Write-Host "Local-only tags:" -ForegroundColor Yellow
    $localOnlyTags | ForEach-Object { Write-Host "  $_" -ForegroundColor Yellow }
}

if ($orphanNotes.Count -gt 0) {
    Write-Host "Orphan release notes:" -ForegroundColor Yellow
    $orphanNotes | ForEach-Object { Write-Host "  RELEASE_NOTES_$_.md" -ForegroundColor Yellow }
}

if ($orphanTags.Count -gt 0) {
    Write-Host "Orphan tags:" -ForegroundColor Yellow
    $orphanTags | ForEach-Object { Write-Host "  $_" -ForegroundColor Yellow }
}

if ($Mode -eq "apply") {
    if ($PruneLocalTagsNotOnRemote -and $remoteTags.Count -gt 0) {
        foreach ($tag in $localOnlyTags) {
            Remove-LocalTag -tag $tag
            Write-Host "Deleted local-only tag: $tag" -ForegroundColor Green
        }
    }

    if ($CleanOrphanNotes) {
        $localTags = Get-LocalSemverTags
        $noteMap = Get-ReleaseNoteMap -releaseNotesDir $releaseNotesDir
        $noteTags = @($noteMap.Keys | Sort-Object -Unique)
        $orphanNotes = @($noteTags | Where-Object { $localTags -notcontains $_ })

        foreach ($tag in $orphanNotes) {
            Remove-Item -Path $noteMap[$tag] -Force
            Write-Host "Deleted orphan release note: RELEASE_NOTES_$tag.md" -ForegroundColor Green
        }
    }

    $indexScript = Join-Path $PSScriptRoot "update-release-index.ps1"
    & $indexScript -RepoRoot $RepoRoot
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to update release index"
    }
}

$issueCount = $localOnlyTags.Count + $orphanNotes.Count + $orphanTags.Count
if ($Strict -and $issueCount -gt 0) {
    exit 2
}

Write-Host "Release state $Mode completed." -ForegroundColor Green
