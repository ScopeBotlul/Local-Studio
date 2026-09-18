$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$toolsRoot = Join-Path $projectRoot '.tools'
New-Item -ItemType Directory -Force -Path $toolsRoot | Out-Null
$setupPath = Join-Path $toolsRoot 'innosetup-7.1.0-x64.exe'
$compilerRoot = Join-Path $toolsRoot 'inno-setup'
Invoke-WebRequest -UseBasicParsing -Uri 'https://github.com/jrsoftware/issrc/releases/download/is-7_1_0/innosetup-7.1.0-x64.exe' -OutFile $setupPath
if ((Get-FileHash -LiteralPath $setupPath -Algorithm SHA256).Hash -ne '0362a383ed217d4c4239b5933866dd96d3eb2102737da92f80f6057a4b40df2f') { throw 'Inno Setup checksum mismatch.' }
if ((Get-AuthenticodeSignature -LiteralPath $setupPath).Status -ne 'Valid') { throw 'Inno Setup signature invalid.' }
$process = Start-Process -FilePath $setupPath -ArgumentList "/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP- /CURRENTUSER /NOICONS /TASKS=`"`" /DIR=`"$compilerRoot`"" -WindowStyle Hidden -PassThru -Wait
if ($process.ExitCode -ne 0) { throw "Inno Setup compiler installation failed: $($process.ExitCode)" }
Write-Output "Compiler ready: $compilerRoot"
