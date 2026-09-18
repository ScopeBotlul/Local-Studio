param([string]$CompilerPath = '', [string]$TestBuild = '', [ValidateSet('dynamic','light','dark')][string]$TestTheme = 'dynamic', [switch]$TestLegacy)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
if (!$CompilerPath) { $CompilerPath = Join-Path $projectRoot '.tools\inno-setup\ISCC.exe' }
if (!(Test-Path -LiteralPath $CompilerPath)) { throw 'Inno Setup 7.1 compiler missing. Run scripts/bootstrap-installer.ps1 or pass -CompilerPath.' }
$version = (Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $projectRoot 'src-tauri\tauri.conf.json') | ConvertFrom-Json).version
if ($version -notmatch '^\d+\.\d+\.\d+$') { throw 'Invalid application version.' }
$binary = Join-Path $projectRoot 'src-tauri\target\release\local-studio.exe'
if (!(Test-Path -LiteralPath $binary)) { throw 'Build the Tauri release executable first.' }
if ((Get-Item -LiteralPath $binary).VersionInfo.ProductVersion -ne $version) { throw 'App version does not match installer version. Rebuild the Tauri executable.' }
$webviewDir = Join-Path $projectRoot '.tools\webview2'
New-Item -ItemType Directory -Force -Path $webviewDir | Out-Null
$bootstrapper = Join-Path $webviewDir 'MicrosoftEdgeWebview2Setup.exe'
if (!(Test-Path -LiteralPath $bootstrapper)) {
    Invoke-WebRequest -UseBasicParsing -Uri 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile $bootstrapper
}
$signature = Get-AuthenticodeSignature -LiteralPath $bootstrapper
if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation(?:,|$)') { throw 'WebView2 bootstrapper has no valid Microsoft signature.' }
$compilerArgs = @('--quiet', "--define=AppVersion=$version", "--define=ProjectRoot=$projectRoot")
if ($TestBuild) {
    if ($TestBuild -notmatch '^[a-z0-9-]+$') { throw 'Invalid test build identifier.' }
    $compilerArgs += "--define=TestBuild=$TestBuild"
    $compilerArgs += "--define=SetupStyle=modern $TestTheme windows11"
}
if ($TestLegacy) {
    if (!$TestBuild) { throw 'Legacy simulation is only permitted in isolated test builds.' }
    $compilerArgs += '--define=TestLegacy'
}
$compilerArgs += Join-Path $projectRoot 'installer\local-studio.iss'
& $CompilerPath @compilerArgs
if ($LASTEXITCODE -ne 0) { throw "Installer compilation failed: $LASTEXITCODE" }
Write-Output "Inno installer built for Local Studio $version (Windows theme at startup)."
