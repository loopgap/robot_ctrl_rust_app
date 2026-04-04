param(
    [string]$Version,
    [string]$BuildTag,
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $PSScriptRoot
$repoRoot = Split-Path -Parent $projectRoot
$manifestPath = Join-Path $projectRoot 'Cargo.toml'
$toolSuiteManifestPath = Join-Path $repoRoot 'rust_tools_suite\Cargo.toml'
$releaseExe = Join-Path $projectRoot 'target\release\robot_control_rust.exe'
$toolSuiteReleaseExe = Join-Path $repoRoot 'rust_tools_suite\target\release\rust_tools_suite.exe'
$docsBundleScript = Join-Path $PSScriptRoot 'build-docs-bundle.ps1'

$distRoot = Join-Path $projectRoot 'dist\windows-x64'
$stageDir = Join-Path $distRoot 'stage'
$outputDir = Join-Path $distRoot 'installer'
$tempDir = Join-Path $distRoot 'iexpress-tmp'

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

if ([string]::IsNullOrWhiteSpace($BuildTag)) {
    $BuildTag = Get-Date -Format 'yyyyMMdd'
}

Write-Host "[IExpressPackage] Version: $Version" -ForegroundColor Cyan
Write-Host "[IExpressPackage] BuildTag: $BuildTag" -ForegroundColor Cyan

if (-not $SkipBuild) {
    Write-Host '[IExpressPackage] Building release binaries...' -ForegroundColor Yellow
    cargo build --release --manifest-path $manifestPath
    if ($LASTEXITCODE -ne 0) {
        throw "robot_control_rust release build failed (exit=$LASTEXITCODE)"
    }
    cargo build --release --manifest-path $toolSuiteManifestPath
    if ($LASTEXITCODE -ne 0) {
        throw "rust_tools_suite release build failed (exit=$LASTEXITCODE)"
    }
}

foreach ($required in @($releaseExe, $toolSuiteReleaseExe, $docsBundleScript)) {
    if (-not (Test-Path $required)) {
        throw "Required file not found: $required"
    }
}

$iexpressExe = Join-Path $env:WINDIR 'System32\iexpress.exe'
if (-not (Test-Path $iexpressExe)) {
    throw "IExpress not found: $iexpressExe"
}

if (Test-Path $stageDir) { Remove-Item $stageDir -Recurse -Force -ErrorAction SilentlyContinue }
if (Test-Path $tempDir) { Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue }
New-Item -ItemType Directory -Force -Path $stageDir | Out-Null
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

Copy-Item -Force $releaseExe (Join-Path $stageDir 'robot_control_rust.exe')
Copy-Item -Force $toolSuiteReleaseExe (Join-Path $stageDir 'rust_tools_suite.exe')
Copy-Item -Force (Join-Path $projectRoot 'ARCHITECTURE_AND_USAGE.md') (Join-Path $stageDir 'ARCHITECTURE_AND_USAGE.md')
& $docsBundleScript -OutputRoot $stageDir -CreateZip
if ($LASTEXITCODE -ne 0) {
    throw "Documentation bundle build failed (exit=$LASTEXITCODE)"
}

$installCmd = Join-Path $stageDir 'install.cmd'
@"
@echo off
setlocal
powershell -NoProfile -ExecutionPolicy Bypass -STA -File "%~dp0install.ps1"
if errorlevel 1 exit /b 1
exit /b 0
"@ | Set-Content -Path $installCmd -Encoding ASCII

$installPs1 = Join-Path $stageDir 'install.ps1'
@"
param(
    [ValidateSet('user','machine')]
    [string]__DOLLAR__Scope,
    [string]__DOLLAR__InstallDir,
    [switch]__DOLLAR__Elevated
)

__DOLLAR__ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms

function Test-IsAdmin {
    __DOLLAR__currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    __DOLLAR__principal = New-Object Security.Principal.WindowsPrincipal(__DOLLAR__currentUser)
    return __DOLLAR__principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Remove-ShortcutSafe([string]__DOLLAR__Path) {
    if (Test-Path __DOLLAR__Path) {
        try { Remove-Item -Path __DOLLAR__Path -Force -ErrorAction Stop } catch {}
    }
}

function Remove-OldInstall([string]__DOLLAR__DirPath) {
    if ([string]::IsNullOrWhiteSpace(__DOLLAR__DirPath)) { return }
    if (-not (Test-Path __DOLLAR__DirPath)) { return }
    foreach (__DOLLAR__Name in @('robot_control_rust.exe','rust_tools_suite.exe','help_index.html','docs_bundle.zip','ARCHITECTURE_AND_USAGE.md','install.ps1','install.cmd')) {
        Remove-Item (Join-Path __DOLLAR__DirPath __DOLLAR__Name) -Force -ErrorAction SilentlyContinue
    }
    Remove-Item (Join-Path __DOLLAR__DirPath 'docs') -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item __DOLLAR__DirPath -Recurse -Force -ErrorAction SilentlyContinue
}

if ([string]::IsNullOrWhiteSpace(__DOLLAR__Scope)) {
    __DOLLAR__choice = [System.Windows.Forms.MessageBox]::Show(
        'Select install scope:`nYes = All users (requires admin)`nNo = Current user',
        'Robot Control Suite - Install Scope',
        [System.Windows.Forms.MessageBoxButtons]::YesNoCancel,
        [System.Windows.Forms.MessageBoxIcon]::Question
    )
    if (__DOLLAR__choice -eq [System.Windows.Forms.DialogResult]::Cancel) { exit 1 }
    __DOLLAR__Scope = if (__DOLLAR__choice -eq [System.Windows.Forms.DialogResult]::Yes) { 'machine' } else { 'user' }
}

if (__DOLLAR__Scope -eq 'machine' -and -not (Test-IsAdmin)) {
    Start-Process powershell -Verb RunAs -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-STA','-File',__DOLLAR__PSCommandPath,'-Scope','machine','-Elevated') | Out-Null
    exit 0
}

if ([string]::IsNullOrWhiteSpace(__DOLLAR__InstallDir)) {
    __DOLLAR__InstallDir = if (__DOLLAR__Scope -eq 'machine') {
        Join-Path __DOLLAR__env:ProgramFiles 'Robot Control Suite'
    } else {
        Join-Path __DOLLAR__env:LOCALAPPDATA 'Robot Control Suite'
    }
}

Remove-ShortcutSafe (Join-Path ([Environment]::GetFolderPath('Desktop')) 'Robot Control Suite.lnk')
Remove-ShortcutSafe (Join-Path ([Environment]::GetFolderPath('Desktop')) 'Rust Tools Suite.lnk')
Remove-ShortcutSafe (Join-Path ([Environment]::GetFolderPath('Programs')) 'Robot Control Suite.lnk')
Remove-ShortcutSafe (Join-Path ([Environment]::GetFolderPath('Programs')) 'Rust Tools Suite.lnk')
Remove-ShortcutSafe (Join-Path ([Environment]::GetFolderPath('CommonDesktopDirectory')) 'Robot Control Suite.lnk')
Remove-ShortcutSafe (Join-Path ([Environment]::GetFolderPath('CommonDesktopDirectory')) 'Rust Tools Suite.lnk')
Remove-ShortcutSafe (Join-Path ([Environment]::GetFolderPath('CommonPrograms')) 'Robot Control Suite.lnk')
Remove-ShortcutSafe (Join-Path ([Environment]::GetFolderPath('CommonPrograms')) 'Rust Tools Suite.lnk')
Remove-OldInstall __DOLLAR__InstallDir

New-Item -ItemType Directory -Force -Path __DOLLAR__InstallDir | Out-Null
Copy-Item -Force (Join-Path __DOLLAR__PSScriptRoot 'robot_control_rust.exe') (Join-Path __DOLLAR__InstallDir 'robot_control_rust.exe')
Copy-Item -Force (Join-Path __DOLLAR__PSScriptRoot 'rust_tools_suite.exe') (Join-Path __DOLLAR__InstallDir 'rust_tools_suite.exe')
Copy-Item -Force (Join-Path __DOLLAR__PSScriptRoot 'help_index.html') (Join-Path __DOLLAR__InstallDir 'help_index.html')
Copy-Item -Force (Join-Path __DOLLAR__PSScriptRoot 'docs_bundle.zip') (Join-Path __DOLLAR__InstallDir 'docs_bundle.zip')
Copy-Item -Force (Join-Path __DOLLAR__PSScriptRoot 'ARCHITECTURE_AND_USAGE.md') (Join-Path __DOLLAR__InstallDir 'ARCHITECTURE_AND_USAGE.md')
Expand-Archive -LiteralPath (Join-Path __DOLLAR__InstallDir 'docs_bundle.zip') -DestinationPath (Join-Path __DOLLAR__InstallDir 'docs') -Force
Remove-Item (Join-Path __DOLLAR__InstallDir 'docs_bundle.zip') -Force -ErrorAction SilentlyContinue

__DOLLAR__desktopDir = if (__DOLLAR__Scope -eq 'machine') { [Environment]::GetFolderPath('CommonDesktopDirectory') } else { [Environment]::GetFolderPath('Desktop') }
__DOLLAR__programsDir = if (__DOLLAR__Scope -eq 'machine') { [Environment]::GetFolderPath('CommonPrograms') } else { [Environment]::GetFolderPath('Programs') }

__DOLLAR__shell = New-Object -ComObject WScript.Shell
__DOLLAR__mainExe = Join-Path __DOLLAR__InstallDir 'robot_control_rust.exe'
__DOLLAR__toolsExe = Join-Path __DOLLAR__InstallDir 'rust_tools_suite.exe'

__DOLLAR__desktopMain = __DOLLAR__shell.CreateShortcut((Join-Path __DOLLAR__desktopDir 'Robot Control Suite.lnk'))
__DOLLAR__desktopMain.TargetPath = __DOLLAR__mainExe
__DOLLAR__desktopMain.WorkingDirectory = __DOLLAR__InstallDir
__DOLLAR__desktopMain.IconLocation = "__DOLLAR__mainExe,0"
__DOLLAR__desktopMain.Save()

__DOLLAR__desktopTools = __DOLLAR__shell.CreateShortcut((Join-Path __DOLLAR__desktopDir 'Rust Tools Suite.lnk'))
__DOLLAR__desktopTools.TargetPath = __DOLLAR__toolsExe
__DOLLAR__desktopTools.WorkingDirectory = __DOLLAR__InstallDir
__DOLLAR__desktopTools.IconLocation = "__DOLLAR__toolsExe,0"
__DOLLAR__desktopTools.Save()

__DOLLAR__menuMain = __DOLLAR__shell.CreateShortcut((Join-Path __DOLLAR__programsDir 'Robot Control Suite.lnk'))
__DOLLAR__menuMain.TargetPath = __DOLLAR__mainExe
__DOLLAR__menuMain.WorkingDirectory = __DOLLAR__InstallDir
__DOLLAR__menuMain.IconLocation = "__DOLLAR__mainExe,0"
__DOLLAR__menuMain.Save()

__DOLLAR__menuTools = __DOLLAR__shell.CreateShortcut((Join-Path __DOLLAR__programsDir 'Rust Tools Suite.lnk'))
__DOLLAR__menuTools.TargetPath = __DOLLAR__toolsExe
__DOLLAR__menuTools.WorkingDirectory = __DOLLAR__InstallDir
__DOLLAR__menuTools.IconLocation = "__DOLLAR__toolsExe,0"
__DOLLAR__menuTools.Save()

Start-Process __DOLLAR__mainExe
"@.Replace('__DOLLAR__', '$') | Set-Content -Path $installPs1 -Encoding UTF8

$outputExe = Join-Path $outputDir ("RobotControlSuite_{0}_x64_{1}_Setup.exe" -f $Version, $BuildTag)
$sedPath = Join-Path $tempDir 'robot_control_suite.sed'

@"
[Version]
Class=IEXPRESS
SEDVersion=3
[Options]
PackagePurpose=InstallApp
ShowInstallProgramWindow=0
HideExtractAnimation=1
UseLongFileName=1
InsideCompressed=0
CAB_FixedSize=0
CAB_ResvCodeSigning=0
RebootMode=N
InstallPrompt=
DisplayLicense=
FinishMessage=
TargetName=$outputExe
FriendlyName=Robot Control Suite $Version
AppLaunched=install.cmd
PostInstallCmd=<None>
AdminQuietInstCmd=install.cmd
UserQuietInstCmd=install.cmd
SourceFiles=SourceFiles
[Strings]
FILE0=install.cmd
FILE1=robot_control_rust.exe
FILE2=rust_tools_suite.exe
FILE3=help_index.html
FILE4=docs_bundle.zip
FILE5=ARCHITECTURE_AND_USAGE.md
FILE6=install.ps1
[SourceFiles]
SourceFiles0=$stageDir
[SourceFiles0]
%FILE0%=
%FILE1%=
%FILE2%=
%FILE3%=
%FILE4%=
%FILE5%=
%FILE6%=
"@ | Set-Content -Path $sedPath -Encoding ASCII

if (Test-Path $outputExe) {
    Remove-Item $outputExe -Force -ErrorAction SilentlyContinue
}

& $iexpressExe /N $sedPath
if ($LASTEXITCODE -ne 0 -and -not (Test-Path $outputExe)) {
    throw "IExpress failed (exit=$LASTEXITCODE)"
}

if (-not (Test-Path $outputExe)) {
    throw "Installer not found: $outputExe"
}

Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item $stageDir -Recurse -Force -ErrorAction SilentlyContinue

$installer = Get-Item $outputExe
Write-Host '[IExpressPackage] Success' -ForegroundColor Green
Write-Host "[IExpressPackage] Installer: $($installer.FullName)" -ForegroundColor Green
Write-Host "[IExpressPackage] Size MB: $([math]::Round($installer.Length / 1MB, 2))" -ForegroundColor Green
