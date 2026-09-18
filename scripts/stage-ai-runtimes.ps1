param([Parameter(Mandatory=$true)][string]$Destination)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
foreach ($kind in @('assistant','speech')) {
    $source = Join-Path $projectRoot ".tools\$kind-runtime"
    $manifest = Get-Content -LiteralPath (Join-Path $projectRoot "src-tauri\$kind-runtime.json") -Raw -Encoding UTF8 | ConvertFrom-Json
    foreach ($file in $manifest.files.PSObject.Properties) {
        $path = Join-Path $source $file.Name
        if (!(Test-Path -LiteralPath $path) -or (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $file.Value) { throw "Pinned $kind runtime missing or changed. Run scripts/bootstrap-ai-runtimes.mjs explicitly." }
    }
    $target = Join-Path $Destination "$kind-runtime"
    New-Item -ItemType Directory -Path $target -Force | Out-Null
    foreach ($file in $manifest.files.PSObject.Properties) { Copy-Item -LiteralPath (Join-Path $source $file.Name) -Destination (Join-Path $target $file.Name) -Force }
}
Write-Output 'Verified assistant and speech runtimes staged.'
