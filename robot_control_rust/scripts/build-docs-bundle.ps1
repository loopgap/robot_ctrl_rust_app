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
$helpIndexPath = Join-Path $OutputRoot 'help_index.html'
$docsZipPath = Join-Path $OutputRoot 'docs_bundle.zip'

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

& $mdbook.Source build $docsRoot -d $docsOutput
if ($LASTEXITCODE -ne 0) {
    throw "mdbook build failed (exit=$LASTEXITCODE)"
}

$redirectHtml = @"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta http-equiv="refresh" content="0; url=docs/index.html">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Robot Control Suite Help</title>
</head>
<body>
  <p>Redirecting to bundled documentation...</p>
  <p><a href="docs/index.html">Open local documentation</a></p>
</body>
</html>
"@

$redirectHtml | Set-Content -Path $helpIndexPath -Encoding UTF8

if ($CreateZip) {
    Compress-Archive -Path (Join-Path $docsOutput '*') -DestinationPath $docsZipPath -CompressionLevel Optimal
}
