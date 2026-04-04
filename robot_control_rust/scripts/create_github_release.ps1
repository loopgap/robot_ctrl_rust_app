param(
    [string]$Owner = "loopgap",
    [string]$Repo = "robot_ctrl_rust_app",
    [string]$Tag = "",
    [string]$ReleaseName = "",
    [string]$BodyFile = "",
    [string[]]$Assets = @(),
    [switch]$Prerelease,
    [switch]$Draft,
    [switch]$PruneExtraAssets
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $PSCommandPath
$RepoRoot = Split-Path -Parent (Split-Path -Parent $ScriptDir)
Set-Location $RepoRoot

if ([string]::IsNullOrWhiteSpace($Tag)) {
    throw "Missing -Tag (example: v0.1.7)."
}

if ([string]::IsNullOrWhiteSpace($ReleaseName)) {
    $ReleaseName = $Tag
}

if ([string]::IsNullOrWhiteSpace($BodyFile)) {
    $BodyFile = "release_notes/RELEASE_NOTES_$Tag.md"
}

if ($null -eq $Assets -or $Assets.Count -eq 0) {
    $Assets = @(
        "release_artifacts/robot_control_rust_windows_x64_portable.zip",
        "release_artifacts/rust_tools_suite_windows_x64_portable.zip",
        "release_artifacts/rust_tools_suite_linux_amd64.deb",
        "release_artifacts/RobotControlSuite_Setup.exe",
        "release_artifacts/checksums-sha256.txt"
    )
}

if (-not $env:GITHUB_TOKEN) {
    throw "Missing GITHUB_TOKEN environment variable."
}

if (-not (Test-Path $BodyFile)) {
    throw "Release notes file not found: $BodyFile"
}

foreach ($asset in $Assets) {
    if (-not (Test-Path $asset)) {
        throw "Asset not found: $asset"
    }
}

$headers = @{
    Authorization = "Bearer $($env:GITHUB_TOKEN)"
    Accept = "application/vnd.github+json"
    "X-GitHub-Api-Version" = "2022-11-28"
}

$bodyText = [string](Get-Content -Raw -Path $BodyFile -Encoding UTF8)

$createPayload = [ordered]@{
    tag_name   = $Tag
    name       = $ReleaseName
    body       = $bodyText
    draft      = [bool]$Draft
    prerelease = [bool]$Prerelease
} | ConvertTo-Json -Depth 5 -Compress
$createPayloadBytes = [System.Text.Encoding]::UTF8.GetBytes($createPayload)

$createUri = "https://api.github.com/repos/$Owner/$Repo/releases"
try {
    $release = Invoke-RestMethod -Method Post -Uri $createUri -Headers $headers -Body $createPayloadBytes -ContentType "application/json; charset=utf-8"
}
catch {
    $statusCode = $null
    if ($_.Exception.Response) {
        $statusCode = $_.Exception.Response.StatusCode.Value__
    }
    if ($statusCode -eq 422) {
        $release = Invoke-RestMethod -Method Get -Uri "https://api.github.com/repos/$Owner/$Repo/releases/tags/$Tag" -Headers $headers
        $updatePayload = [ordered]@{
            name       = $ReleaseName
            body       = $bodyText
            draft      = [bool]$Draft
            prerelease = [bool]$Prerelease
        } | ConvertTo-Json -Depth 5 -Compress
        $updatePayloadBytes = [System.Text.Encoding]::UTF8.GetBytes($updatePayload)
        $release = Invoke-RestMethod -Method Patch -Uri "https://api.github.com/repos/$Owner/$Repo/releases/$($release.id)" -Headers $headers -Body $updatePayloadBytes -ContentType "application/json; charset=utf-8"
    }
    else {
        throw
    }
}

$baseUploadUrl = ([string]$release.upload_url).Replace('{?name,label}', '')
$desiredAssetNames = @($Assets | ForEach-Object { [System.IO.Path]::GetFileName($_) })

if ($PruneExtraAssets) {
    foreach ($item in @($release.assets)) {
        if ($desiredAssetNames -notcontains $item.name) {
            Invoke-RestMethod -Method Delete -Uri "https://api.github.com/repos/$Owner/$Repo/releases/assets/$($item.id)" -Headers $headers
        }
    }
}

foreach ($asset in $Assets) {
    $assetName = [System.IO.Path]::GetFileName($asset)
    $existing = @($release.assets | Where-Object { $_.name -eq $assetName })
    foreach ($item in $existing) {
        Invoke-RestMethod -Method Delete -Uri "https://api.github.com/repos/$Owner/$Repo/releases/assets/$($item.id)" -Headers $headers
    }
    $uploadUri = "$($baseUploadUrl)?name=$([System.Uri]::EscapeDataString($assetName))"
    Invoke-RestMethod -Method Post -Uri $uploadUri -Headers $headers -InFile $asset -ContentType "application/octet-stream"
}

Write-Output "Release created: $($release.html_url)"
