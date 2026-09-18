param([Parameter(Mandatory=$true)][string]$Destination)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$source = Join-Path $projectRoot '.tools\video-runtime'
$manifest = Get-Content -LiteralPath (Join-Path $projectRoot 'src-tauri\video-runtime.json') -Raw -Encoding UTF8 | ConvertFrom-Json
foreach ($file in $manifest.files.PSObject.Properties) {
    $path = Join-Path $source $file.Name
    if (!(Test-Path -LiteralPath $path) -or (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $file.Value) { throw 'Pinned video runtime missing or changed. Run scripts/bootstrap-video-runtime.mjs explicitly.' }
}
New-Item -ItemType Directory -Path $Destination -Force | Out-Null
foreach ($file in $manifest.files.PSObject.Properties) { Copy-Item -LiteralPath (Join-Path $source $file.Name) -Destination (Join-Path $Destination $file.Name) -Force }
Write-Output 'Verified video runtime staged.'
