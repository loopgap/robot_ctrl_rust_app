<#
.SYNOPSIS
    Windows task launcher.
.DESCRIPTION
    Windows-only entrypoint under scripts/windows that delegates to make.ps1.
#>
# Keep this script parameterless so named flags are passed through unchanged.
$PassthroughArgs = @($args)

$ErrorActionPreference = "Stop"
$ScriptsDir = Split-Path -Parent $PSCommandPath
$Impl = Join-Path $ScriptsDir "make.ps1"

if (-not (Test-Path $Impl)) {
    throw "Windows task implementation not found: $Impl"
}

& $Impl @PassthroughArgs

if ($null -ne $LASTEXITCODE) {
    exit $LASTEXITCODE
}
if (-not $?) {
    exit 1
}
