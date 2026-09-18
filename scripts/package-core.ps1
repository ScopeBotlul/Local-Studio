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
& (Join-Path $PSScriptRoot 'stage-image-runtime.ps1') -Destination (Join-Path $portableRoot 'image-runtime')
& (Join-Path $PSScriptRoot 'stage-video-runtime.ps1') -Destination (Join-Path $portableRoot 'video-runtime')
& (Join-Path $PSScriptRoot 'stage-ai-runtimes.ps1') -Destination $portableRoot
$readme = @'
LOCAL STUDIO {VERSION} — CORE + HUGGING FACE ENTWICKLUNGSSTAND

Start: Local Studio.exe doppelklicken. Windows x64 mit installiertem WebView2.
Daten: Local-Studio-Data neben der EXE. Zum Entpacken einen beschreibbaren Ordner wählen.

Update einer portablen Version: Alte App regulär schließen, dann die Programmdateien und alle vier Runtime-Ordner (image-runtime, video-runtime, assistant-runtime, speech-runtime)
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

Neu in 0.26.0: lokale 18+-Sperre mit Altersbestaetigung, PIN oder Passwort,
Neustartsperre und Modellkennzeichnung. Geschuetzte Prompts, Galerie-Metadaten,
Projekte und Chatverlaeufe. Laufende Bilder werden beim Sperren weiter berechnet.
Kopien/Exporte behalten die Kennzeichnung. Geschuetzte Projekte: Format 5.
Einrichtung oben ueber 18+. App-Zugriffsschutz, keine Dateiverschluesselung.
Local-Studio-Data beim Update vollstaendig behalten. Gemischte Verwaltungs-
und Verlaufsansichten verlangen vorsorglich eine Entsperrung.

Neu in 0.25.0: SDXL-Bild-zu-Bild, Inpainting mit pixelgenauem Maskenschutz,
Stapel mit gleicher/aufsteigender Seed-Folge und bewusst waehlbarer CPU-VAE.
Referenz und Maske werden pro Auftrag kopiert; Projektformat 4 bettet beide ein.
PNG-Galeriebild als Referenz uebernehmen; Ebenenmasken als PNG exportieren.
Gemeinsame Ressourcenwarteschlange, optionale Live-Hardwarewerte, Windows-Akzent
und Infobereich. Gepruefte Modellverschiebung, bewusste gemeinsame Modellupdates,
echte lokale Messwerte und bearbeitbare gelernte Praeferenzen.
Noch keine vollstaendige Umsetzung des Masterprompts; u.a. KI-Video, Musikmodelle,
Workflows, Vision/Inhaltsindex und weitere Editorfunktionen offen.

Neu in 0.24.0: Lokaler Assistent mit bewusstem Modelldownload, DE/EN-Chat,
Laden/Entladen, Abbruch und sichtbaren Studio-Werkzeugen. Hardware lesen,
installierte Modelle und Galerie-Metadaten suchen, Bildauftraege vorbereiten.
Automatische lokale Transkription fuer Video-/Audioclips, Text/Zeiten pruefen,
in die Timeline uebernehmen und als SRT speichern. Keine Sprechertrennung.
Modelle sind nicht beigepackt. Alle vier Runtime-Ordner beim Update ersetzen:
image-runtime, video-runtime, assistant-runtime und speech-runtime.

Neu in 0.23.0: Einzelbildvorschau am Abspielkopf ohne kompletten Timeline-Render.
Proxies und echte Audio-Wellenformen unter Videoschnitt berechnen. Export nutzt
Originale. Clipkanten ziehen, Strg-Mehrfachauswahl, gemeinsam bewegen und einrasten.
Bildeditor: Skalier-/Drehgriffe, Pinselkontur und Projektmedien direkt einfuegen.
Einstellungen > Speicher: registrierte Render-/Proxy-/Wellenform-Dateien nach
Aufbewahrungsfrist bereinigen. Aktive Projektcaches und Originale bleiben erhalten.

Neu in 0.22.0: Studio mit Ebenen/Masken und lokalem Video-/Audio-Mehrspurschnitt.
PNG/JPEG-Kompositionen, MP4/WebM-Export, Keyframes, manuelle Untertitel und SRT.
Beide Editoren samt Medien in einer Projektdatei speichern (Format 3).
Keine KI-Videoerzeugung; Vorschau bewusst rendern. FFmpeg-Lizenzen/Quellen
stehen unter video-runtime. Originalmedien bleiben unveraendert.

Neu in 0.21.0: Helligkeit / Kontrast / Saettigung / relative Farbtemperatur,
Vorher/Nachher, Original + aktive Bearbeitungsschritte in .localstudio-Projekten.
Im Editor ins Projekt uebernehmen, Editor schliessen und Projekt speichern.
Projektbild bearbeiten funktioniert auch ohne urspruengliche Galeriedatei.
Galerie-Mehrfachauswahl > Bilder korrigieren: neue PNG/JPEG-Varianten, Fortschritt,
Einzelfehler und Abbruch nach aktuellem Bild. Originale bleiben unveraendert.
Projektdateien mit Rezept benoetigen Version 0.21.0 oder neuer.

Neu in 0.20.0: Windows-Menueleiste Datei / Bearbeiten / Ansicht / Hilfe.
Projekte erstellen, oeffnen, zuletzt geoeffnete, speichern / speichern unter,
schliessen; Rueckgaengig / Wiederholen im Bildeditor, Zoom, Anleitung und Info.
Hilfe > Nach Updates suchen prueft signierte GitHub-Veroeffentlichungen.
Automatische Pruefung beim Start ist dort abschaltbar; kein automatischer Download.
Portable: ZIP bewusst herunterladen und Local-Studio-Data immer behalten.
Installierte Ausgabe: geprueftes Update herunterladen, bestaetigen und neu starten.

Neu in 0.19.0: Galerie > Bild bearbeiten: Zuschneiden, 90-Grad-Drehung, Spiegeln,
Groesse mit/ohne Seitenverhaeltnis, Undo/Redo (Strg+Z/Y, in Einstellungen aenderbar).
PNG mit Alpha oder JPEG mit Qualitaet als neue Datei/Variante exportieren.
Lokale Entwuerfe beim erneuten Oeffnen desselben Bildes bewusst fortsetzen.
Originale bleiben erhalten. Statische 8-Bit-PNG/JPEG/BMP bis 64 MiB / 32 MP.
ICC-RGB-Profile bleiben erhalten, EXIF/GPS werden entfernt.
Ebenen, Masken, RAW/HDR und weitere Formate folgen spaeter.

Neu in 0.18.0: Video-Miniaturen aus lokal decodierten Videoframes mit Cache.
Windows-Ordnerueberwachung aktualisiert die Galerie bei Datei-Aenderungen.
Kopie als Variante anlegen, Herkunftskette ansehen und Hauptversion bestimmen.
Verknuepfungen bleiben bei Umbenennen erhalten; geloeschte Quellen nur als Metadaten.
Zuletzt geoeffnete Projekte direkt aus der Projektleiste oeffnen.
Installer registriert .localstudio fuer Doppelklick. Portable registriert nichts;
Dateien koennen ueber Oeffnen mit oder als EXE-Argument geoeffnet werden.
Eine laufende App uebernimmt den Projektwunsch und fragt bei ungespeicherten Aenderungen.
Video-Codecs abhaengig von WebView/Windows; nicht lesbare Clips behalten Symbolkarte.

Neu in 0.17.0: Bilder direkt aus Studio und Galerie ins Projekt uebernehmen.
Dateien aus Explorer auf Projektleiste oder Galerie ziehen; Originale bleiben erhalten.
Projekte umbenennen, entfernte Medien zurueckholen, bis zu 20 lokale Recovery-Staende.
Einstellungen > Speicher und Bereinigung zeigt bekannte Arbeitsdateien mit Vorschau.
Automatische Bereinigung standardmaessig an, ab sieben Tagen; Frist einstellbar.
Aktive Projekte, die letzten drei Recovery-Staende und ungespeicherte Bilder geschuetzt.
Modelle, Galerieoriginale, Downloads und unbekannte Dateien bleiben erhalten.

Neu in 0.16.0: Echte .localstudio-Projekte mit Bild-Studio-Eingaben und Medien.
Projektleiste aufklappen, Namen eingeben, anlegen und Medien hinzufuegen.
Strg+S speichert; Speichern unter erstellt eine weitere Projektdatei.
Bild-/Video-/Audiovorschau, Medien entfernen, Kopien in Galerie exportieren.
Modellgewichte bleiben extern; Originalmodell per SHA-256 wieder zuordnen.
Lokalen Projektarbeitsstand nach Neustart bewusst fortsetzen.
Beim Beenden Projektdatei im gemeinsamen Dialog optional mitspeichern.
Maximal 100 Medien / 2 GiB. Ab 0.17.0 sichere Bereinigung bekannter Arbeitskopien.
Editoren, Timeline, Dateiassoziation und vollstaendige Recovery bleiben offen.

Neu in 0.15.0: Zehn getrennte Speicherorte mit Schreibpruefung und Standard-Ruecksetzung.
Neue Bildauftraege behalten ihren temporaeren Ausgabeordner auch nach Pfadwechsel.
Elf aktive Tastaturaktionen frei belegen, Konflikte erkennen und Belegungen zuruecksetzen.
Galerie mit vollstaendigem Einpassen, 100-Prozent-Ansicht und Maus-Pan.
Vorhandene Dateien werden beim Pfadwechsel nicht verschoben. Projekte und Recovery
werden ab 0.16.0 genutzt; Proxies bleiben ein vorbereitetes Ziel.

Neu in 0.14.0: Zwei Bilder nebeneinander oder mit Schieberegler vergleichen,
gemeinsam zoomen/verschieben und A/B tauschen. Originale bleiben unveraendert.
Galerie nach Name, Aenderungsdatum oder Groesse sortieren, jeweils in beide
Richtungen. Strg+A, Esc, Pfeiltasten, F2 und Entf fuer Galerieaktionen.
Entf oeffnet weiterhin zuerst die Papierkorb-Bestaetigung.

Neu in 0.13.0: Strg-/Umschalt-Auswahl, Modell-/Promptsuche, erweiterte Metadaten,
Umbenennen und Verschieben innerhalb der Galerie, Sammelaktionen fuer Dateien.
Lokaler App-Papierkorb mit Vorschau, Wiederherstellen und bestaetigtem endgueltigem
Loeschen/Leeren. Keine automatische Leerung. Konflikte ueberschreiben keine Dateien;
bei Fehlern stoppen Dateiaktionen mit sichtbarem Teilergebnis.

Neu in 0.12.0: Mehrere Medien in Raster oder Liste auswaehlen und gemeinsam
Favoriten setzen/aufheben oder einzelne Tags hinzufuegen/entfernen.
Bis zu 50 Medien der aktuellen Seite. Bei Konflikten keine Teilaenderungen.
Originaldateien und individuelle andere Tags bleiben erhalten.

Neu in 0.11.0: Raster-/Listenansicht mit lokalen Bildminiaturen fuer PNG, JPEG,
WebP, GIF und BMP. Sichtbare Karten laden bei Bedarf; Cache bleibt ueber Neustart.
Vorschaubilder neu aufbauen veraendert keine Originale oder Favoriten/Tags.
Video/Audio/AVIF vorerst als Symbolkarten mit vorhandener Medienvorschau.

Neu in 0.10.0: Favoriten und frei vergebene Tags fuer alle Galerie-Medien.
Tags und Favoriten kombinieren, nach Tags suchen. Lokale Datenbank speichert
Markierungen getrennt von den Originaldateien. Umbenennen innerhalb derselben
Galerie auf NTFS behaelt die Zuordnung; Kopien erhalten eigene Markierungen.

Neu in 0.9.0: Gemeinsame Galerie fuer Bilder, Video, Audio und Musik in echten
Ordnern. Dateiimport kopiert mit Hashpruefung; Originale bleiben erhalten.
Ordnernavigation, Suche/Typfilter, automatische Erkennung waehrend der Ansicht,
Bildzoom und lokale Video-/Audiowiedergabe, abhaengig vom Format/Codec.
Drag-and-drop und Projekte ab 0.17.0; Video-Miniaturen ab 0.18.0.

Neu in 0.8.0: Weitere Bilder waehrend einer Generierung einreihen, maximal 20
wartende Auftraege mit eigenen Modellen/Prompts/Parametern. Nacheinander ausfuehren,
wartende Auftraege einzeln abbrechen und nach Neustart bewusst erneut einreihen.
Einstellungen aus Bildauftraegen/Galerie wiederherstellen ohne Generierungsstart.

Neu in 0.7.1: Der Beenden-Dialog hat einen deckenden Hintergrund passend zum
hellen, dunklen oder Windows-Systemdesign.

Neu in 0.7.0: Modellauswahl mit GB im Studio, Parameter pro Modell und Bildjobs
unter Auftraege. Beim Beenden Bilder speichern, verwerfen oder Abbrechen.
Optional Arbeitsstand behalten; nach einem Absturz Wiederherstellung anbieten.
Inferenz wird nicht automatisch fortgesetzt.

Neu in 0.6.1: Modellformate, unbestaetigte Kandidaten und alte ausgeblendete Treffer
werden getrennt angezeigt. Programm-/Testordner und reine Textdateien mit Modell-
Endung werden ausgefiltert. Alte Suchlisten werden beim Start eingeordnet.
Originaldateien bleiben erhalten.

Neu in 0.6.0: Schnellsuche durchsucht typische Modellablagen und HF-Caches.
Studio > Image kann einzelne vollstaendige SDXL-Safetensors-Checkpoints mit
NVIDIA/Vulkan ausfuehren. Modell waehlen, Ausfuehrbarkeit pruefen, Prompt eingeben,
Bild generieren. Fortschritt, Abbruch, PNG-Vorschau und In Galerie speichern.
Der Ordner image-runtime gehoert zum Programm. Keine Modelldateien beigepackt.
Ungespeicherte Bilder werden beim Beenden behandelt; automatische Bereinigung noch offen.

Neu in 0.5.1: Vollstaendige Suche durchsucht alle lokalen Laufwerksbuchstaben.
Fortschritt und Abbruch sind sichtbar; uebersprungene Pfade werden aufgefuehrt.

Neu in 0.5.0: Unter Modelle > Lokal gespeichert einen Modellordner durchsuchen.
Dateien bleiben am Originalort. Groesse, Format, Struktur und fehlende Index-Dateien
werden angezeigt. Erneut pruefen und Aus Liste entfernen veraendern keine Modelldateien.
Der Importstatus allein bestaetigt keine Ausfuehrbarkeit; SDXL seit 0.6.0 im Studio pruefen.

Neu in 0.4.2: Installer und Deinstallation passen sich beim Start an Windows an.

Neu in 0.4.1: Modellgroessen in GB in Suche, Details, Dateiauswahl und lokaler Liste.
Repository-Gesamtgroesse umfasst alle Varianten; die Auswahl bestimmt den Download.

Neu in 0.4.0: Dateien in den Modell-Details selbst auswählen, Download prüfen
und Auswahl herunterladen. Downloads mit echter Pause/Fortsetzung, Abbruch,
Retry, Priorität und Neustart-Recovery. Lokale Dateien unter Modelle > Lokal
gespeichert erneut prüfen. Hash-Prüfung bedeutet keine Ausführbarkeit.

Noch nicht enthalten: universelle Modell-/Runtime-Verwaltung, weitere Bildadapter,
vollstaendiger Bildeditor, erweiterte Video-/Audiofunktionen, vollstaendige Galerie/Projekte,
vollstaendige Assistentenaktionen und Workflows. Keine Modelle beigepackt. Die abschaltbare Updatepruefung
verbindet sich beim Start mit GitHub; sie uebertraegt keine Medien oder Prompts.
Bei einem anderen Programmordner oder PC ist eine erneute Anmeldung erforderlich.
Die Gesamtanforderungen bleiben im Projekt in SPEC.md und PLAN.md erhalten.

Dies ist ein Entwicklungsstand. Signierte Updatepruefung ist enthalten;
die vollstaendige Produktabnahme bleibt offen. Keine automatische
Selbstaktualisierung in dieser portablen Version.
'@
$readme = $readme.Replace('{VERSION}', $version)
[IO.File]::WriteAllText((Join-Path $portableRoot 'LIESMICH.txt'), $readme, [Text.UTF8Encoding]::new($false))
$archive = Join-Path $releaseRoot "Local-Studio-$version-hub-portable.zip"
# Package only the known build files, never local test or user data.
Compress-Archive -LiteralPath @((Join-Path $portableRoot 'Local Studio.exe'), (Join-Path $portableRoot 'portable.marker'), (Join-Path $portableRoot 'LIESMICH.txt'), (Join-Path $portableRoot 'image-runtime'), (Join-Path $portableRoot 'video-runtime'), (Join-Path $portableRoot 'assistant-runtime'), (Join-Path $portableRoot 'speech-runtime')) -DestinationPath $archive -Force
$installer = Join-Path $projectRoot "src-tauri\target\release\bundle\inno\Local-Studio-$version-hub-setup.exe"
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
