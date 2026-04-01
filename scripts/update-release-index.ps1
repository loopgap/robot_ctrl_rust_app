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

function Get-TagInfo([string]$tag) {
    $m = [regex]::Match($tag, '^v(?<major>\d+)\.(?<minor>\d+)\.(?<patch>\d+)(?<suffix>[-.].+)?$')
    if (-not $m.Success) {
        return $null
    }

    return [PSCustomObject]@{
        Tag = $tag
        Version = $tag.TrimStart('v')
        Major = [int]$m.Groups['major'].Value
        Minor = [int]$m.Groups['minor'].Value
        Patch = [int]$m.Groups['patch'].Value
        Suffix = $m.Groups['suffix'].Value
        SuffixRank = if ([string]::IsNullOrWhiteSpace($m.Groups['suffix'].Value)) { 1 } else { 0 }
    }
}

$noteMap = @{}
$noteFiles = Get-ChildItem -Path $releaseNotesDir -Filter "RELEASE_NOTES_v*.md" -File -ErrorAction SilentlyContinue
foreach ($note in $noteFiles) {
    $m = [regex]::Match($note.Name, '^RELEASE_NOTES_(v\d+\.\d+\.\d+(?:[-.].+)?)\.md$')
    if ($m.Success) {
        $noteMap[$m.Groups[1].Value] = $note.Name
    }
}

$localTagsRaw = git tag --list "v*"
if ($LASTEXITCODE -ne 0) {
    throw "Failed to read local git tags"
}

$remoteTags = @()
$remoteTagStatusEnabled = $true
git fetch --tags --prune --quiet
if ($LASTEXITCODE -eq 0) {
    $remoteTagsRaw = git ls-remote --tags origin "v*"
    if ($LASTEXITCODE -eq 0) {
        $remoteTags = @($remoteTagsRaw |
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
    else {
        $remoteTagStatusEnabled = $false
    }
}
else {
    $remoteTagStatusEnabled = $false
}

$localTags = @($localTagsRaw |
    Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
    ForEach-Object { $_.Trim() } |
    Where-Object { $_ -match '^v\d+\.\d+\.\d+([-.].+)?$' } |
    Sort-Object -Unique)

$allTags = @((@($noteMap.Keys) + @($localTags)) |
    Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
    Sort-Object -Unique)

$rows = @()
foreach ($tag in $allTags) {
    $tagInfo = Get-TagInfo -tag $tag
    if ($null -eq $tagInfo) {
        continue
    }

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

    $releaseNotesPath = if ($noteMap.ContainsKey($tag)) { "release_notes/$($noteMap[$tag])" } else { "-" }
    $localTagStatus = if ($localTags -contains $tag) { "present" } else { "missing" }
    $remoteTagStatus = if (-not $remoteTagStatusEnabled) { "unknown" } elseif ($remoteTags -contains $tag) { "present" } else { "missing" }

    $rows += [PSCustomObject]@{
        Major = $tagInfo.Major
        Minor = $tagInfo.Minor
        Patch = $tagInfo.Patch
        SuffixRank = $tagInfo.SuffixRank
        Suffix = $tagInfo.Suffix
        Version = $tagInfo.Version
        Tag = $tag
        LocalTagStatus = $localTagStatus
        RemoteTagStatus = $remoteTagStatus
        ReleaseNotes = $releaseNotesPath
        LocalArchiveStatus = $archiveStatus
        LocalArchivePath = $archivePath
    }
}

$rows = @($rows | Sort-Object -Property @(
    @{ Expression = { $_.Major }; Descending = $true },
    @{ Expression = { $_.Minor }; Descending = $true },
    @{ Expression = { $_.Patch }; Descending = $true },
    @{ Expression = { $_.SuffixRank }; Descending = $true },
    @{ Expression = { $_.Suffix }; Descending = $true }))

$header = @(
    "# Release Index",
    "",
    "此文件由 scripts/update-release-index.ps1 生成，用于记录版本、Tag、本地/远端 Tag 状态与归档状态。",
    "",
    "| Version | Tag | Local Tag Status | Remote Tag Status | Release Notes | Local Archive Status | Local Archive Path |",
    "|---|---|---|---|---|---|---|"
)

$body = @()
foreach ($row in ($rows | Where-Object { -not [string]::IsNullOrWhiteSpace($_.Tag) })) {
    $body += "| $($row.Version) | $($row.Tag) | $($row.LocalTagStatus) | $($row.RemoteTagStatus) | $($row.ReleaseNotes) | $($row.LocalArchiveStatus) | $($row.LocalArchivePath) |"
}

if ($body.Count -eq 0) {
    $body += "| - | - | - | - | - | - | - |"
}

$content = ($header + $body + @("", "更新时间(UTC): $(Get-Date -AsUTC -Format 'yyyy-MM-dd HH:mm:ss')")) -join "`n"
Set-Content -Path $indexPath -Value $content -Encoding UTF8

Write-Host "Updated release index: $indexPath" -ForegroundColor Green
