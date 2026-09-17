$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$toolsRoot = Join-Path $projectRoot '.tools'
New-Item -ItemType Directory -Path $toolsRoot -Force | Out-Null
$installer = Join-Path $toolsRoot 'rustup-init.exe'
$installerUri = 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe'
Invoke-WebRequest -UseBasicParsing -Uri $installerUri -OutFile $installer
$hashResponse = Invoke-WebRequest -UseBasicParsing -Uri ($installerUri + '.sha256')
$hashText = if ($hashResponse.Content -is [byte[]]) { [Text.Encoding]::UTF8.GetString($hashResponse.Content) } else { [string]$hashResponse.Content }
$expected = ($hashText.Trim() -split '\s+')[0]
$actual = (Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash
if ($actual -ine $expected) { throw 'Rust installer SHA256 mismatch.' }
$env:CARGO_HOME = Join-Path $toolsRoot 'cargo'
$env:RUSTUP_HOME = Join-Path $toolsRoot 'rustup'
& $installer -y --no-modify-path --profile minimal --default-host x86_64-pc-windows-msvc --default-toolchain stable
if ($LASTEXITCODE -ne 0) { throw 'Rust installation failed.' }
Write-Output 'Project-local Rust toolchain ready. Use npm run desktop:dev.'

