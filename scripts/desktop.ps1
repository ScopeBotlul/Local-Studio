param([ValidateSet('dev','build','test','check')][string]$Action = 'dev')
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$localCargo = Join-Path $projectRoot '.tools\cargo'
$localRustup = Join-Path $projectRoot '.tools\rustup'
if (Test-Path -LiteralPath (Join-Path $localCargo 'bin\cargo.exe')) {
    $env:CARGO_HOME = $localCargo
    $env:RUSTUP_HOME = $localRustup
    $env:Path = (Join-Path $localCargo 'bin') + ';' + $env:Path
}
# cargo test does not launch from a Visual Studio developer shell.
# Expose the installed SDK resource compiler explicitly for tauri-winres.
if (-not (Get-Command rc.exe -ErrorAction SilentlyContinue)) {
    $sdkBin = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
    $sdk = Get-ChildItem -LiteralPath $sdkBin -Directory -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -match '^10\.0\.\d+\.0$' -and (Test-Path -LiteralPath (Join-Path $_.FullName 'x64\rc.exe')) } |
        Sort-Object { [version]$_.Name } -Descending | Select-Object -First 1
    if ($sdk) { $env:Path = (Join-Path $sdk.FullName 'x64') + ';' + $env:Path }
}
Set-Location -LiteralPath $projectRoot
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Rust MSVC toolchain missing. Install Rust or use scripts/bootstrap-rust.ps1.'
}
if ($Action -eq 'test' -or $Action -eq 'check') {
    & cargo $Action --manifest-path src-tauri/Cargo.toml
} elseif ($Action -eq 'build') {
    & npm.cmd run tauri -- build --no-bundle
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    & (Join-Path $PSScriptRoot 'stage-image-runtime.ps1') -Destination (Join-Path $projectRoot 'src-tauri\target\release\image-runtime')
    & (Join-Path $PSScriptRoot 'build-installer.ps1')
} else {
    & npm.cmd run tauri -- $Action
}
exit $LASTEXITCODE
