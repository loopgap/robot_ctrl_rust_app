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

function Get-ScriptExitCode {
	if ($null -eq $LASTEXITCODE) {
		if ($?) {
			return 0
		}
		return 1
	}
	return [int]$LASTEXITCODE
}

Write-Header "Pre-commit Check"
Write-Step "Git workflow validation"
$global:LASTEXITCODE = 0
& "$ScriptsDir\git-check.ps1"
$exitCode = Get-ScriptExitCode
if ($exitCode -ne 0) {
	Write-Error "Git workflow check failed"
	exit $exitCode
}
Write-Success "Git workflow validation passed"

Write-Step "Workspace process-file residue audit"
$global:LASTEXITCODE = 0
& "$ScriptsDir\cleanup-process-files.ps1" -Mode audit -Strict
$exitCode = Get-ScriptExitCode
if ($exitCode -ne 0) {
	Write-Error "Found transient process files in workspace. Run: ./make.ps1 workspace-cleanup"
	exit $exitCode
}

Write-Step "Workspace path policy (staged files)"
$global:LASTEXITCODE = 0
& "$ScriptsDir\enforce-workspace-structure.ps1" -Mode audit -Strict -UseStagedPaths
$exitCode = Get-ScriptExitCode
if ($exitCode -ne 0) {
	Write-Error "Found misplaced or blocked staged paths. Move files into standard directories first."
	exit $exitCode
}

Write-Success "Workspace structure checks passed"

exit $script:ExitCode
