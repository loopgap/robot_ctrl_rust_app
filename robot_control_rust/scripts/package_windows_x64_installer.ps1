param(
    [string]$Version,
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $PSScriptRoot
$repoRoot = Split-Path -Parent $projectRoot
$manifestPath = Join-Path $projectRoot 'Cargo.toml'
$toolSuiteManifestPath = Join-Path $repoRoot 'rust_tools_suite\Cargo.toml'
$releaseExe = Join-Path $projectRoot 'target\release\robot_control_rust.exe'
$toolSuiteReleaseExe = Join-Path $repoRoot 'rust_tools_suite\target\release\rust_tools_suite.exe'
$helpHtml = Join-Path $repoRoot 'docs\help\index.html'
$issPath = Join-Path $projectRoot 'installer\robot_control_rust_x64.iss'

$distRoot = Join-Path $projectRoot 'dist\windows-x64'
$stageDir = Join-Path $distRoot 'stage'
$outputDir = Join-Path $distRoot 'installer'

if (-not (Test-Path $manifestPath)) {
    throw "Cargo.toml not found: $manifestPath"
}

if ([string]::IsNullOrWhiteSpace($Version)) {
    $cargoToml = Get-Content $manifestPath -Raw
    $m = [regex]::Match($cargoToml, '(?m)^version\s*=\s*"([^"]+)"')
    if (-not $m.Success) {
        throw 'Failed to read version from Cargo.toml'
    }
    $Version = $m.Groups[1].Value
}

Write-Host "[Package] Version: $Version" -ForegroundColor Cyan

if (-not $SkipBuild) {
    Write-Host '[Package] Building release binaries...' -ForegroundColor Yellow
    cargo build --release --manifest-path $manifestPath
    if ($LASTEXITCODE -ne 0) {
        throw "robot_control_rust release build failed (exit=$LASTEXITCODE)"
    }
    cargo build --release --manifest-path $toolSuiteManifestPath
    if ($LASTEXITCODE -ne 0) {
        throw "rust_tools_suite release build failed (exit=$LASTEXITCODE)"
    }
}

foreach ($required in @($releaseExe, $toolSuiteReleaseExe, $helpHtml, $issPath)) {
    if (-not (Test-Path $required)) {
        throw "Required file not found: $required"
    }
}

if (Test-Path $stageDir) {
    Remove-Item $stageDir -Recurse -Force -ErrorAction SilentlyContinue
}
New-Item -ItemType Directory -Force -Path $stageDir | Out-Null
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

Copy-Item -Force $releaseExe (Join-Path $stageDir 'robot_control_rust.exe')
Copy-Item -Force $toolSuiteReleaseExe (Join-Path $stageDir 'rust_tools_suite.exe')
Copy-Item -Force $helpHtml (Join-Path $stageDir 'help_index.html')
Copy-Item -Force (Join-Path $projectRoot 'ARCHITECTURE_AND_USAGE.md') (Join-Path $stageDir 'ARCHITECTURE_AND_USAGE.md')

$isccCandidates = @(
    "$env:ProgramFiles(x86)\Inno Setup 6\ISCC.exe",
    "$env:ProgramFiles\Inno Setup 6\ISCC.exe",
    "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
    "$env:LOCALAPPDATA\Programs\JRSoftware\Inno Setup 6\ISCC.exe"
)

$iscc = $isccCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $iscc) {
    $regKeys = @(
        'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*',
        'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*',
        'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*'
    )
    $entry = Get-ItemProperty $regKeys -ErrorAction SilentlyContinue |
        Where-Object { $_.DisplayName -like 'Inno Setup*' -and $_.InstallLocation } |
        Select-Object -First 1
    if ($entry) {
        $candidate = Join-Path $entry.InstallLocation 'ISCC.exe'
        if (Test-Path $candidate) {
            $iscc = $candidate
        }
    }
}

if (-not $iscc) {
    Write-Host '[Package] Inno Setup not found. Falling back to iExpress...' -ForegroundColor Yellow
    & (Join-Path $PSScriptRoot 'package_windows_x64_iexpress_installer.ps1') -Version $Version -SkipBuild:$SkipBuild
    if ($LASTEXITCODE -ne 0) {
        throw "iExpress fallback failed (exit=$LASTEXITCODE)"
    }
    return
}

Write-Host "[Package] Using ISCC: $iscc" -ForegroundColor Green

& $iscc `
    "/DAppVersion=$Version" `
    "/DProjectRoot=$projectRoot" `
    "/DStageDir=$stageDir" `
    "/DOutputDir=$outputDir" `
    $issPath

if ($LASTEXITCODE -ne 0) {
    throw "ISCC failed (exit=$LASTEXITCODE)"
}

$installer = Get-ChildItem $outputDir -Filter "*${Version}*_x64_Setup.exe" -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1

if (-not $installer) {
    throw "Installer not found in $outputDir"
}

Write-Host '[Package] Success' -ForegroundColor Green
Write-Host "[Package] Installer: $($installer.FullName)" -ForegroundColor Green
Write-Host "[Package] Size MB: $([math]::Round($installer.Length / 1MB, 2))" -ForegroundColor Green

