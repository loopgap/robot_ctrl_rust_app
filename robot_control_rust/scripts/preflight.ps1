$ErrorActionPreference = 'Stop'

Write-Host "[Preflight] Start" -ForegroundColor Cyan

$steps = @(
    @{ Name = 'format'; Command = 'cargo fmt --check' },
    @{ Name = 'build_debug'; Command = 'cargo build' },
    @{ Name = 'test_debug'; Command = 'cargo test' },
    @{ Name = 'test_release'; Command = 'cargo test --release' },
    @{ Name = 'clippy'; Command = 'cargo clippy --all-targets -- -D warnings' },
    @{ Name = 'build_release'; Command = 'cargo build --release' }
)

$results = @()

foreach ($step in $steps) {
    Write-Host "[Preflight] Running $($step.Name): $($step.Command)" -ForegroundColor Yellow
    Invoke-Expression $step.Command
    $code = $LASTEXITCODE
    $results += [PSCustomObject]@{
        Step = $step.Name
        ExitCode = $code
    }
    if ($code -ne 0) {
        Write-Host "[Preflight] Failed at $($step.Name) (exit=$code)" -ForegroundColor Red
        $results | Format-Table -AutoSize
        exit $code
    }
}

$releaseBin = "target/release/robot_control_rust.exe"
if (-not (Test-Path $releaseBin)) {
    $releaseBin = "target/release/robot_control_rust"
}

if (Test-Path $releaseBin) {
    $size = (Get-Item $releaseBin).Length
    Write-Host "[Preflight] Release binary: $releaseBin ($size bytes)" -ForegroundColor Green
}

Write-Host "[Preflight] All checks passed" -ForegroundColor Green
$results | Format-Table -AutoSize
