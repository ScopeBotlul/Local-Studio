$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$version = (Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $root 'package.json') | ConvertFrom-Json).version
$exe = Join-Path $root 'src-tauri\target\release\local-studio.exe'
$portable = Join-Path $root "releases\Local Studio Hub $version"
$archive = Join-Path $root "releases\Local-Studio-$version-hub-portable.zip"
$installer = Join-Path $root "releases\Local-Studio-$version-hub-setup.exe"
$builtInstaller = Join-Path $root "src-tauri\target\release\bundle\inno\Local-Studio-$version-hub-setup.exe"
function Hash([string]$Path) { (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant() }
if ((Hash $exe) -ne (Hash (Join-Path $portable 'Local Studio.exe'))) { throw 'Portable executable differs from release executable.' }
if ((Hash $installer) -ne (Hash $builtInstaller)) { throw 'Packaged installer differs from built installer.' }
$expected = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
@('Local Studio.exe','portable.marker','LIESMICH.txt') | ForEach-Object { [void]$expected.Add($_) }
$runtimeCounts = @{}
foreach ($kind in @('image','video','assistant','speech')) {
    $manifest = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $root "src-tauri\$kind-runtime.json") | ConvertFrom-Json
    $runtimeCounts[$kind] = @($manifest.files.PSObject.Properties).Count
    foreach ($file in $manifest.files.PSObject.Properties) {
        $relative = "$kind-runtime/$($file.Name)"
        [void]$expected.Add($relative)
        if ((Hash (Join-Path $portable $relative)) -ne [string]$file.Value) { throw "Runtime hash mismatch: $relative" }
    }
}
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [IO.Compression.ZipFile]::OpenRead($archive)
try {
    $actual = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($entry in $zip.Entries) { if ($entry.Name) { [void]$actual.Add($entry.FullName.Replace('\','/')) } }
    if (!$actual.SetEquals($expected)) { throw 'Portable archive entries differ from the allowlist.' }
    $entry = $zip.GetEntry('Local Studio.exe'); $stream = $entry.Open()
    try { $sha = [Security.Cryptography.SHA256]::Create(); $zipExe = ([BitConverter]::ToString($sha.ComputeHash($stream))).Replace('-','').ToLowerInvariant() } finally { $stream.Dispose() }
    if ($zipExe -ne (Hash $exe)) { throw 'Archived executable differs from release executable.' }
    $readme = $zip.GetEntry('LIESMICH.txt'); $reader = [IO.StreamReader]::new($readme.Open(), [Text.Encoding]::UTF8)
    try { if (!$reader.ReadToEnd().Contains("Neu in $version")) { throw 'Portable readme is not current.' } } finally { $reader.Dispose() }
} finally { $zip.Dispose() }
foreach ($name in @('package.json','package-lock.json','src-tauri\tauri.conf.json')) {
    $text = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $root $name)
    if ($text -notmatch ('"version"\s*:\s*"' + [regex]::Escape($version) + '"')) { throw "Version mismatch: $name" }
}
$sums = Get-Content -Raw -Encoding ASCII -LiteralPath (Join-Path $root "releases\SHA256SUMS-$version.txt")
if (!$sums.Contains((Hash $archive)) -or !$sums.Contains((Hash $installer))) { throw 'Checksum file mismatch.' }
$result = [ordered]@{ passed=$true; scope='Distribution bytes'; version=$version; executableSha256=(Hash $exe); zipSha256=(Hash $archive); installerSha256=(Hash $installer); zipFiles=$expected.Count; runtimeFiles=$runtimeCounts }
$json = $result | ConvertTo-Json -Depth 5
[IO.File]::WriteAllText((Join-Path $root ".artifacts\package-bytes-verification-$version.json"), $json, [Text.UTF8Encoding]::new($false))
$json
