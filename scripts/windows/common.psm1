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

$script:ModuleDir = $PSScriptRoot

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
    param(
        [Parameter(Mandatory = $true)][string]$Command,
        [string]$Arguments = "",
        [string]$WorkingDir = "."
    )

    try {
        Push-Location $WorkingDir
        $global:LASTEXITCODE = 0

        $output = if ([string]::IsNullOrWhiteSpace($Arguments)) {
            & $Command 2>&1
        }
        else {
            & ([scriptblock]::Create("& $Command $Arguments")) 2>&1
        }

        $exitCode = if ($null -eq $LASTEXITCODE) {
            if ($?) { 0 } else { 1 }
        }
        else {
            [int]$LASTEXITCODE
        }

        $combined = @($output)
        $stdOut = ($combined |
            Where-Object { $_ -isnot [System.Management.Automation.ErrorRecord] } |
            ForEach-Object { $_.ToString() }) -join [Environment]::NewLine
        $stdErr = ($combined |
            Where-Object { $_ -is [System.Management.Automation.ErrorRecord] } |
            ForEach-Object { $_.ToString() }) -join [Environment]::NewLine

        return @{
            ExitCode = $exitCode
            Output   = $combined
            StdOut   = $stdOut
            StdErr   = $stdErr
        }
    }
    catch {
        return @{
            ExitCode = 1
            Output   = @($_.Exception.Message)
            StdOut   = ""
            StdErr   = $_.Exception.Message
        }
    }
    finally {
        Pop-Location
    }
}

function Get-ProjectDirs {
    $root = Split-Path -Parent $script:ModuleDir
    if (-not (Test-Path $root)) {
        return @()
    }

    Get-ChildItem -Path $root -Filter "Cargo.toml" -Recurse -Depth 1 |
        ForEach-Object { $_.DirectoryName } |
        Sort-Object -Unique
}

Export-ModuleMember -Function @("Write-Header", "Write-Step", "Write-Info", "Write-Success", "Write-Error", "Write-Warning", "Invoke-CommandWithOutput", "Get-ProjectDirs")
