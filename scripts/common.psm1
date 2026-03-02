# 颜色定义
$ESC = [char]27
$RED = "${ESC}[31m"
$GREEN = "${ESC}[32m"
$YELLOW = "${ESC}[33m"
$BLUE = "${ESC}[34m"
$MAGENTA = "${ESC}[35m"
$CYAN = "${ESC}[36m"
$RESET = "${ESC}[0m"
$BOLD = "${ESC}[1m"

# 图标定义
$ICON_CHECK = "✓"
$ICON_CROSS = "✗"
$ICON_WARN = "⚠"
$ICON_INFO = "ℹ"
$ICON_ROCKET = "🚀"
$ICON_GEAR = "⚙"

# 计数器
$script:ErrorCount = 0
$script:WarningCount = 0

function Write-Header {
    param([string]$Title)
    Write-Host ""
    Write-Host "${BOLD}${CYAN}══════════════════════════════════════════════════════════════${RESET}"
    Write-Host "${BOLD}${CYAN}  $Title${RESET}"
    Write-Host "${BOLD}${CYAN}══════════════════════════════════════════════════════════════${RESET}"
}

function Write-Step {
    param([string]$Message)
    Write-Host "${BLUE}${ICON_GEAR}${RESET} $Message"
}

function Write-Success {
    param([string]$Message)
    Write-Host "${GREEN}${ICON_CHECK}${RESET} $Message"
}

function Write-Error {
    param([string]$Message)
    $script:ErrorCount++
    Write-Host "${RED}${ICON_CROSS}${RESET} $Message"
}

function Write-Warning {
    param([string]$Message)
    $script:WarningCount++
    Write-Host "${YELLOW}${ICON_WARN}${RESET} $Message"
}

function Write-Info {
    param([string]$Message)
    Write-Host "${MAGENTA}${ICON_INFO}${RESET} $Message"
}

function Get-ProjectDirs {
    $dirs = @()
    if (Test-Path "robot_control_rust\Cargo.toml") { $dirs += "robot_control_rust" }
    if (Test-Path "rust_micro_tools\Cargo.toml") { $dirs += "rust_micro_tools" }
    if (Test-Path "rust_indie_tools") {
        Get-ChildItem "rust_indie_tools" -Directory | ForEach-Object {
            if (Test-Path "$($_.FullName)\Cargo.toml") { $dirs += "rust_indie_tools\$($_.Name)" }
        }
    }
    return $dirs
}

function Invoke-CommandWithOutput {
    param(
        [string]$Command,
        [string]$Arguments,
        [string]$WorkingDir = "."
    )
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $Command
    $psi.Arguments = $Arguments
    $psi.WorkingDirectory = $WorkingDir
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    
    $proc = New-Object System.Diagnostics.Process
    $proc.StartInfo = $psi
    $proc.Start() | Out-Null
    
    $stdout = $proc.StandardOutput.ReadToEnd()
    $stderr = $proc.StandardError.ReadToEnd()
    $proc.WaitForExit()
    
    return @{
        ExitCode = $proc.ExitCode
        StdOut = $stdout
        StdErr = $stderr
    }
}

Export-ModuleMember -Function *
