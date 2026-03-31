#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
	Bump workspace versions and create release tag.
.DESCRIPTION
	Updates Cargo.toml versions across rust_serial projects, generates a release
	notes draft, and creates an annotated git tag.
#>

param(
	[ValidateSet("patch", "minor", "major")]
	[string]$Part = "patch",
	[switch]$Push,
	[switch]$NoVerify,
	[switch]$AllowDirty,
	[switch]$NoTag
)

$ErrorActionPreference = "Stop"

$ScriptDir = $PSScriptRoot
$RepoRoot = Split-Path -Parent $ScriptDir
Set-Location $RepoRoot

function Update-ReleaseIndex {
	$indexScript = Join-Path $ScriptDir "update-release-index.ps1"
	if (-not (Test-Path $indexScript)) {
		throw "Missing release index updater: $indexScript"
	}

	& $indexScript -RepoRoot $RepoRoot
	if ($LASTEXITCODE -ne 0) {
		throw "Failed to update release index"
	}
}

function Get-CurrentBranch {
	return (git rev-parse --abbrev-ref HEAD).Trim()
}

function Get-ProjectManifests {
	$manifests = @(
		"robot_control_rust/Cargo.toml",
		"rust_micro_tools/Cargo.toml"
	)

	$indie = Get-ChildItem "rust_indie_tools" -Directory -ErrorAction SilentlyContinue |
		Where-Object { Test-Path (Join-Path $_.FullName "Cargo.toml") } |
		ForEach-Object {
			$relative = $_.FullName.Replace($RepoRoot + "\\", "")
			"$relative/Cargo.toml"
		}

	return $manifests + $indie
}

function Get-VersionFromManifest([string]$manifestPath) {
	$content = Get-Content -Path $manifestPath -Raw -Encoding UTF8
	$m = [regex]::Match($content, '(?m)^version\s*=\s*"(\d+\.\d+\.\d+)"\s*$')
	if (-not $m.Success) {
		throw "Cannot read version from $manifestPath"
	}
	return $m.Groups[1].Value
}

function Get-NextVersion([string]$current, [string]$part) {
	$chunks = $current.Split('.') | ForEach-Object { [int]$_ }
	$major = $chunks[0]
	$minor = $chunks[1]
	$patch = $chunks[2]

	switch ($part) {
		"major" { $major += 1; $minor = 0; $patch = 0 }
		"minor" { $minor += 1; $patch = 0 }
		default { $patch += 1 }
	}

	return "$major.$minor.$patch"
}

function Update-ManifestVersion([string]$manifestPath, [string]$newVersion) {
	$content = Get-Content -Path $manifestPath -Raw -Encoding UTF8
	$updated = [regex]::Replace(
		$content,
		'(?m)^version\s*=\s*"\d+\.\d+\.\d+"\s*$',
		"version = `"$newVersion`"",
		1
	)
	Set-Content -Path $manifestPath -Value $updated -Encoding UTF8
}

function Ensure-CleanWorktree {
	param([switch]$AllowDirty)
	if ($AllowDirty) {
		return
	}
	$status = git status --porcelain
	if (-not [string]::IsNullOrWhiteSpace($status)) {
		throw "Working tree is not clean. Commit or stash changes first, or use -AllowDirty."
	}
}

function Ensure-ReleaseBranch {
	$branch = Get-CurrentBranch
	if ($branch -notin @("main", "master")) {
		throw "Release tagging is restricted to main/master. Current branch: $branch"
	}
}

function Ensure-TagNotExists([string]$tagName) {
	$local = git tag -l $tagName
	if (-not [string]::IsNullOrWhiteSpace($local)) {
		throw "Tag already exists locally: $tagName"
	}

	git fetch --tags --quiet
	$remote = git ls-remote --tags origin $tagName
	if (-not [string]::IsNullOrWhiteSpace($remote)) {
		throw "Tag already exists on origin: $tagName"
	}
}

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

Ensure-CleanWorktree -AllowDirty:$AllowDirty
Ensure-ReleaseBranch

$anchorManifest = "robot_control_rust/Cargo.toml"
$currentVersion = Get-VersionFromManifest -manifestPath $anchorManifest
$nextVersion = Get-NextVersion -current $currentVersion -part $Part
$tagName = "v$nextVersion"

Ensure-TagNotExists -tagName $tagName

$manifests = Get-ProjectManifests
foreach ($manifest in $manifests) {
	if (Test-Path $manifest) {
		Update-ManifestVersion -manifestPath $manifest -newVersion $nextVersion
		Write-Host "Updated $manifest -> $nextVersion" -ForegroundColor Cyan
	}
}

$releaseNotesDir = Join-Path $RepoRoot "release_notes"
New-Item -ItemType Directory -Path $releaseNotesDir -Force | Out-Null
$releaseNotesPath = Join-Path $releaseNotesDir "RELEASE_NOTES_$tagName.md"

$notes = @"
# $tagName

## Highlights
- Describe major improvements here.

## Fixes
- Describe bug fixes here.

## Verification
- [ ] make.ps1 preflight
- [ ] CI passed
- [ ] Release assets verified (exe/setup/checksums)
"@
Set-Content -Path $releaseNotesPath -Value $notes -Encoding UTF8

Update-ReleaseIndex
$releaseIndexPath = Join-Path $releaseNotesDir "RELEASE_INDEX.md"

Invoke-Git -Command ("git add " + (($manifests + $releaseNotesPath + $releaseIndexPath) -join " ")) -ErrorMessage "Failed to stage bump changes"
Invoke-Git -Command "git commit -m 'chore(release): bump version to $tagName'" -ErrorMessage "Failed to create bump commit"

if (-not $NoTag) {
	Invoke-Git -Command "git tag -a $tagName -m 'Release $tagName'" -ErrorMessage "Failed to create release tag"
	Write-Host "Created tag: $tagName" -ForegroundColor Green
}

if ($Push) {
	$verifyFlag = if ($NoVerify) { " --no-verify" } else { "" }
	Invoke-Git -Command ("git push$verifyFlag origin HEAD") -ErrorMessage "Failed to push branch"
	if (-not $NoTag) {
		Invoke-Git -Command ("git push$verifyFlag origin $tagName") -ErrorMessage "Failed to push tag"
	}
	Write-Host "Pushed branch and tag to origin" -ForegroundColor Green
}

Write-Host "Release bump completed: $currentVersion -> $nextVersion" -ForegroundColor Green
