#!/usr/bin/env pwsh
#Requires -Version 7.0

param(
    [string]$RepoRoot = ""
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = Split-Path -Parent $PSScriptRoot
}

$releaseNotesDir = Join-Path $RepoRoot "release_notes"
$archiveRoot = Join-Path $releaseNotesDir "archive_assets"
$indexPath = Join-Path $releaseNotesDir "RELEASE_INDEX.md"

if (-not (Test-Path $releaseNotesDir)) {
    throw "release_notes directory not found: $releaseNotesDir"
}

$noteFiles = Get-ChildItem -Path $releaseNotesDir -Filter "RELEASE_NOTES_v*.md" -File -ErrorAction SilentlyContinue |
    Sort-Object Name

$rows = @()
foreach ($note in $noteFiles) {
    $m = [regex]::Match($note.Name, '^RELEASE_NOTES_(v\d+\.\d+\.\d+(?:[-.].+)?)\.md$')
    if (-not $m.Success) {
        continue
    }

    $tag = $m.Groups[1].Value
    $version = $tag.TrimStart('v')

    $archiveDir = Join-Path $archiveRoot $tag
    if (Test-Path $archiveDir) {
        $archivedFiles = Get-ChildItem -Path $archiveDir -File -Recurse -ErrorAction SilentlyContinue
        if ($null -ne $archivedFiles -and $archivedFiles.Count -gt 0) {
            $archiveStatus = "archived"
            $archivePath = "release_notes/archive_assets/$tag"
        }
        else {
            $archiveStatus = "empty"
            $archivePath = "release_notes/archive_assets/$tag"
        }
    }
    else {
        $archiveStatus = "not-archived"
        $archivePath = "-"
    }

    $rows += [PSCustomObject]@{
        Version = $version
        Tag = $tag
        ReleaseNotes = "release_notes/$($note.Name)"
        LocalArchiveStatus = $archiveStatus
        LocalArchivePath = $archivePath
    }
}

$header = @(
    "# Release Index",
    "",
    "此文件由 scripts/update-release-index.ps1 生成，用于记录版本、Tag 与本地归档状态。",
    "",
    "| Version | Tag | Release Notes | Local Archive Status | Local Archive Path |",
    "|---|---|---|---|---|"
)

$body = @()
foreach ($row in $rows) {
    $body += "| $($row.Version) | $($row.Tag) | $($row.ReleaseNotes) | $($row.LocalArchiveStatus) | $($row.LocalArchivePath) |"
}

if ($body.Count -eq 0) {
    $body += "| - | - | - | - | - |"
}

$content = ($header + $body + @("", "更新时间(UTC): $(Get-Date -AsUTC -Format 'yyyy-MM-dd HH:mm:ss')")) -join "`n"
Set-Content -Path $indexPath -Value $content -Encoding UTF8

Write-Host "Updated release index: $indexPath" -ForegroundColor Green
