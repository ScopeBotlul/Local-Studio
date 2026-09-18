param([Parameter(Mandatory=$true)][string]$Destination)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$source = Join-Path $projectRoot '.tools\image-runtime'
$manifest = Get-Content -LiteralPath (Join-Path $projectRoot 'src-tauri\image-runtime.json') -Raw -Encoding UTF8 | ConvertFrom-Json
foreach ($file in $manifest.files.PSObject.Properties) {
    $path = Join-Path $source $file.Name
    if (!(Test-Path -LiteralPath $path) -or (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $file.Value) {
        throw 'Pinned image runtime missing or changed. Run python scripts/bootstrap-image-runtime.py explicitly.'
    }
}
if (@(Get-ChildItem -LiteralPath $source).Count -ne @($manifest.files.PSObject.Properties).Count) { throw 'Unexpected image runtime files.' }
New-Item -ItemType Directory -Path $Destination -Force | Out-Null
foreach ($file in $manifest.files.PSObject.Properties) { Copy-Item -LiteralPath (Join-Path $source $file.Name) -Destination (Join-Path $Destination $file.Name) -Force }
Write-Output 'Verified image runtime staged.'
