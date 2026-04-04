param(
    [Parameter(Mandatory = $true)]
    [string]$OutputRoot,
    [switch]$CreateZip
)

$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $PSScriptRoot
$repoRoot = Split-Path -Parent $projectRoot
$docsRoot = Join-Path $repoRoot 'docs'
$docsOutput = Join-Path $OutputRoot 'docs'
$bookOutput = Join-Path $docsOutput 'book'
$helpIndexPath = Join-Path $OutputRoot 'help_index.html'
$docsIndexPath = Join-Path $docsOutput 'index.html'
$docsZipPath = Join-Path $OutputRoot 'docs_bundle.zip'
$localHelpSource = Join-Path $docsRoot 'help\index.html'

if (-not (Test-Path (Join-Path $docsRoot 'book.toml'))) {
    throw "mdBook config not found: $(Join-Path $docsRoot 'book.toml')"
}

$mdbook = Get-Command mdbook -ErrorAction SilentlyContinue
if (-not $mdbook) {
    throw 'mdbook was not found in PATH. Install mdbook before packaging release assets.'
}

if (Test-Path $docsOutput) {
    Remove-Item $docsOutput -Recurse -Force -ErrorAction SilentlyContinue
}
if ($CreateZip -and (Test-Path $docsZipPath)) {
    Remove-Item $docsZipPath -Force -ErrorAction SilentlyContinue
}

New-Item -ItemType Directory -Force -Path $OutputRoot | Out-Null
New-Item -ItemType Directory -Force -Path $docsOutput | Out-Null

& $mdbook.Source build $docsRoot -d $bookOutput
if ($LASTEXITCODE -ne 0) {
    throw "mdbook build failed (exit=$LASTEXITCODE)"
}

if (-not (Test-Path $localHelpSource)) {
    throw "Local help source not found: $localHelpSource"
}

Copy-Item -Force $localHelpSource $helpIndexPath
Copy-Item -Force $localHelpSource $docsIndexPath

if ($CreateZip) {
    Compress-Archive -Path (Join-Path $docsOutput '*') -DestinationPath $docsZipPath -CompressionLevel Optimal
}
