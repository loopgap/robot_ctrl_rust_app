param(
    [string]$Version,
    [string]$OutputDir,
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $PSScriptRoot
$repoRoot = Split-Path -Parent $projectRoot
$mainManifestPath = Join-Path $projectRoot 'Cargo.toml'
$suiteManifestPath = Join-Path $repoRoot 'rust_tools_suite\Cargo.toml'
$mainExe = Join-Path $projectRoot 'target\release\robot_control_rust.exe'
$suiteExe = Join-Path $repoRoot 'rust_tools_suite\target\release\rust_tools_suite.exe'
$docsBundleScript = Join-Path $PSScriptRoot 'build-docs-bundle.ps1'
$tempRoot = Join-Path $projectRoot 'dist\windows-x64\release-assets-tmp'
$docsRoot = Join-Path $tempRoot 'docs-root'
$mainBundleRoot = Join-Path $tempRoot 'robot_control_rust'
$suiteBundleRoot = Join-Path $tempRoot 'rust_tools_suite'

if ([string]::IsNullOrWhiteSpace($Version)) {
    $cargoToml = Get-Content $mainManifestPath -Raw
    $m = [regex]::Match($cargoToml, '(?m)^version\s*=\s*"([^"]+)"')
    if (-not $m.Success) {
        throw 'Failed to read version from Cargo.toml'
    }
    $Version = $m.Groups[1].Value
}

if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path $repoRoot 'release_artifacts'
}

if (-not $SkipBuild) {
    cargo build --release --manifest-path $mainManifestPath
    if ($LASTEXITCODE -ne 0) {
        throw "robot_control_rust release build failed (exit=$LASTEXITCODE)"
    }
    cargo build --release --manifest-path $suiteManifestPath
    if ($LASTEXITCODE -ne 0) {
        throw "rust_tools_suite release build failed (exit=$LASTEXITCODE)"
    }
}

foreach ($required in @($mainExe, $suiteExe, $docsBundleScript)) {
    if (-not (Test-Path $required)) {
        throw "Required file not found: $required"
    }
}

if (Test-Path $tempRoot) {
    Remove-Item $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
}
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
New-Item -ItemType Directory -Force -Path $docsRoot | Out-Null

& $docsBundleScript -OutputRoot $docsRoot
if ($LASTEXITCODE -ne 0) {
    throw "Documentation bundle build failed (exit=$LASTEXITCODE)"
}

$mainZip = Join-Path $OutputDir ("robot_control_rust_{0}_windows_x64_portable.zip" -f $Version)
$suiteZip = Join-Path $OutputDir ("rust_tools_suite_{0}_windows_x64_portable.zip" -f $Version)

foreach ($path in @($mainZip, $suiteZip)) {
    if (Test-Path $path) {
        Remove-Item $path -Force -ErrorAction SilentlyContinue
    }
}

New-Item -ItemType Directory -Force -Path $mainBundleRoot | Out-Null
New-Item -ItemType Directory -Force -Path $suiteBundleRoot | Out-Null

Copy-Item -Force $mainExe (Join-Path $mainBundleRoot 'robot_control_rust.exe')
Copy-Item -Force (Join-Path $projectRoot 'ARCHITECTURE_AND_USAGE.md') (Join-Path $mainBundleRoot 'ARCHITECTURE_AND_USAGE.md')
Copy-Item -Force (Join-Path $docsRoot 'help_index.html') (Join-Path $mainBundleRoot 'help_index.html')
Copy-Item -Recurse -Force (Join-Path $docsRoot 'docs') (Join-Path $mainBundleRoot 'docs')

Copy-Item -Force $suiteExe (Join-Path $suiteBundleRoot 'rust_tools_suite.exe')
Copy-Item -Force (Join-Path $repoRoot 'rust_tools_suite\README.md') (Join-Path $suiteBundleRoot 'README.md')
Copy-Item -Force (Join-Path $docsRoot 'help_index.html') (Join-Path $suiteBundleRoot 'help_index.html')
Copy-Item -Recurse -Force (Join-Path $docsRoot 'docs') (Join-Path $suiteBundleRoot 'docs')

Compress-Archive -Path (Join-Path $mainBundleRoot '*') -DestinationPath $mainZip -CompressionLevel Optimal
Compress-Archive -Path (Join-Path $suiteBundleRoot '*') -DestinationPath $suiteZip -CompressionLevel Optimal
