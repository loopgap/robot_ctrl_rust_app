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
Write-Success "Module import successful"

exit $script:ExitCode
