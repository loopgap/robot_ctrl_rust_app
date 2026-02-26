param(
    [string]$Version,
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $projectRoot 'Cargo.toml'
$releaseExe = Join-Path $projectRoot 'target\release\robot_control_rust.exe'

$distRoot = Join-Path $projectRoot 'dist\windows-x64'
$stageDir = Join-Path $distRoot 'stage'
$outputDir = Join-Path $distRoot 'installer'
 $bundleDir = Join-Path $distRoot 'installer-bundle'

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

Write-Host "[PortablePackage] Version: $Version" -ForegroundColor Cyan

$outputExe = Join-Path $outputDir ("RobotControlSuite_{0}_x64_Setup.exe" -f $Version)
$outputZip = Join-Path $outputDir ("RobotControlSuite_{0}_x64_InstallerBundle.zip" -f $Version)

if (-not $SkipBuild) {
    Write-Host "[PortablePackage] Building release binary..." -ForegroundColor Yellow
    cargo build --release --manifest-path $manifestPath
    if ($LASTEXITCODE -ne 0) {
        throw "Release build failed (exit=$LASTEXITCODE)"
    }
}

if (-not (Test-Path $releaseExe)) {
    throw "Release exe not found: $releaseExe"
}

New-Item -ItemType Directory -Force -Path $stageDir | Out-Null
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
New-Item -ItemType Directory -Force -Path $bundleDir | Out-Null

Copy-Item -Force $releaseExe (Join-Path $stageDir 'robot_control_rust.exe')
Copy-Item -Force (Join-Path $projectRoot 'ARCHITECTURE_AND_USAGE.md') (Join-Path $stageDir 'ARCHITECTURE_AND_USAGE.md')
$installCmd = Join-Path $bundleDir 'Install_RobotControlSuite_x64.cmd'
$uninstallCmd = Join-Path $bundleDir 'Uninstall_RobotControlSuite_x64.cmd'

if (Test-Path $bundleDir) {
    Get-ChildItem $bundleDir -Force | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
}

Copy-Item -Force (Join-Path $stageDir 'robot_control_rust.exe') (Join-Path $bundleDir 'robot_control_rust.exe')
Copy-Item -Force (Join-Path $stageDir 'ARCHITECTURE_AND_USAGE.md') (Join-Path $bundleDir 'ARCHITECTURE_AND_USAGE.md')

@'
@echo off
setlocal
set "TARGET=%LOCALAPPDATA%\Robot Control Suite"
if not exist "%TARGET%" mkdir "%TARGET%"
copy /Y "%~dp0robot_control_rust.exe" "%TARGET%\robot_control_rust.exe" >nul
copy /Y "%~dp0ARCHITECTURE_AND_USAGE.md" "%TARGET%\ARCHITECTURE_AND_USAGE.md" >nul

powershell -NoProfile -ExecutionPolicy Bypass -Command "$s=(New-Object -ComObject WScript.Shell); $lnk=$s.CreateShortcut([System.IO.Path]::Combine($env:USERPROFILE,'Desktop','Robot Control Suite.lnk')); $lnk.TargetPath=[System.IO.Path]::Combine($env:LOCALAPPDATA,'Robot Control Suite','robot_control_rust.exe'); $lnk.WorkingDirectory=[System.IO.Path]::Combine($env:LOCALAPPDATA,'Robot Control Suite'); $lnk.Save();"
powershell -NoProfile -ExecutionPolicy Bypass -Command "$sm=[Environment]::GetFolderPath('StartMenu'); $dir=Join-Path $sm 'Programs'; $s=(New-Object -ComObject WScript.Shell); $lnk=$s.CreateShortcut((Join-Path $dir 'Robot Control Suite.lnk')); $lnk.TargetPath=[System.IO.Path]::Combine($env:LOCALAPPDATA,'Robot Control Suite','robot_control_rust.exe'); $lnk.WorkingDirectory=[System.IO.Path]::Combine($env:LOCALAPPDATA,'Robot Control Suite'); $lnk.Save();"

echo Installed to: %TARGET%
start "" "%TARGET%\robot_control_rust.exe"
exit /b 0
'@ | Set-Content -Path $installCmd -Encoding ASCII

@'
@echo off
setlocal
set "TARGET=%LOCALAPPDATA%\Robot Control Suite"
del /F /Q "%TARGET%\robot_control_rust.exe" 2>nul
del /F /Q "%TARGET%\ARCHITECTURE_AND_USAGE.md" 2>nul
del /F /Q "%TARGET%\Uninstall_RobotControlSuite_x64.cmd" 2>nul
rmdir "%TARGET%" 2>nul
del /F /Q "%USERPROFILE%\Desktop\Robot Control Suite.lnk" 2>nul
del /F /Q "%APPDATA%\Microsoft\Windows\Start Menu\Programs\Robot Control Suite.lnk" 2>nul
echo Uninstall completed.
exit /b 0
'@ | Set-Content -Path $uninstallCmd -Encoding ASCII

if (Test-Path $outputZip) {
    Remove-Item $outputZip -Force
}

Compress-Archive -Path (Join-Path $bundleDir '*') -DestinationPath $outputZip -CompressionLevel Optimal

if (-not (Test-Path $outputZip)) {
    throw "Installer bundle not found: $outputZip"
}

$installer = Get-Item $outputZip

Write-Host "[PortablePackage] Success" -ForegroundColor Green
Write-Host "[PortablePackage] Installer: $($installer.FullName)" -ForegroundColor Green
Write-Host "[PortablePackage] Size MB: $([math]::Round($installer.Length / 1MB, 2))" -ForegroundColor Green
