$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$binary = Join-Path $projectRoot 'src-tauri\target\release\local-studio.exe'
if (-not (Test-Path -LiteralPath $binary)) { throw 'Build the release executable first.' }
$version = (Get-Content -LiteralPath (Join-Path $projectRoot 'src-tauri\tauri.conf.json') -Raw -Encoding UTF8 | ConvertFrom-Json).version
if ($version -notmatch '^\d+\.\d+\.\d+$') { throw 'Expected numeric release version.' }
$releaseRoot = Join-Path $projectRoot 'releases'
$portableRoot = Join-Path $releaseRoot "Local Studio Hub $version"
New-Item -ItemType Directory -Path $portableRoot -Force | Out-Null
Copy-Item -LiteralPath $binary -Destination (Join-Path $portableRoot 'Local Studio.exe') -Force
[IO.File]::WriteAllText((Join-Path $portableRoot 'portable.marker'), 'Local Studio Hub portable development build', [Text.UTF8Encoding]::new($false))
$readme = @'
LOCAL STUDIO {VERSION} — CORE + HUGGING FACE ENTWICKLUNGSSTAND

Start: Local Studio.exe doppelklicken. Windows x64 mit installiertem WebView2.
Daten: Local-Studio-Data neben der EXE. Zum Entpacken einen beschreibbaren Ordner wählen.

Update einer portablen Version: Alte App regulär schließen, dann die drei Dateien
aus dem ZIP in den bisherigen App-Ordner entpacken und ersetzen. Den Ordner
Local-Studio-Data behalten; er enthält die vorhandenen Einstellungen und Aufträge.

Enthalten: lokale Einstellungen, Deutsch/Englisch, Theme und UI-Zoom,
Hardwareerkennung, SQLite, SHA-256-Prüfsummenberechnung im separaten Worker,
Warteschlange, Abbruch, persistente Aufträge und Crash-Erkennung.
Neu: echte Hugging-Face-Modellsuche mit Filtern/Seiten, Modell-Details, Revisionen,
Dateiinformationen und Model Cards. Erweiterte Token-Anmeldung mit Windows-
Anmeldespeicher. Der Nutzer hat App-Login und Konto-Prüfung bestätigt.
OAuth/Browserlogin ist mit registrierter oeffentlicher Client-ID aktiviert.
Unter Hugging Face auf Anmelden klicken: Die Anmeldung startet jetzt direkt
im integrierten Browser. Nach Abschluss erscheint wieder die Kontoansicht.
Unter Hugging Face > Website im Studio liegt jetzt die echte Website.
Die Website-Anmeldung ist separat; ihre Cookies liegen im eigenen WebView-Profil.
App-Tokens werden nicht in Website-Cookies umgewandelt. Modellseiten lassen sich
über In Local Studio öffnen an die Modellansicht übergeben.

Neu in 0.4.1: Modellgroessen in GB in Suche, Details, Dateiauswahl und lokaler Liste.
Repository-Gesamtgroesse umfasst alle Varianten; die Auswahl bestimmt den Download.

Neu in 0.4.0: Dateien in den Modell-Details selbst auswählen, Download prüfen
und Auswahl herunterladen. Downloads mit echter Pause/Fortsetzung, Abbruch,
Retry, Priorität und Neustart-Recovery. Lokale Dateien unter Modelle > Lokal
gespeichert erneut prüfen. Hash-Prüfung bedeutet keine Ausführbarkeit.

Noch nicht enthalten: Modellscan/-import, vollständige Runtime-Installation, KI-Inferenz, Bild-/Video-/Audioeditor, Galerie/Projekte, Assistent
und Workflows. Keine Modelle beigepackt. Netzwerkzugriff nur auf bewusste Aktion.
Bei einem anderen Programmordner oder PC ist eine erneute Anmeldung erforderlich.
Die Gesamtanforderungen bleiben im Projekt in SPEC.md und PLAN.md erhalten.

Dies ist ein früher Core-Build. Signierte Updates und vollständige Release-
Abnahme gehören zum späteren Packaging-Meilenstein. Keine automatische
Selbstaktualisierung in dieser portablen Version.
'@
$readme = $readme.Replace('{VERSION}', $version)
[IO.File]::WriteAllText((Join-Path $portableRoot 'LIESMICH.txt'), $readme, [Text.UTF8Encoding]::new($false))
$archive = Join-Path $releaseRoot "Local-Studio-$version-hub-portable.zip"
# Package only the known build files, never local test or user data.
Compress-Archive -LiteralPath @((Join-Path $portableRoot 'Local Studio.exe'), (Join-Path $portableRoot 'portable.marker'), (Join-Path $portableRoot 'LIESMICH.txt')) -DestinationPath $archive -Force
$installer = Join-Path $projectRoot "src-tauri\target\release\bundle\nsis\Local Studio_${version}_x64-setup.exe"
if (Test-Path -LiteralPath $installer) {
    Copy-Item -LiteralPath $installer -Destination (Join-Path $releaseRoot "Local-Studio-$version-hub-setup.exe") -Force
}
$packageFiles = @($archive)
$packagedInstaller = Join-Path $releaseRoot "Local-Studio-$version-hub-setup.exe"
if (Test-Path -LiteralPath $packagedInstaller) { $packageFiles += $packagedInstaller }
$hashes = Get-FileHash -LiteralPath $packageFiles -Algorithm SHA256
$checksums = ($hashes | ForEach-Object { "$($_.Hash.ToLowerInvariant())  $([IO.Path]::GetFileName($_.Path))" }) -join "`n"
[IO.File]::WriteAllText((Join-Path $releaseRoot "SHA256SUMS-$version.txt"), ($checksums + "`n"), [Text.Encoding]::ASCII)
$hashes | Format-Table Hash,Path -AutoSize
