param(
    [string]$Version,
    [string]$BuildTag,
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $projectRoot 'Cargo.toml'
$releaseExe = Join-Path $projectRoot 'target\release\robot_control_rust.exe'

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

Write-Host "[IExpressPackage] Version: $Version" -ForegroundColor Cyan
if ([string]::IsNullOrWhiteSpace($BuildTag)) {
    $BuildTag = Get-Date -Format 'yyyyMMdd'
}
Write-Host "[IExpressPackage] BuildTag: $BuildTag" -ForegroundColor Cyan

if (-not $SkipBuild) {
    Write-Host "[IExpressPackage] Building release binary..." -ForegroundColor Yellow
    cargo build --release --manifest-path $manifestPath
    if ($LASTEXITCODE -ne 0) {
        throw "Release build failed (exit=$LASTEXITCODE)"
    }
}

if (-not (Test-Path $releaseExe)) {
    throw "Release exe not found: $releaseExe"
}

$iexpressExe = Join-Path $env:WINDIR 'System32\iexpress.exe'
if (-not (Test-Path $iexpressExe)) {
    throw "IExpress not found: $iexpressExe"
}

New-Item -ItemType Directory -Force -Path $stageDir | Out-Null
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
if (Test-Path $tempDir) {
    Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
}
New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

Copy-Item -Force $releaseExe (Join-Path $stageDir 'robot_control_rust.exe')
Copy-Item -Force (Join-Path $projectRoot 'ARCHITECTURE_AND_USAGE.md') (Join-Path $stageDir 'ARCHITECTURE_AND_USAGE.md')

$installCmd = Join-Path $stageDir 'install.cmd'
@'
@echo off
setlocal
set "LOG=%TEMP%\RobotControlSuite_install.log"
echo ==== Robot Control Suite Installer ==== > "%LOG%"
echo TIME: %DATE% %TIME% >> "%LOG%"
echo SCRIPT_DIR: %~dp0 >> "%LOG%"
powershell -NoProfile -ExecutionPolicy Bypass -STA -File "%~dp0install.ps1" >> "%LOG%" 2>&1
if errorlevel 1 (
        powershell -NoProfile -ExecutionPolicy Bypass -Command "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.MessageBox]::Show('Installation failed. Check log: %TEMP%\\RobotControlSuite_install.log','Robot Control Suite',[System.Windows.Forms.MessageBoxButtons]::OK,[System.Windows.Forms.MessageBoxIcon]::Error)" >nul 2>&1
  pause
  exit /b 1
)
exit /b 0
'@ | Set-Content -Path $installCmd -Encoding ASCII

$installPs1 = Join-Path $stageDir 'install.ps1'
@'
param(
    [ValidateSet('user','machine')]
    [string]$Scope,
    [string]$InstallDir,
    [switch]$Elevated
)

$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

function Test-IsAdmin {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Get-RegisteredInstallDir {
    param([string]$RegPath)
    try {
        $item = Get-ItemProperty -Path $RegPath -ErrorAction Stop
        return $item.InstallDir
    } catch {
        return $null
    }
}

function Remove-ShortcutSafe {
    param([string]$Path)
    if (Test-Path $Path) {
        try { Remove-Item -Path $Path -Force -ErrorAction Stop } catch {}
    }
}

function Remove-OldInstall {
    param([string]$DirPath)
    if ([string]::IsNullOrWhiteSpace($DirPath)) { return }
    if (-not (Test-Path $DirPath)) { return }

    try {
        Remove-Item (Join-Path $DirPath 'robot_control_rust.exe') -Force -ErrorAction SilentlyContinue
        Remove-Item (Join-Path $DirPath 'ARCHITECTURE_AND_USAGE.md') -Force -ErrorAction SilentlyContinue
        Remove-Item (Join-Path $DirPath 'install.ps1') -Force -ErrorAction SilentlyContinue
        Remove-Item (Join-Path $DirPath 'install.cmd') -Force -ErrorAction SilentlyContinue
        Remove-Item $DirPath -Recurse -Force -ErrorAction SilentlyContinue
    } catch {}
}

if ([string]::IsNullOrWhiteSpace($Scope)) {
    $choice = [System.Windows.Forms.MessageBox]::Show(
        'Select install scope:`nYes = All users (requires admin)`nNo = Current user',
        'Robot Control Suite - Install Scope',
        [System.Windows.Forms.MessageBoxButtons]::YesNoCancel,
        [System.Windows.Forms.MessageBoxIcon]::Question
    )

    if ($choice -eq [System.Windows.Forms.DialogResult]::Cancel) {
        exit 1
    }
    $Scope = if ($choice -eq [System.Windows.Forms.DialogResult]::Yes) { 'machine' } else { 'user' }
}

$defaultDir = if ($Scope -eq 'machine') {
    Join-Path $env:ProgramFiles 'Robot Control Suite'
} else {
    Join-Path $env:LOCALAPPDATA 'Robot Control Suite'
}

if ([string]::IsNullOrWhiteSpace($InstallDir)) {
    $InstallDir = $defaultDir
}

if (-not $Elevated) {
    $folderDialog = New-Object System.Windows.Forms.FolderBrowserDialog
    $folderDialog.Description = 'Select install folder (Cancel to use default)'
    $folderDialog.SelectedPath = $InstallDir
    $folderResult = $folderDialog.ShowDialog()
    if ($folderResult -eq [System.Windows.Forms.DialogResult]::OK -and -not [string]::IsNullOrWhiteSpace($folderDialog.SelectedPath)) {
        $InstallDir = $folderDialog.SelectedPath
    }
}

if ($Scope -eq 'machine' -and -not (Test-IsAdmin)) {
    $args = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`" -Scope machine -InstallDir `"$InstallDir`" -Elevated"
    Start-Process -FilePath 'powershell.exe' -ArgumentList $args -Verb RunAs | Out-Null
    exit 0
}

$legacyUserDir = Join-Path $env:LOCALAPPDATA 'Robot Control Suite'
$legacyMachineDir = Join-Path $env:ProgramFiles 'Robot Control Suite'
$registeredUserDir = Get-RegisteredInstallDir -RegPath 'HKCU:\Software\Robot Control Suite'
$registeredMachineDir = Get-RegisteredInstallDir -RegPath 'HKLM:\Software\Robot Control Suite'

$oldCandidates = @($legacyUserDir, $legacyMachineDir, $registeredUserDir, $registeredMachineDir) |
    Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
    Select-Object -Unique

foreach ($oldDir in $oldCandidates) {
    try {
        $oldFull = [System.IO.Path]::GetFullPath($oldDir).TrimEnd('\\')
        $targetFull = [System.IO.Path]::GetFullPath($InstallDir).TrimEnd('\\')
        if ($oldFull -ieq $targetFull) { continue }
    } catch {}
    Remove-OldInstall -DirPath $oldDir
}

Remove-ShortcutSafe -Path (Join-Path ([Environment]::GetFolderPath('Desktop')) 'Robot Control Suite.lnk')
Remove-ShortcutSafe -Path (Join-Path ([Environment]::GetFolderPath('CommonDesktopDirectory')) 'Robot Control Suite.lnk')
Remove-ShortcutSafe -Path (Join-Path ([Environment]::GetFolderPath('Programs')) 'Robot Control Suite.lnk')
Remove-ShortcutSafe -Path (Join-Path ([Environment]::GetFolderPath('CommonPrograms')) 'Robot Control Suite.lnk')

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item -Force (Join-Path $PSScriptRoot 'robot_control_rust.exe') (Join-Path $InstallDir 'robot_control_rust.exe')
Copy-Item -Force (Join-Path $PSScriptRoot 'ARCHITECTURE_AND_USAGE.md') (Join-Path $InstallDir 'ARCHITECTURE_AND_USAGE.md')

$desktopDir = if ($Scope -eq 'machine') {
    [Environment]::GetFolderPath('CommonDesktopDirectory')
} else {
    [Environment]::GetFolderPath('Desktop')
}

$programsDir = if ($Scope -eq 'machine') {
    [Environment]::GetFolderPath('CommonPrograms')
} else {
    [Environment]::GetFolderPath('Programs')
}

$shell = New-Object -ComObject WScript.Shell
$targetExe = Join-Path $InstallDir 'robot_control_rust.exe'

$desktopLink = Join-Path $desktopDir 'Robot Control Suite.lnk'
$shortcutDesktop = $shell.CreateShortcut($desktopLink)
$shortcutDesktop.TargetPath = $targetExe
$shortcutDesktop.WorkingDirectory = $InstallDir
$shortcutDesktop.IconLocation = "$targetExe,0"
$shortcutDesktop.Save()

$menuLink = Join-Path $programsDir 'Robot Control Suite.lnk'
$shortcutMenu = $shell.CreateShortcut($menuLink)
$shortcutMenu.TargetPath = $targetExe
$shortcutMenu.WorkingDirectory = $InstallDir
$shortcutMenu.IconLocation = "$targetExe,0"
$shortcutMenu.Save()

if ($Scope -eq 'machine') {
    New-Item -Path 'HKLM:\Software\Robot Control Suite' -Force | Out-Null
    Set-ItemProperty -Path 'HKLM:\Software\Robot Control Suite' -Name 'InstallDir' -Value $InstallDir
} else {
    New-Item -Path 'HKCU:\Software\Robot Control Suite' -Force | Out-Null
    Set-ItemProperty -Path 'HKCU:\Software\Robot Control Suite' -Name 'InstallDir' -Value $InstallDir
}

$launchPrompt = [System.Windows.Forms.MessageBox]::Show(
    "Installation completed.`nScope: $Scope`nPath: $InstallDir`n`nLaunch now?",
    'Robot Control Suite',
    [System.Windows.Forms.MessageBoxButtons]::YesNo,
    [System.Windows.Forms.MessageBoxIcon]::Information
)

if ($launchPrompt -eq [System.Windows.Forms.DialogResult]::Yes) {
    Start-Process -FilePath $targetExe | Out-Null
}
'@ | Set-Content -Path $installPs1 -Encoding ASCII

$outputExe = Join-Path $outputDir ("RobotControlSuite_{0}_x64_{1}_Setup.exe" -f $Version, $BuildTag)
$outputCab = Join-Path $outputDir ("~RobotControlSuite_{0}_x64_{1}_Setup.CAB" -f $Version, $BuildTag)
$sedPath = Join-Path $tempDir 'package.sed'

if (Test-Path $outputExe) {
    Remove-Item $outputExe -Force -ErrorAction SilentlyContinue
}
if (Test-Path $outputCab) {
    Remove-Item $outputCab -Force -ErrorAction SilentlyContinue
}

$sed = @"
[Version]
Class=IEXPRESS
SEDVersion=3
[Options]
PackagePurpose=InstallApp
ShowInstallProgramWindow=1
HideExtractAnimation=1
UseLongFileName=1
InsideCompressed=0
CAB_FixedSize=0
CAB_ResvCodeSigning=0
RebootMode=N
InstallPrompt=
DisplayLicense=
FinishMessage=Installation complete.
TargetName=$outputExe
FriendlyName=Robot Control Suite $Version ($BuildTag)
AppLaunched=cmd /c install.cmd
PostInstallCmd=<None>
AdminQuietInstCmd=cmd /c install.cmd
UserQuietInstCmd=cmd /c install.cmd
SourceFiles=SourceFiles
[Strings]
FILE0=install.cmd
FILE1=robot_control_rust.exe
FILE2=ARCHITECTURE_AND_USAGE.md
FILE3=install.ps1
[SourceFiles]
SourceFiles0=$stageDir
[SourceFiles0]
%FILE0%=
%FILE1%=
%FILE2%=
%FILE3%=
"@

$sed | Set-Content -Path $sedPath -Encoding ASCII

Write-Host "[IExpressPackage] Using built-in tool: $iexpressExe" -ForegroundColor Green
$buildStart = Get-Date
try {
    $process = Start-Process -FilePath $iexpressExe -ArgumentList @('/N','/Q',$sedPath) -Wait -PassThru
    $hasOutput = Test-Path $outputExe
    $isFreshOutput = $false
    if ($hasOutput) {
        $outInfo = Get-Item $outputExe
        $isFreshOutput = $outInfo.LastWriteTime -ge $buildStart
    }

    if ($process.ExitCode -ne 0 -and -not $isFreshOutput) {
        throw "IExpress failed (exit=$($process.ExitCode))"
    }
    if ($process.ExitCode -ne 0 -and $isFreshOutput) {
        Write-Host "[IExpressPackage] Warning: IExpress returned exit=$($process.ExitCode), but installer was generated." -ForegroundColor Yellow
    }
    if (-not $hasOutput) {
        throw "Installer not found: $outputExe"
    }
}
finally {
    Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item $stageDir -Recurse -Force -ErrorAction SilentlyContinue
}

$installer = Get-Item $outputExe
Write-Host "[IExpressPackage] Success" -ForegroundColor Green
Write-Host "[IExpressPackage] Installer: $($installer.FullName)" -ForegroundColor Green
Write-Host "[IExpressPackage] Size MB: $([math]::Round($installer.Length / 1MB, 2))" -ForegroundColor Green
