<#
.SYNOPSIS
    Compatibility wrapper for Windows task implementation.
.DESCRIPTION
    Keeps the public entrypoint at scripts/make.ps1 while delegating to scripts/windows/make.ps1.
#>
# Keep this wrapper parameterless so named flags are passed through unchanged.
$PassthroughArgs = @($args)

$ErrorActionPreference = "Stop"
$ScriptsDir = Split-Path -Parent $PSCommandPath
$Impl = Join-Path $ScriptsDir "windows\make.ps1"

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