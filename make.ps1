<#
.SYNOPSIS
    Unified task runner for the rust_serial workspace.
.DESCRIPTION
    PowerShell entrypoint equivalent to the root Makefile.
#>
param(
    [Parameter(Position = 0)]
    [ValidateSet("all", "check", "fmt", "fmt-check", "clippy", "test", "test-release",
                 "build", "release", "doc", "audit", "clean", "preflight", "help")]
    [string]$Target = "help"
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Get-Location).Path
$AuditDbPath = Join-Path $RepoRoot ".cargo-advisory-db"

$CoreProjects = @("robot_control_rust", "rust_micro_tools")
$IndieProjects = Get-ChildItem "rust_indie_tools" -Directory -ErrorAction SilentlyContinue |
    Where-Object { Test-Path "$($_.FullName)\Cargo.toml" } |
    ForEach-Object { "rust_indie_tools\$($_.Name)" }
$AllProjects = $CoreProjects + $IndieProjects

function Write-Header($Message) {
    Write-Host "`n== $Message ==" -ForegroundColor Cyan
}

function Show-FailureGuidance {
    param(
        [string]$Summary,
        [string]$SuggestedCommand,
        [string]$FixDirection,
        [string]$WhereToInspect
    )
    Write-Host ""
    Write-Host "Problem: $Summary" -ForegroundColor Red
    Write-Host "Suggested command: $SuggestedCommand" -ForegroundColor Yellow
    Write-Host "Fix direction: $FixDirection" -ForegroundColor Cyan
    Write-Host "Inspect first: $WhereToInspect" -ForegroundColor DarkGray
}

function Invoke-ForEachProject {
    param(
        [scriptblock]$Action,
        [string[]]$ProjectsToRun = $AllProjects
    )
    foreach ($Project in $ProjectsToRun) {
        Write-Host "-> $Project" -ForegroundColor DarkGray
        & $Action $Project
        if ($LASTEXITCODE -ne 0) {
            Write-Host "FAILED: $Project" -ForegroundColor Red
            exit 1
        }
    }
}

switch ($Target) {
		"all" {
		Write-Header "Run all checks (fmt-check + clippy + test + build)"
		& $PSCommandPath fmt-check
		& $PSCommandPath clippy
		& $PSCommandPath test
		& $PSCommandPath build
		Write-Host "`nAll checks passed." -ForegroundColor Green
		}

    "fmt" {
        Write-Header "Format code"
        Invoke-ForEachProject -Action { param($Project)
            cargo fmt --manifest-path "$Project\Cargo.toml"
        }
    }
    "fmt-check" {
        Write-Header "Check formatting"
        Invoke-ForEachProject -Action { param($Project)
            cargo fmt --check --manifest-path "$Project\Cargo.toml"
            if ($LASTEXITCODE -ne 0) {
                Show-FailureGuidance `
                    "$Project contains unformatted Rust code" `
                    "cargo fmt --manifest-path $Project\Cargo.toml" `
                    "Format this project, then rerun .\make.ps1 fmt-check" `
                    "Recently edited .rs files under $Project"
            }
        }
    }
    "clippy" {
        Write-Header "Run clippy"
        Invoke-ForEachProject -Action { param($Project)
            cargo clippy --manifest-path "$Project\Cargo.toml" --all-targets -- -D warnings
            if ($LASTEXITCODE -ne 0) {
                Show-FailureGuidance `
                    "$Project failed clippy" `
                    "cargo clippy --manifest-path $Project\Cargo.toml --all-targets -- -D warnings" `
                    "Fix warning-level findings first, then resolve specific lints" `
                    "The first error or warning in the clippy log"
            }
        }
    }
    "test" {
        Write-Header "Run tests"
        Invoke-ForEachProject -Action { param($Project)
            cargo test --manifest-path "$Project\Cargo.toml"
            if ($LASTEXITCODE -ne 0) {
                Show-FailureGuidance `
                    "$Project tests failed" `
                    "cargo test --manifest-path $Project\Cargo.toml" `
                    "Reproduce the failing test first, then separate assertion, fixture, and platform issues" `
                    "The failing test name and owning module"
            }
        }
    }
    "test-release" {
        Write-Header "Run release tests"
        Invoke-ForEachProject -Action { param($Project)
            cargo test --release --manifest-path "$Project\Cargo.toml"
            if ($LASTEXITCODE -ne 0) {
                Show-FailureGuidance `
                    "$Project release tests failed" `
                    "cargo test --release --manifest-path $Project\Cargo.toml" `
                    "Check release-only behavior, feature gates, and initialization paths" `
                    "The failing release-mode test case"
            }
        }
    }
    "build" {
        Write-Header "Debug build"
        Invoke-ForEachProject -ProjectsToRun $CoreProjects -Action { param($Project)
            cargo build --manifest-path "$Project\Cargo.toml"
            if ($LASTEXITCODE -ne 0) {
                Show-FailureGuidance `
                    "$Project debug build failed" `
                    "cargo build --manifest-path $Project\Cargo.toml" `
                    "Fix compilation errors first, then verify dependencies and feature flags" `
                    "The first compiler error in the build log"
            }
        }
    }
    "release" {
        Write-Header "Release build"
        Invoke-ForEachProject -ProjectsToRun $CoreProjects -Action { param($Project)
            cargo build --release --manifest-path "$Project\Cargo.toml"
            if ($LASTEXITCODE -ne 0) {
                Show-FailureGuidance `
                    "$Project release build failed" `
                    "cargo build --release --manifest-path $Project\Cargo.toml" `
                    "Check system dependencies, target configuration, and release-only code paths" `
                    "The first compiler error in the release build log"
            }
        }
        Write-Header "Build artifacts"
        Get-ChildItem "robot_control_rust\target\release\robot_control_rust*" -ErrorAction SilentlyContinue |
            Select-Object Name, @{N="SizeMB";E={[math]::Round($_.Length / 1MB, 2)}}
        Get-ChildItem "rust_micro_tools\target\release\rust_micro_tools*" -ErrorAction SilentlyContinue |
            Select-Object Name, @{N="SizeMB";E={[math]::Round($_.Length / 1MB, 2)}}
    }
    "doc" {
        Write-Header "Build docs"
        Invoke-ForEachProject -Action { param($Project)
            $DocCommand = '$env:RUSTDOCFLAGS="-D warnings"; cargo doc --no-deps --manifest-path '
            $DocCommand += "$Project\Cargo.toml"
            $env:RUSTDOCFLAGS = "-D warnings"
            cargo doc --no-deps --manifest-path "$Project\Cargo.toml"
            if ($LASTEXITCODE -ne 0) {
                Show-FailureGuidance `
                    "$Project docs build failed" `
                    $DocCommand `
                    "Fix rustdoc warnings and invalid docs comments" `
                    "The first rustdoc warning or error"
            }
            Remove-Item Env:RUSTDOCFLAGS -ErrorAction SilentlyContinue
        }
    }
    "audit" {
        Write-Header "Run cargo-audit and cargo-deny"
        foreach ($Project in $AllProjects) {
            Write-Host "-> $Project" -ForegroundColor DarkGray
            $LockFile = Join-Path $Project "Cargo.lock"
            cargo audit -d $AuditDbPath -f $LockFile
            if ($LASTEXITCODE -ne 0) {
                Show-FailureGuidance `
                    "$Project security audit failed" `
                    "cargo audit -d $AuditDbPath -f $Project\Cargo.lock" `
                    "Upgrade vulnerable dependencies first, then consider regenerating Cargo.lock" `
                    "The advisory ID and affected crate in the audit output"
                exit 1
            }
            Push-Location $Project
            cargo deny check advisories bans sources --config "$RepoRoot\deny.toml"
            $DenyExit = $LASTEXITCODE
            Pop-Location
            if ($DenyExit -ne 0) {
                Show-FailureGuidance `
                    "$Project dependency policy check failed" `
                    "cargo deny check advisories bans sources --config $RepoRoot\deny.toml" `
                    "Review source policy, advisories, and duplicate dependency bans" `
                    "The first cargo-deny error for the current project"
                exit 1
            }
        }
    }
    "clean" {
        Write-Header "Clean build artifacts"
        Invoke-ForEachProject -Action { param($Project)
            cargo clean --manifest-path "$Project\Cargo.toml"
        }
    }
    "check" {
        Write-Header "Fast validation"
        & $PSCommandPath fmt-check
        & $PSCommandPath clippy
        & $PSCommandPath test
        Write-Host "`nAll checks passed." -ForegroundColor Green
    }
    "preflight" {
        Write-Header "Preflight validation"
        & $PSCommandPath fmt-check
        & $PSCommandPath clippy
        & $PSCommandPath test
        & $PSCommandPath test-release
        & $PSCommandPath release
        & $PSCommandPath doc
        Write-Host "`nPreflight passed. Ready to release." -ForegroundColor Green
    }
    "help" {
        Write-Host @"

  rust_serial unified task runner
  =======================================

  Usage: .\make.ps1 <target>

  Targets:
    all          fmt-check + clippy + test + build
    check        format + clippy + test
    fmt          format all Rust projects
    fmt-check    verify formatting without rewriting files
    clippy       run clippy with -D warnings
    test         run tests for all Rust projects
    test-release run tests in release mode
    build        debug build for release projects
    release      release build for release projects
    doc          build docs with rustdoc warnings denied
    audit        run cargo-audit for all Rust projects
    preflight    full validation before release
    clean        clean all target directories
    help         show this help

"@ -ForegroundColor Cyan
    }
}
