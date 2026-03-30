# Common PowerShell module for Git hooks
# Provides utility functions for consistent output formatting

$script:Colors = @{
    RESET = [char]27 + "[0m"
    RED = [char]27 + "[31m"
    GREEN = [char]27 + "[32m"
    YELLOW = [char]27 + "[33m"
    BLUE = [char]27 + "[34m"
    CYAN = [char]27 + "[36m"
    BOLD = [char]27 + "[1m"
}

function Write-Header {
    param([string]$Message)
    Write-Host ""
    Write-Host "${script:Colors.CYAN}${script:Colors.BOLD}=== $Message ===${script:Colors.RESET}"
    Write-Host ""
}

function Write-Step {
    param([string]$Message)
    Write-Host "${script:Colors.BLUE} $Message"
}

function Write-Info {
    param([string]$Message)
    Write-Host "${script:Colors.YELLOW}i${script:Colors.RESET} $Message"
}

function Write-Success {
    param([string]$Message)
    Write-Host "${script:Colors.GREEN}V${script:Colors.RESET} $Message"
}

function Write-Error {
    param([string]$Message)
    Write-Host "${script:Colors.RED}X${script:Colors.RESET} $Message"
}

function Write-Warning {
    param([string]$Message)
    Write-Host "${script:Colors.YELLOW}!${script:Colors.RESET} $Message"
}

function Invoke-CommandWithOutput {
    param([string]$Command, [string]$Arguments, [string]$WorkingDir = ".")
    try {
        $output = & $Command $Arguments 2>&1
        return @{ ExitCode = $LASTEXITCODE; Output = $output }
    }
    catch {
        return @{ ExitCode = 1; Output = $_.Exception.Message }
    }
}

function Get-ProjectDirs {
    $root = Split-Path (Split-Path (Split-Path -Parent $MyInvocation.MyCommand.Path) -Parent) -Parent
    Get-ChildItem $root -Filter "Cargo.toml" -Recurse -Depth 1 | ForEach-Object { $_.DirectoryName }
}

Export-ModuleMember -Function @("Write-Header", "Write-Step", "Write-Info", "Write-Success", "Write-Error", "Write-Warning", "Invoke-CommandWithOutput", "Get-ProjectDirs")
