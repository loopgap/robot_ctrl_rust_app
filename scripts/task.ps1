<#
.SYNOPSIS
    Cross-platform task launcher (PowerShell).
.DESCRIPTION
    Detects current OS and dispatches to system-specific implementation under scripts/.
#>

# Keep this script parameterless so arbitrary flags (for example -ReleaseNotesFile)
# are not parsed here and can be forwarded to scripts/make.ps1 unchanged.
$PassthroughArgs = @($args)

$ErrorActionPreference = "Stop"
$ScriptsDir = Split-Path -Parent $PSCommandPath
$RepoRoot = Split-Path -Parent $ScriptsDir

$IsWindowsOS = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)

if ($IsWindowsOS) {
    $impl = Join-Path $ScriptsDir "make.ps1"
    if (-not (Test-Path $impl)) {
        throw "Windows task implementation not found: $impl"
    }

    & $impl @PassthroughArgs
}
else {
    $make = Get-Command make -ErrorAction SilentlyContinue
    if (-not $make) {
        throw "make command is required on non-Windows systems. Install make and retry."
    }

    $mk = Join-Path $ScriptsDir "Makefile"
    if (-not (Test-Path $mk)) {
        throw "Unix task implementation not found: $mk"
    }

    Push-Location $RepoRoot
    try {
        & $make.Source "-f" $mk @PassthroughArgs
    }
    finally {
        Pop-Location
    }
}

if ($null -ne $LASTEXITCODE) {
    exit $LASTEXITCODE
}
if (-not $?) {
    exit 1
}
