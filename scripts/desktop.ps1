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
Set-Location -LiteralPath $projectRoot
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Rust MSVC toolchain missing. Install Rust or use scripts/bootstrap-rust.ps1.'
}
if ($Action -eq 'test' -or $Action -eq 'check') {
    & cargo $Action --manifest-path src-tauri/Cargo.toml
} else {
    & npm.cmd run tauri -- $Action
}
exit $LASTEXITCODE
