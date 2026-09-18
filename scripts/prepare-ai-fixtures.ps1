param([string]$Destination = (Join-Path $PSScriptRoot '../.artifacts'))
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Speech
[System.IO.Directory]::CreateDirectory([System.IO.Path]::GetFullPath($Destination)) | Out-Null
foreach ($item in @(
  @{ Language = 'en'; Culture = 'en-US'; Text = 'Welcome to Local Studio. This program edits pictures and videos on your computer. Your files stay private.' },
  @{ Language = 'de'; Culture = 'de-DE'; Text = 'Willkommen im lokalen Studio. Wir bearbeiten Bilder und Videos auf diesem Computer. Alle Dateien bleiben auf deinem Geraet.' }
)) {
  $voice = New-Object System.Speech.Synthesis.SpeechSynthesizer
  try {
    $selected = $voice.GetInstalledVoices() | Where-Object { $_.Enabled -and $_.VoiceInfo.Culture.Name -eq $item.Culture } | Select-Object -First 1
    if (-not $selected) { throw "Install a Windows SAPI voice for $($item.Culture) to generate this test fixture." }
    $voice.SelectVoice($selected.VoiceInfo.Name)
    $target = [System.IO.Path]::GetFullPath((Join-Path $Destination "speech24-$($item.Language).wav"))
    if (Test-Path -LiteralPath $target) { throw "Fixture already exists: $target. Choose a new destination or reuse the existing fixture." }
    $voice.SetOutputToWaveFile($target)
    $voice.Speak($item.Text)
    $voice.SetOutputToNull()
    Write-Output "Generated real speech: $target ($($selected.VoiceInfo.Name))"
  } finally { $voice.Dispose() }
}
