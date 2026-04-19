<#
.SYNOPSIS
    Unified task runner for the rust_serial workspace.
.DESCRIPTION
    PowerShell entrypoint equivalent to the root Makefile.
#>
param(
    [Parameter(Position = 0)]
    [ValidateSet("all", "check", "fmt", "fmt-check", "clippy", "test", "test-release",
                 "build", "release", "doc", "audit", "clean", "preflight",
                 "ci-local", "ci-local-full",
                 "release-sync", "release-sync-apply", "release-index", "workflow-seal", "workflow-seal-apply", "workspace-guard", "workspace-cleanup", "release-notes-validate",
                 "smart-bump", "smart-rollback", "pr-helper",
                 "git-check", "git-review-before-push", "git-hooks-install", "git-hooks-uninstall", "git-pr-check", "git-pr-create", "git-pr-merge",
                 "docs-bundle", "release-publish", "build-release-slim",
                 "package-windows-installer", "package-windows-assets", "package-windows-portable-installer",
                 "release-local-windows", "release-local-linux", "release-local-dual",
                 "go-check", "go-preflight", "go-build", "go-test", "go-doc", "go-audit", "go-release-sync", "go-workflow-seal", "go-git-check", "go-rust-review", "go-review", "go-install-hooks", "help")]
    [string]$Target = "help",

    [string]$ReleaseNotesFile = "",

    [ValidateSet("draft", "release")]
    [string]$ReleaseNotesMode = "release",

    [string]$PackageVersion = "",

    [string]$PackageBuildTag = "",

    [string]$PackageOutputDir = "",

    [switch]$PackageSkipBuild,

    [string]$DocsOutputRoot = "",

    [switch]$DocsCreateZip,

    [string]$ReleaseOwner = "loopgap",

    [string]$ReleaseRepo = "robot_ctrl_rust_app",

    [string]$ReleaseTag = "",

    [string]$ReleaseName = "",

    [string]$ReleaseBodyFile = "",

    [string[]]$ReleaseAsset = @(),

    [switch]$ReleasePrerelease,

    [switch]$ReleaseDraft,

    [switch]$ReleasePruneExtraAssets,

    [ValidateSet("patch", "minor", "major")]
    [string]$BumpPart = "patch",

    [switch]$BumpPush,

    [switch]$BumpNoVerify,

    [switch]$BumpAllowDirty,

    [switch]$BumpNoTag,

    [switch]$BumpSkipReleaseStateAudit,

    [switch]$BumpSkipProcessCleanup,

    [switch]$BumpSkipWorkspaceGuard,

    [string]$RollbackTag = "",

    [string]$RollbackOwner = "loopgap",

    [string]$RollbackRepo = "robot_ctrl_rust_app",

    [switch]$RollbackDeleteRelease,

    [switch]$RollbackDeleteRemoteTag,

    [switch]$RollbackDeleteLocalTag,

    [switch]$RollbackRevertLastCommit,

    [switch]$RollbackPushRevert,

    [switch]$RollbackNoVerify,

    [switch]$RollbackSkipProcessCleanup,

    [switch]$RollbackSkipWorkspaceGuard,

    [switch]$RollbackSkipIndexRefresh,

    [switch]$PrCreate,

    [switch]$PrCheck,

    [switch]$PrMerge,

    [switch]$PrDraft,

    [string]$PrTitle = "",

    [string]$PrBody = "",

    [string]$PrBase = "main",

    [string]$PrHead = "",

    [switch]$PrAutoFill
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSCommandPath))
$AuditDbPath = Join-Path $RepoRoot ".cargo-advisory-db"
$AuditIgnoreIds = @("RUSTSEC-2023-0071")
$CoreProjects = @("robot_control_rust", "rust_tools_suite")
$AllProjects = $CoreProjects
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

function Invoke-PwshScript {
    param(
        [string]$ScriptRelativePath,
        [string[]]$Arguments = @()
    )

    $pwsh = Get-Command pwsh -ErrorAction SilentlyContinue
    if (-not $pwsh) {
        throw "pwsh (PowerShell 7+) is required to run $ScriptRelativePath"
    }

    $scriptPath = Join-Path $RepoRoot $ScriptRelativePath
    & $pwsh.Source -NoProfile -File $scriptPath @Arguments
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

function Invoke-WorkspaceGuard {
    Invoke-GoRusktask -Arguments @("workspace-guard", "--mode", "audit", "--strict")
}

function Invoke-WorkspaceCleanup {
    Invoke-GoRusktask -Arguments @("workspace-cleanup", "--mode", "apply")
}

function Invoke-GoRusktask {
    param([string[]]$Arguments = @())

    $go = Get-Command go -ErrorAction SilentlyContinue
    if (-not $go) {
        throw "go command is required. Install Go and retry."
    }

    $rusktaskDir = Join-Path $RepoRoot "scripts\go\rusktask"
    $goModPath = Join-Path $rusktaskDir "go.mod"
    if (-not (Test-Path $goModPath)) {
        throw "rusktask module not found: $goModPath"
    }

    Push-Location $rusktaskDir
    try {
        & $go.Source run . @Arguments
        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
    }
    finally {
        Pop-Location
    }
}

function Invoke-LinuxDebPackage {
    $bash = Get-Command bash -ErrorAction SilentlyContinue
    if (-not $bash) {
        throw "bash is required for Linux packaging. Install Git Bash/WSL and retry."
    }

    $scriptPathNative = Join-Path $RepoRoot "rust_tools_suite\packaging\package_deb.sh"
    if (-not (Test-Path $scriptPathNative)) {
        throw "Linux package script not found: $scriptPathNative"
    }

    $scriptPathBash = $scriptPathNative -replace '\\', '/'
    $args = @($scriptPathBash)
    if (-not [string]::IsNullOrWhiteSpace($PackageVersion)) {
        $args += @("--version", $PackageVersion)
    }
    if ($PackageSkipBuild) {
        $args += "--skip-build"
    }

    & $bash.Source @args
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

function Invoke-MakeSubTarget {
    param([string]$TargetName)

    & $PSCommandPath $TargetName
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
    if (-not $?) {
        exit 1
    }
}

switch ($Target) {
		"all" {
		Write-Header "Run all checks (fmt-check + clippy + test + build)"
        Invoke-MakeSubTarget "workspace-cleanup"
        Invoke-MakeSubTarget "workspace-guard"
		Invoke-MakeSubTarget "fmt-check"
		Invoke-MakeSubTarget "clippy"
		Invoke-MakeSubTarget "test"
		Invoke-MakeSubTarget "build"
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
                    "Format this project, then rerun .\scripts\task.ps1 fmt-check" `
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
        Get-ChildItem "rust_tools_suite\target\release\rust_tools_suite*" -ErrorAction SilentlyContinue |
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
            $AuditArgs = @("audit", "-d", $AuditDbPath, "-f", $LockFile)
            foreach ($IgnoreId in $AuditIgnoreIds) {
                $AuditArgs += @("--ignore", $IgnoreId)
            }
            cargo @AuditArgs
            if ($LASTEXITCODE -ne 0) {
                Show-FailureGuidance `
                    "$Project security audit failed" `
                    "cargo audit -d $AuditDbPath -f $Project\Cargo.lock --ignore $($AuditIgnoreIds -join ' --ignore ')" `
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
        Invoke-MakeSubTarget "workspace-cleanup"
        Invoke-MakeSubTarget "workspace-guard"
        Invoke-MakeSubTarget "fmt-check"
        Invoke-MakeSubTarget "clippy"
        Invoke-MakeSubTarget "test"
        Write-Host "`nAll checks passed." -ForegroundColor Green
    }
    "preflight" {
        Write-Header "Preflight validation"
        Invoke-MakeSubTarget "workspace-cleanup"
        Invoke-MakeSubTarget "workspace-guard"
        Invoke-MakeSubTarget "fmt-check"
        Invoke-MakeSubTarget "clippy"
        Invoke-MakeSubTarget "test"
        Invoke-MakeSubTarget "test-release"
        Invoke-MakeSubTarget "release"
        Invoke-MakeSubTarget "doc"
        Write-Host "`nPreflight passed. Ready to release." -ForegroundColor Green
    }
    "ci-local" {
        Write-Header "Local CI (core)"
        Invoke-GoRusktask -Arguments @("workspace-cleanup", "--mode", "audit", "--strict")
        Invoke-GoRusktask -Arguments @("workspace-guard", "--mode", "audit", "--strict")
        Invoke-MakeSubTarget "fmt-check"
        Invoke-MakeSubTarget "clippy"
        Invoke-MakeSubTarget "test"
        Invoke-MakeSubTarget "doc"
        Write-Host "`nLocal CI core pipeline passed." -ForegroundColor Green
    }
    "ci-local-full" {
        Write-Header "Local CI (full)"
        Invoke-MakeSubTarget "ci-local"
        Invoke-MakeSubTarget "audit"
        Write-Host "`nLocal CI full pipeline passed." -ForegroundColor Green
    }
    "release-sync" {
        Write-Header "Release state audit"
        Invoke-MakeSubTarget "workspace-cleanup"
        Invoke-MakeSubTarget "workspace-guard"
        Invoke-GoRusktask -Arguments @("release-sync", "--mode", "audit")
    }
    "release-sync-apply" {
        Write-Header "Release state normalize"
        Invoke-MakeSubTarget "workspace-cleanup"
        Invoke-GoRusktask -Arguments @("release-sync", "--mode", "apply", "--prune-local-tags-not-on-remote", "--clean-orphan-notes")
        Invoke-MakeSubTarget "workspace-cleanup"
        Invoke-MakeSubTarget "workspace-guard"
    }
    "release-index" {
        Write-Header "Release index rebuild"
        Invoke-GoRusktask -Arguments @("update-release-index")
    }
    "workflow-seal" {
        Write-Header "Workflow seal (audit)"
        Invoke-GoRusktask -Arguments @("workflow-seal", "--mode", "audit")
    }
    "workflow-seal-apply" {
        Write-Header "Workflow seal (apply)"
        Invoke-GoRusktask -Arguments @("workflow-seal", "--mode", "apply", "--prune-local-tags-not-on-remote", "--clean-orphan-notes")
    }
    "workspace-guard" {
        Write-Header "Workspace structure guard"
        Invoke-WorkspaceGuard
    }
    "workspace-cleanup" {
        Write-Header "Workspace process-file cleanup"
        Invoke-WorkspaceCleanup
    }
    "release-notes-validate" {
        Write-Header "Release notes validation"

        $effectiveFile = $ReleaseNotesFile
        if ([string]::IsNullOrWhiteSpace($effectiveFile)) {
            $effectiveFile = $env:RELEASE_NOTES_FILE
        }

        if ([string]::IsNullOrWhiteSpace($effectiveFile)) {
            throw "Release notes file is required. Use -ReleaseNotesFile or set RELEASE_NOTES_FILE."
        }

        $resolved = Resolve-Path -Path $effectiveFile -ErrorAction SilentlyContinue
        if (-not $resolved) {
            throw "Release notes file not found: $effectiveFile"
        }

        Invoke-GoRusktask -Arguments @("release-notes", "validate", "--file", $resolved.Path, "--mode", $ReleaseNotesMode)
    }
    "smart-bump" {
        Write-Header "Smart version bump"
        $args = @("smart-bump", "--part", $BumpPart)
        if ($BumpPush) {
            $args += "--push"
        }
        if ($BumpNoVerify) {
            $args += "--no-verify"
        }
        if ($BumpAllowDirty) {
            $args += "--allow-dirty"
        }
        if ($BumpNoTag) {
            $args += "--no-tag"
        }
        if ($BumpSkipReleaseStateAudit) {
            $args += "--skip-release-state-audit"
        }
        if ($BumpSkipProcessCleanup) {
            $args += "--skip-process-cleanup"
        }
        if ($BumpSkipWorkspaceGuard) {
            $args += "--skip-workspace-guard"
        }
        Invoke-GoRusktask -Arguments $args
    }
    "smart-rollback" {
        Write-Header "Smart release rollback"
        if ([string]::IsNullOrWhiteSpace($RollbackTag)) {
            throw "RollbackTag is required for smart-rollback target"
        }

        $args = @("smart-rollback", "--tag", $RollbackTag, "--owner", $RollbackOwner, "--repo", $RollbackRepo)
        if ($RollbackDeleteRelease) {
            $args += "--delete-release"
        }
        if ($RollbackDeleteRemoteTag) {
            $args += "--delete-remote-tag"
        }
        if ($RollbackDeleteLocalTag) {
            $args += "--delete-local-tag"
        }
        if ($RollbackRevertLastCommit) {
            $args += "--revert-last-commit"
        }
        if ($RollbackPushRevert) {
            $args += "--push-revert"
        }
        if ($RollbackNoVerify) {
            $args += "--no-verify"
        }
        if ($RollbackSkipProcessCleanup) {
            $args += "--skip-process-cleanup"
        }
        if ($RollbackSkipWorkspaceGuard) {
            $args += "--skip-workspace-guard"
        }
        if ($RollbackSkipIndexRefresh) {
            $args += "--skip-index-refresh"
        }

        Invoke-GoRusktask -Arguments $args
    }
    "pr-helper" {
        Write-Header "Pull request helper"
        $args = @("pr-helper", "--base", $PrBase)

        if ($PrCreate) {
            $args += "--create"
        }
        if ($PrCheck) {
            $args += "--check"
        }
        if ($PrMerge) {
            $args += "--merge"
        }
        if ($PrDraft) {
            $args += "--draft"
        }
        if ($PrAutoFill) {
            $args += "--auto-fill"
        }
        if (-not [string]::IsNullOrWhiteSpace($PrTitle)) {
            $args += @("--title", $PrTitle)
        }
        if (-not [string]::IsNullOrWhiteSpace($PrBody)) {
            $args += @("--body", $PrBody)
        }
        if (-not [string]::IsNullOrWhiteSpace($PrHead)) {
            $args += @("--head", $PrHead)
        }

        Invoke-GoRusktask -Arguments $args
    }
    "git-check" {
        Write-Header "Git workflow check"
        Invoke-GoRusktask -Arguments @("git-check")
    }
    "git-review-before-push" {
        Write-Header "Git review before push"
        Invoke-GoRusktask -Arguments @("review", "--before-push")
    }
    "git-hooks-install" {
        Write-Header "Install Git hooks"
        Invoke-GoRusktask -Arguments @("install-hooks")
    }
    "git-hooks-uninstall" {
        Write-Header "Uninstall Git hooks"
        Invoke-GoRusktask -Arguments @("install-hooks", "--uninstall")
    }
    "git-pr-check" {
        Write-Header "PR readiness check"
        $args = @("pr-helper", "--check", "--base", $PrBase)
        if (-not [string]::IsNullOrWhiteSpace($PrHead)) {
            $args += @("--head", $PrHead)
        }
        Invoke-GoRusktask -Arguments $args
    }
    "git-pr-create" {
        Write-Header "Create pull request"
        $args = @("pr-helper", "--create", "--base", $PrBase)
        if ($PrDraft) {
            $args += "--draft"
        }
        if ($PrAutoFill) {
            $args += "--auto-fill"
        }
        if (-not [string]::IsNullOrWhiteSpace($PrTitle)) {
            $args += @("--title", $PrTitle)
        }
        if (-not [string]::IsNullOrWhiteSpace($PrBody)) {
            $args += @("--body", $PrBody)
        }
        if (-not [string]::IsNullOrWhiteSpace($PrHead)) {
            $args += @("--head", $PrHead)
        }
        Invoke-GoRusktask -Arguments $args
    }
    "git-pr-merge" {
        Write-Header "Merge pull request"
        $args = @("pr-helper", "--merge", "--base", $PrBase)
        if (-not [string]::IsNullOrWhiteSpace($PrHead)) {
            $args += @("--head", $PrHead)
        }
        Invoke-GoRusktask -Arguments $args
    }
    "docs-bundle" {
        Write-Header "Build docs bundle"
        $args = @("docs-bundle")
        if (-not [string]::IsNullOrWhiteSpace($DocsOutputRoot)) {
            $args += @("--output-root", $DocsOutputRoot)
        }
        if ($DocsCreateZip) {
            $args += "--create-zip"
        }
        Invoke-GoRusktask -Arguments $args
    }
    "release-publish" {
        Write-Header "Publish GitHub release"
        if ([string]::IsNullOrWhiteSpace($ReleaseTag)) {
            throw "ReleaseTag is required for release-publish target"
        }

        $args = @("release-publish", "--owner", $ReleaseOwner, "--repo", $ReleaseRepo, "--tag", $ReleaseTag)
        if (-not [string]::IsNullOrWhiteSpace($ReleaseName)) {
            $args += @("--release-name", $ReleaseName)
        }
        if (-not [string]::IsNullOrWhiteSpace($ReleaseBodyFile)) {
            $args += @("--body-file", $ReleaseBodyFile)
        }
        foreach ($assetPath in $ReleaseAsset) {
            if (-not [string]::IsNullOrWhiteSpace($assetPath)) {
                $args += @("--asset", $assetPath)
            }
        }
        if ($ReleasePrerelease) {
            $args += "--prerelease"
        }
        if ($ReleaseDraft) {
            $args += "--draft"
        }
        if ($ReleasePruneExtraAssets) {
            $args += "--prune-extra-assets"
        }

        Invoke-GoRusktask -Arguments $args
    }
    "build-release-slim" {
        Write-Header "Build slim release"
        Invoke-GoRusktask -Arguments @("build-release-slim")
    }
    "package-windows-installer" {
        Write-Header "Package Windows installer"
        $args = @("package-windows-installer")
        if (-not [string]::IsNullOrWhiteSpace($PackageVersion)) {
            $args += @("--version", $PackageVersion)
        }
        if (-not [string]::IsNullOrWhiteSpace($PackageBuildTag)) {
            $args += @("--build-tag", $PackageBuildTag)
        }
        if ($PackageSkipBuild) {
            $args += "--skip-build"
        }
        Invoke-GoRusktask -Arguments $args
    }
    "package-windows-assets" {
        Write-Header "Package Windows portable assets"
        $args = @("package-windows-assets")
        if (-not [string]::IsNullOrWhiteSpace($PackageVersion)) {
            $args += @("--version", $PackageVersion)
        }
        if (-not [string]::IsNullOrWhiteSpace($PackageOutputDir)) {
            $args += @("--output-dir", $PackageOutputDir)
        }
        if ($PackageSkipBuild) {
            $args += "--skip-build"
        }
        Invoke-GoRusktask -Arguments $args
    }
    "package-windows-portable-installer" {
        Write-Header "Package Windows portable installer bundle"
        $args = @("package-windows-portable-installer")
        if (-not [string]::IsNullOrWhiteSpace($PackageVersion)) {
            $args += @("--version", $PackageVersion)
        }
        if (-not [string]::IsNullOrWhiteSpace($PackageOutputDir)) {
            $args += @("--output-dir", $PackageOutputDir)
        }
        if ($PackageSkipBuild) {
            $args += "--skip-build"
        }
        Invoke-GoRusktask -Arguments $args
    }
    "release-local-windows" {
        Write-Header "Local Windows release pipeline"
        Invoke-MakeSubTarget "ci-local"

        $installerArgs = @("package-windows-installer")
        if (-not [string]::IsNullOrWhiteSpace($PackageVersion)) {
            $installerArgs += @("--version", $PackageVersion)
        }
        if (-not [string]::IsNullOrWhiteSpace($PackageBuildTag)) {
            $installerArgs += @("--build-tag", $PackageBuildTag)
        }
        if ($PackageSkipBuild) {
            $installerArgs += "--skip-build"
        }
        Invoke-GoRusktask -Arguments $installerArgs

        $assetsArgs = @("package-windows-assets")
        if (-not [string]::IsNullOrWhiteSpace($PackageVersion)) {
            $assetsArgs += @("--version", $PackageVersion)
        }
        if (-not [string]::IsNullOrWhiteSpace($PackageOutputDir)) {
            $assetsArgs += @("--output-dir", $PackageOutputDir)
        }
        if ($PackageSkipBuild) {
            $assetsArgs += "--skip-build"
        }
        Invoke-GoRusktask -Arguments $assetsArgs

        Write-Host "`nLocal Windows release pipeline passed." -ForegroundColor Green
    }
    "release-local-linux" {
        Write-Header "Local Linux release pipeline"
        Invoke-MakeSubTarget "ci-local"
        Invoke-LinuxDebPackage
        Write-Host "`nLocal Linux release pipeline passed." -ForegroundColor Green
    }
    "release-local-dual" {
        Write-Header "Local dual-system release pipeline"
        Invoke-MakeSubTarget "ci-local"

        $installerArgs = @("package-windows-installer")
        if (-not [string]::IsNullOrWhiteSpace($PackageVersion)) {
            $installerArgs += @("--version", $PackageVersion)
        }
        if (-not [string]::IsNullOrWhiteSpace($PackageBuildTag)) {
            $installerArgs += @("--build-tag", $PackageBuildTag)
        }
        if ($PackageSkipBuild) {
            $installerArgs += "--skip-build"
        }
        Invoke-GoRusktask -Arguments $installerArgs

        $assetsArgs = @("package-windows-assets")
        if (-not [string]::IsNullOrWhiteSpace($PackageVersion)) {
            $assetsArgs += @("--version", $PackageVersion)
        }
        if (-not [string]::IsNullOrWhiteSpace($PackageOutputDir)) {
            $assetsArgs += @("--output-dir", $PackageOutputDir)
        }
        if ($PackageSkipBuild) {
            $assetsArgs += "--skip-build"
        }
        Invoke-GoRusktask -Arguments $assetsArgs

        Invoke-LinuxDebPackage
        Write-Host "`nLocal dual-system release pipeline passed." -ForegroundColor Green
    }
    "go-check" {
        Write-Header "Go orchestrator check"
        Invoke-GoRusktask -Arguments @("check")
    }
    "go-preflight" {
        Write-Header "Go orchestrator preflight"
        Invoke-GoRusktask -Arguments @("preflight")
    }
    "go-build" {
        Write-Header "Go orchestrator build"
        Invoke-GoRusktask -Arguments @("build")
    }
    "go-test" {
        Write-Header "Go orchestrator test"
        Invoke-GoRusktask -Arguments @("test")
    }
    "go-doc" {
        Write-Header "Go orchestrator doc"
        Invoke-GoRusktask -Arguments @("doc")
    }
    "go-audit" {
        Write-Header "Go orchestrator audit"
        Invoke-GoRusktask -Arguments @("audit")
    }
    "go-release-sync" {
        Write-Header "Go orchestrator release-sync"
        Invoke-GoRusktask -Arguments @("release-sync", "--mode", "audit")
    }
    "go-workflow-seal" {
        Write-Header "Go orchestrator workflow-seal"
        Invoke-GoRusktask -Arguments @("workflow-seal", "--mode", "audit")
    }
    "go-git-check" {
        Write-Header "Go orchestrator git-check"
        Invoke-GoRusktask -Arguments @("git-check")
    }
    "go-rust-review" {
        Write-Header "Go orchestrator rust-review"
        Invoke-GoRusktask -Arguments @("rust-review")
    }
    "go-review" {
        Write-Header "Go orchestrator review"
        Invoke-GoRusktask -Arguments @("review")
    }
    "go-install-hooks" {
        Write-Header "Go orchestrator install-hooks"
        Invoke-GoRusktask -Arguments @("install-hooks")
    }
    "help" {
        Write-Host @"

  rust_serial unified task runner
  =======================================

    Usage: .\scripts\task.ps1 <target> [-ReleaseNotesFile <path>] [-ReleaseNotesMode <draft|release>] [-PackageVersion <X.Y.Z>] [-PackageBuildTag <yyyymmdd>] [-PackageOutputDir <path>] [-PackageSkipBuild]

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
    ci-local     local CI core pipeline (governance + fmt/clippy/test/doc)
    ci-local-full local CI full pipeline (ci-local + audit)
    release-sync audit release tags/notes/archive consistency
    release-sync-apply normalize local release tags/notes state
    release-index rebuild release_notes/RELEASE_INDEX.md
    workflow-seal run cleanup + structure + release-state seal (audit)
    workflow-seal-apply normalize release-state and reseal workspace
    workspace-guard enforce workspace layout and path policy
    workspace-cleanup remove transient process files
    release-notes-validate validate release notes using Go rusktask
    smart-bump bump semantic version and create tag via Go
    smart-rollback rollback release tag/release/commit via Go
    pr-helper check/create/merge pull request via Go
    git-check   run Git workflow validation
    git-review-before-push run full review pipeline before push
    git-hooks-install install managed Git hooks
    git-hooks-uninstall uninstall managed Git hooks
    git-pr-check PR readiness check
    git-pr-create create pull request
    git-pr-merge merge pull request
    docs-bundle build docs/help bundle via Go
    release-publish create/update GitHub release via Go (requires GITHUB_TOKEN)
    build-release-slim build robot_control_rust slim release target
    package-windows-installer package installer via Go (Inno/iExpress fallback)
    package-windows-assets package portable zips via Go
    package-windows-portable-installer package portable installer bundle via Go
    release-local-windows local Windows release pipeline
    release-local-linux local Linux release pipeline (requires bash + dpkg-deb + mdbook)
    release-local-dual local dual-system release pipeline
    go-check    run check via Go orchestrator
    go-preflight run preflight via Go orchestrator
    go-build    run build via Go orchestrator
    go-test     run test via Go orchestrator
    go-doc      run doc via Go orchestrator
    go-audit    run audit via Go orchestrator
    go-release-sync run release sync via Go orchestrator
    go-workflow-seal run workflow seal via Go orchestrator
    go-git-check run git workflow check via Go orchestrator
    go-rust-review run Rust review pipeline via Go orchestrator
    go-review run combined review pipeline via Go orchestrator
    go-install-hooks install managed Git hooks via Go orchestrator
    clean        clean all target directories
    help         show this help

"@ -ForegroundColor Cyan
    }
}





