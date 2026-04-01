#!/usr/bin/env pwsh
#Requires -Version 7.0

# Pre-commit Hook
$ErrorActionPreference = "Stop"
$script:ExitCode = 0

# Get script directory (hooks/)
$HookDir = Split-Path -Parent $MyInvocation.MyCommand.Path
# Get scripts directory (parent of hooks/)
$ScriptsDir = Split-Path -Parent $HookDir
# Module path
$ModulePath = Join-Path $ScriptsDir 'common.psm1'

# Import common module
Import-Module $ModulePath -Force -Global

Write-Header "Pre-commit Check"
Write-Step "Git workflow validation"
& "$ScriptsDir\git-check.ps1"
if ($LASTEXITCODE -ne 0) {
	Write-Error "Git workflow check failed"
	exit $LASTEXITCODE
}
Write-Success "Git workflow validation passed"

Write-Step "Workspace process-file residue audit"
& "$ScriptsDir\cleanup-process-files.ps1" -Mode audit -Strict
if ($LASTEXITCODE -ne 0) {
	Write-Error "Found transient process files in workspace. Run: ./make.ps1 workspace-cleanup"
	exit 1
}

Write-Step "Workspace path policy (staged files)"
& "$ScriptsDir\enforce-workspace-structure.ps1" -Mode audit -Strict -UseStagedPaths
if ($LASTEXITCODE -ne 0) {
	Write-Error "Found misplaced or blocked staged paths. Move files into standard directories first."
	exit 1
}

Write-Success "Workspace structure checks passed"

exit $script:ExitCode
