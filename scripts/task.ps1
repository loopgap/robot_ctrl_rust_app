<#
.SYNOPSIS
    Workspace task entrypoint.
.DESCRIPTION
    Compatibility wrapper for the Windows task runner under scripts/windows.
#>
$ErrorActionPreference = "Stop"
$ScriptsDir = Split-Path -Parent $PSCommandPath
$Impl = Join-Path $ScriptsDir "windows\task.ps1"

if (-not (Test-Path $Impl)) {
    throw "Windows task runner not found: $Impl"
}

& $Impl @args

if ($null -ne $LASTEXITCODE) {
    exit $LASTEXITCODE
}
if (-not $?) {
    exit 1
}
