param(
    [switch]$SkipTests,
    [switch]$SkipClippy
)

$ErrorActionPreference = 'Stop'

Write-Host "[ReleaseSlim] Start" -ForegroundColor Cyan

function Remove-PathWithRetry {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,
        [int]$Retries = 6,
        [int]$DelayMs = 800
    )
    if (-not (Test-Path $Path)) {
        return $true
    }
    for ($i = 1; $i -le $Retries; $i++) {
        try {
            Remove-Item $Path -Recurse -Force -ErrorAction Stop
            return $true
        } catch {
            if ($i -eq $Retries) {
                Write-Host "[ReleaseSlim] Failed to remove $Path after $Retries retries: $($_.Exception.Message)" -ForegroundColor Red
                return $false
            }
            Start-Sleep -Milliseconds $DelayMs
        }
    }
    return $false
}

$targetDir = Join-Path (Get-Location) 'target'
$beforeBytes = 0
if (Test-Path $targetDir) {
    $beforeBytes = (Get-ChildItem $targetDir -Recurse -Force -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum
}
Write-Host "[ReleaseSlim] Target size before: $beforeBytes bytes" -ForegroundColor Yellow

$steps = @(
    @{ Name = 'fmt'; Command = 'cargo fmt --check' }
)

if (-not $SkipClippy) {
    $steps += @{ Name = 'clippy'; Command = 'cargo clippy --all-targets -- -D warnings' }
}

if (-not $SkipTests) {
    $steps += @{ Name = 'test_debug'; Command = 'cargo test' }
    $steps += @{ Name = 'test_release'; Command = 'cargo test --release' }
}

$steps += @{ Name = 'prepare_release_dirs'; Command = 'echo prepare_release_dirs' }
$steps += @{ Name = 'build_release'; Command = 'cargo build --release' }

$results = @()
foreach ($step in $steps) {
    Write-Host "[ReleaseSlim] Running $($step.Name): $($step.Command)" -ForegroundColor Yellow
    if ($step.Name -eq 'prepare_release_dirs') {
        $cleanupTargets = @(
            'target/debug',
            'target/flycheck0',
            'target/release/deps',
            'target/release/build',
            'target/release/incremental',
            'target/release/examples'
        )
        $allOk = $true
        foreach ($cleanupPath in $cleanupTargets) {
            if (Test-Path $cleanupPath) {
                $ok = Remove-PathWithRetry -Path $cleanupPath
                if ($ok) {
                    Write-Host "[ReleaseSlim] Cleaned $cleanupPath" -ForegroundColor DarkGray
                } else {
                    Write-Host "[ReleaseSlim] Warning: skip locked path $cleanupPath" -ForegroundColor Yellow
                }
            }
        }
        $code = 0
    } else {
        Invoke-Expression $step.Command
        $code = $LASTEXITCODE
    }
    $results += [PSCustomObject]@{ Step = $step.Name; ExitCode = $code }
    if ($code -ne 0) {
        Write-Host "[ReleaseSlim] Failed at $($step.Name) (exit=$code)" -ForegroundColor Red
        $results | Format-Table -AutoSize
        exit $code
    }
}

$removePaths = @('target/debug', 'target/flycheck0', 'target/release/deps', 'target/release/build', 'target/release/incremental', 'target/release/examples')
foreach ($path in $removePaths) {
    if (Test-Path $path) {
        $ok = Remove-PathWithRetry -Path $path
        if ($ok) {
            Write-Host "[ReleaseSlim] Removed $path" -ForegroundColor DarkGray
        }
    }
}

$releaseBin = 'target/release/robot_control_rust.exe'
if (-not (Test-Path $releaseBin)) {
    $releaseBin = 'target/release/robot_control_rust'
}

$afterBytes = 0
if (Test-Path $targetDir) {
    $afterBytes = (Get-ChildItem $targetDir -Recurse -Force -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum
}

Write-Host "[ReleaseSlim] Target size after : $afterBytes bytes" -ForegroundColor Green

if ($beforeBytes -gt 0) {
    $delta = $beforeBytes - $afterBytes
    $ratio = [Math]::Round(($afterBytes / [double]$beforeBytes) * 100, 2)
    Write-Host "[ReleaseSlim] Reduced by $delta bytes, remaining $ratio%" -ForegroundColor Green
}

if (Test-Path $releaseBin) {
    $item = Get-Item $releaseBin
    Write-Host "[ReleaseSlim] Release binary: $($item.FullName)" -ForegroundColor Green
    Write-Host "[ReleaseSlim] Binary size  : $($item.Length) bytes" -ForegroundColor Green
    Write-Host "[ReleaseSlim] LastWriteTime: $($item.LastWriteTime)" -ForegroundColor Green
} else {
    Write-Host "[ReleaseSlim] Release binary not found" -ForegroundColor Red
    exit 1
}

Write-Host "[ReleaseSlim] Done" -ForegroundColor Cyan
$results | Format-Table -AutoSize
