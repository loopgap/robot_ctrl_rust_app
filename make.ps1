# robot_ctrl_rust_app Workspace - PowerShell Build Script
#
# Usage:
#   .\make.ps1 <target>
#
# Targets: all, check, fmt, fmt-check, clippy, test, test-release,
#          build, release, doc, audit, clean, preflight, help

param(
    [Parameter(Position = 0)]
    [ValidateSet("all", "check", "check-parallel", "fmt", "fmt-check", "clippy", "test", "test-release",
                 "build", "build-parallel", "release", "release-parallel", "build-all-parallel",
                 "test-all-parallel", "doc", "audit", "clean", "preflight", "help")]
    [string]$Target = "help"
)

$ErrorActionPreference = "Stop"
$RepoRoot = $PSScriptRoot

# Crate names
$RobotCore = "crates/robot_core"
$RobotControl = "crates/robot_control"
$ToolsSuite = "crates/tools_suite"
$Devtools = "crates/devtools"

$AllCrates = @($RobotCore, $RobotControl, $ToolsSuite, $Devtools)
$BuildCrates = @($RobotControl, $ToolsSuite)

# Get CPU count for parallel jobs
$CpuCount = [Environment]::ProcessorCount
if ($CpuCount -eq 0) { $CpuCount = 4 }

function Write-Header($Message) {
    Write-Host "`n== $Message ==" -ForegroundColor Cyan
}

function Invoke-Cargo {
    param(
        [string[]]$Arguments
    )
    cargo @Arguments
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

switch ($Target) {
    "all" {
        Write-Header "Run all checks and build"
        & $PSCommandPath "check"
        & $PSCommandPath "build"
        Write-Host "`nAll checks passed." -ForegroundColor Green
    }

    "check" {
        Write-Header "Check (fmt + clippy + test)"
        & $PSCommandPath "fmt-check"
        & $PSCommandPath "clippy"
        & $PSCommandPath "test"
        Write-Host "`n✓ All checks passed" -ForegroundColor Green
    }

    "check-parallel" {
        Write-Header "Check (parallel: fmt + clippy + test)"
        & $PSCommandPath "fmt"
        & $PSCommandPath "clippy"
        & $PSCommandPath "test"
        Write-Host "`n✓ All checks passed (parallel)" -ForegroundColor Green
    }

    "fmt" {
        Write-Header "Formatting code"
        Invoke-Cargo -Arguments "fmt"
    }

    "fmt-check" {
        Write-Header "Checking format"
        Invoke-Cargo -Arguments "fmt", "--check"
    }

    "clippy" {
        Write-Header "Clippy static analysis"
        Invoke-Cargo -Arguments "clippy", "--all-targets", "--", "-D", "warnings"
    }

    "test" {
        Write-Header "Running tests"
        Invoke-Cargo -Arguments "test", "--all"
    }

    "test-release" {
        Write-Header "Running tests in release mode"
        Invoke-Cargo -Arguments "test", "--release", "--all"
    }

    "test-all-parallel" {
        Write-Header "Running all tests in parallel (using $CpuCount jobs)"
        Invoke-Cargo -Arguments "test", "--all", "-j", $CpuCount
    }

    "build" {
        Write-Header "Debug build"
        Invoke-Cargo -Arguments "build", "-p", $RobotControl, "-p", $ToolsSuite
    }

    "build-parallel" {
        Write-Header "Debug build (parallel, using $CpuCount jobs)"
        Invoke-Cargo -Arguments "build", "-p", $RobotControl, "-p", $ToolsSuite, "-j", $CpuCount
    }

    "release" {
        Write-Header "Release build"
        Invoke-Cargo -Arguments "build", "--release", "-p", $RobotControl, "-p", $ToolsSuite
        Write-Host "`n== Build artifacts ==" -ForegroundColor Cyan
        Get-ChildItem target/release/*.exe -ErrorAction SilentlyContinue | ForEach-Object {
            Write-Host $_.Name -ForegroundColor Gray
        }
    }

    "release-parallel" {
        Write-Header "Release build (parallel, using $CpuCount jobs)"
        Invoke-Cargo -Arguments "build", "--release", "-p", $RobotControl, "-p", $ToolsSuite, "-j", $CpuCount
        Write-Host "`n== Build artifacts ==" -ForegroundColor Cyan
        Get-ChildItem target/release/*.exe -ErrorAction SilentlyContinue | ForEach-Object {
            Write-Host $_.Name -ForegroundColor Gray
        }
    }

    "build-all-parallel" {
        Write-Header "Build all crates in parallel (using $CpuCount jobs)"
        Invoke-Cargo -Arguments "build", "--all", "-j", $CpuCount
    }

    "doc" {
        Write-Header "Generating documentation"
        $env:RUSTDOCFLAGS = "-D warnings"
        Invoke-Cargo -Arguments "doc", "--no-deps"
        Remove-Item Env:\RUSTDOCFLAGS
    }

    "audit" {
        Write-Header "Security audit"
        foreach ($crate in $AllCrates) {
            Write-Host "-> $crate" -ForegroundColor DarkGray
            Invoke-Cargo -Arguments "audit", "-f", "$crate/Cargo.lock"
        }
    }

    "clean" {
        Write-Header "Cleaning build artifacts"
        Invoke-Cargo -Arguments "clean"
    }

    "preflight" {
        Write-Header "Preflight checks"
        & $PSCommandPath "fmt-check"
        & $PSCommandPath "clippy"
        & $PSCommandPath "test"
        & $PSCommandPath "test-release"
        & $PSCommandPath "release"
        & $PSCommandPath "doc"
        Write-Host "`n🚀 Preflight all passed! Ready to release." -ForegroundColor Green
    }

    "help" {
        Write-Host @"
Available targets:
  make.ps1 all              Run all checks and build
  make.ps1 check            Format + clippy + test (fast validation)
  make.ps1 check-parallel   Format + clippy + test (parallel)
  make.ps1 fmt               Auto-format all code
  make.ps1 fmt-check         Check format (no modifications)
  make.ps1 clippy            Static analysis
  make.ps1 test              Run all tests
  make.ps1 test-release      Run tests in release mode
  make.ps1 test-all-parallel Run all tests in parallel
  make.ps1 build            Debug build
  make.ps1 build-parallel   Debug build (parallel)
  make.ps1 release          Release build
  make.ps1 release-parallel Release build (parallel)
  make.ps1 build-all-parallel Build all crates in parallel
  make.ps1 doc             Generate documentation
  make.ps1 audit           Security audit
  make.ps1 clean           Clean all build artifacts
  make.ps1 preflight       Full pre-release validation
  make.ps1 help            Show this help
"@
    }
}