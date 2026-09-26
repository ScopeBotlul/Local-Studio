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

Neu in 0.30.0: kompakte zentrierte Hauptnavigation nach Arbeitsbereichen.
Create oeffnet eine zweite Werkzeugleiste fuer Bildgenerierung, Bildbearbeitung,
Videoschnitt und GIF-Studio. Galerie und Modelle bleiben zentral erreichbar;
Hugging Face, Downloads, Auftraege und Einstellungen liegen rechts. Die bisherige
doppelte Studio-Navigation und Breadcrumb-Leiste wurden entfernt.

Neu in 0.36.13: Ein Benachrichtigungssymbol neben den Downloads sammelt
Wiederherstellungen und wichtige Hinweise, ohne den Arbeitsbereich zu verdecken.
Projekt-, Bildarbeitsstand- und Auftragswiederherstellungen lassen sich direkt
im Benachrichtigungsfenster aufrufen; ein Zaehler zeigt offene Hinweise.

Neu in 0.36.14: Das AMD-Low-Memory-Profil deaktiviert auf kleinen Windows-APUs
zusaetzlich mmap fuer grosse Safetensors und den nativen comfy-kitchen-HIP-
Dispatcher. Damit umgeht Local Studio zwei offizielle ROCm-/UMA-Risikopfade,
die zum in Bugreport #5 gemeldeten Sampling-Absturz passen.

Neu in 0.36.15: Das Bildstudio kann automatisch zwischen ComfyUI und dem
mitgelieferten Vulkan-Worker waehlen. AMD-/Intel-Handhelds ohne NVIDIA verwenden
stable-diffusion.cpp mit automatischer Speicherverteilung; NVIDIA und LoRAs
verwenden weiterhin ComfyUI. Vulkan und ComfyUI lassen sich auch bewusst waehlen.
Alte Projekte werden kompatibel mit automatischer Engine-Auswahl geladen.

Neu in 0.36.16: Die Oberflaechenskalierung fuellt den Arbeitsbereich wieder
vollstaendig und das laufende Windows-Fenstericon folgt der Akzentfarbe. LoRAs
lassen sich auf AMD-/Intel-Handhelds jetzt auch mit dem isolierten Vulkan-Worker
auswaehlen und anwenden; ComfyUI bleibt als alternative Engine erhalten.

Neu in 0.36.17: Der Hauptarbeitsbereich heisst wieder Studio. Die Akzentfarbe
wird als grosses Windows-Fenstericon gesetzt, damit auch die laufende
Taskleistenschaltflaeche statt nur des kleinen Fenstericons aktualisiert wird.

Neu in 0.36.18: Modelle hat jetzt eine eigene Werkzeugleiste fuer lokal
gespeicherte Modelle, Hugging Face und Civitai. Civitai bietet Anmeldung und
Website im getrennten integrierten Browserprofil, schnelle Typfilter sowie die
bestehende sichere Detail- und Downloadpruefung. Die Website-Sitzung wird nicht
als API-Schluessel ausgelesen oder in Downloadauftraegen gespeichert.

Neu in 0.36.12: AMD-Systeme mit hoechstens 16 GB gemeinsamem Speicher starten
ComfyUI ohne DynamicVRAM und mit Low-VRAM-Modus. Der SDXL-Textencoder laeuft
dadurch auf der CPU und umgeht den auf dem ROG Ally abgestuerzten ROCm-Pfad.

Neu in 0.36.11: Verwaltete AMD-ComfyUI-Installationen deaktivieren gepinnten
Hostspeicher und asynchrones Weight-Offloading. Das reduziert den im ROG-Ally-
Bericht sichtbaren RAM-Druck beim Laden grosser SDXL-Modelle. Crashberichte
bewahren zusaetzlich gezielt den Bereich um native Fatal- und Speicherfehler.

Neu in 0.36.10: Ein waehrend der Bildgenerierung abgestuerztes ComfyUI beendet
den Auftrag jetzt sofort mit einer klaren Meldung. Bugreports behalten Anfang
und Ende des Crashlogs und enthalten auch die Anzahl der Bildauftraege.

Neu in 0.36.9: Der Local-Studio-Modellordner wird ComfyUI jetzt als direkter
absoluter Checkpoint-Suchpfad uebergeben. Dadurch erkennt die verwaltete Engine
auch Checkpoints in verschachtelten hf-Downloadordnern zuverlaessig.

Neu in 0.36.8: Die Modellbereitschaft wird direkt gegen ComfyUIs tatsaechliche
Checkpoint-Liste geprueft. Eine fremd gestartete Instanz auf Port 8188 wird in den
Einstellungen klar als extern angezeigt; sie muss beendet werden, bevor Local
Studio seine verwaltete Engine mit dem zusaetzlichen Modellordner starten kann.

Neu in 0.36.7: Vor dem Workflow liest Local Studio die exakten Checkpoint-, LoRA-,
Sampler- und Scheduler-Namen aus der laufenden ComfyUI-API. Verschachtelte
Windows-Modellpfade werden dadurch im von ComfyUI erwarteten Format gesendet.
Konkrete Workflow-Ablehnungen stehen zusaetzlich in den Auftragsdetails und Logs.

Neu in 0.36.6: Das Bildstudio zeigt beim Oeffnen einen leeren Canvas statt
automatisch den letzten fehlgeschlagenen Auftrag. Laufende und wartende Auftraege
bleiben sichtbar; alte Ergebnisse und Fehler koennen bewusst im Verlauf geoeffnet
werden und sind eindeutig als Meldung des gespeicherten Auftrags gekennzeichnet.

Neu in 0.36.5: Verwaltete ComfyUI-Auftraege besitzen keine pauschale Grenze fuer
den von Windows gerade als frei gemeldeten RAM mehr. Auf Geraeten mit gemeinsamem
CPU-/GPU-Speicher entscheidet ComfyUI/ROCm ueber Laden und Auslagern. Reihenfolge
und die Sperre gegen parallele GPU-Auftraege bleiben erhalten.

Neu in 0.36.4: ComfyUI-Auftraege werden auf PCs mit gemeinsamem CPU-/GPU-Speicher
nicht mehr wegen der Dateigroesse des Checkpoints vorzeitig abgewiesen. Local Studio
reserviert eine kleine Arbeitsreserve und laesst ComfyUI/ROCm den tatsaechlichen
Speicherbedarf verwalten. Die serielle GPU-Ausfuehrung bleibt aktiv.

Neu in 0.36.3: Die von Local Studio verwaltete ComfyUI-Engine erhaelt den lokalen
Modellordner als zusaetzlichen Checkpoint-Pfad. Heruntergeladene SDXL-Modelle wie
WAI Illustrious laufen dadurch auch ueber das AMD-ROCm-Paket, ohne Kopie der Gewichte.

Neu in 0.36.2: Erfolgreich gepruefte Modell-Downloads werden automatisch in die
Modellbibliothek uebernommen und stehen dadurch ohne manuellen Suchlauf im Studio
bereit. Bereits abgeschlossene Downloads werden beim Start einmalig nachgetragen.

Neu in 0.36.1: Unter dem Download-Symbol zeigt eine schmale Leiste den gemeinsamen
Fortschritt aller laufenden Downloads in der eingestellten Akzentfarbe. Downloads
mit bekannter Groesse werden nach Bytes zusammengefasst; bei unbekannter Groesse
zeigt die Leiste einen laufenden Status.

Neu in 0.36.0: Unter Hilfe > Fehler melden erstellt Local Studio einen vollstaendigen
bereinigten Diagnosebericht mit App-Einstellungen, Hardware, Komponentenstatus und
begrenzten Logs. Tokens, Prompts, Medien und persoenliche Pfade bleiben ausgeschlossen.
Der Bericht wird lokal gespeichert und als vorausgefuelltes GitHub-Issue geoeffnet.

Neu in 0.35.5: AMD-Portable-Installationen verwenden automatisch den kompatiblen
Split-Cross-Attention-Modus. Dadurch wird der native AOTriton-Starttest umgangen,
der auf dem ROG Ally mit gfx1103 in amdhip64_7.dll abstuerzen kann.

Neu in 0.35.4: Portable ComfyUI startet im offiziellen Windows-Standalone-Modus.
Startausgaben werden lokal protokolliert; ein frueher Absturz wird sofort erkannt
und das Protokoll kann in den ComfyUI-Einstellungen aufgeklappt werden.

Neu in 0.35.3: Pruefung und Entpacken des ComfyUI-Archivs laufen unter Windows
ohne sichtbares tar.exe-Konsolenfenster. Der Installationsstatus bleibt direkt
im Einrichtungsdialog sichtbar.

Neu in 0.35.2: Der offizielle AMD-Build enthaelt 3.666 ROCm-Kerneldateien mit
einem Sternchen im 7z-Namen. Windows tar legt dieses Zeichen sicher als
Vollbreitenstern ab. Die Archivpruefung erlaubt genau diese .aks2-Dateien im
offiziellen aotriton-Ordner; Wildcards an allen anderen Stellen bleiben gesperrt.

Neu in 0.35.1: Die ComfyUI-Installation akzeptiert die langen, verschachtelten
Python-/PyTorch-Pfade des offiziellen portablen Archivs. Die Sicherheitspruefung
bleibt komponentenweise aktiv und sperrt absolute Pfade, Laufwerkspfade,
Verzeichniswechsel, ungueltige Zeichen und reservierte Windows-Geraetenamen.

Neu in 0.35.0: ComfyUI kann im Update-Center nach bewusstem Klick direkt
aktualisiert werden. Local Studio beendet eine selbst gestartete Engine, fuehrt
den offiziellen stabilen Core-Updater mit der eingebetteten Python-Laufzeit aus
und startet die Engine danach wieder. Fortschritt und lokales Protokoll bleiben
sichtbar. Erforderliche Core-Abhaengigkeiten duerfen angepasst werden; Custom
Nodes werden nicht aktualisiert. Laufende Bildauftraege sperren das Update.

Neu in 0.34.0: Das Update-Center zeigt Local Studio und ComfyUI in zwei Bereichen.
Die gemeinsame automatische Startpruefung sucht fuer beide nach neuen Versionen,
installiert aber nichts ohne Nutzeraktion. ComfyUI verwendet den offiziellen
portablen Updater; Custom Nodes werden nicht automatisch aktualisiert.

Neu in 0.33.1: Der ComfyUI-Installationsbutton zeigt waehrend des Vorgangs direkt
die aktuelle Phase und den echten prozentualen Downloadfortschritt. Nach Download,
Pruefung und Entpacken wird die Installation automatisch als Engine eingebunden.

Neu in 0.33.0: ComfyUI Portable wird bei der Einrichtung direkt in Local Studio
heruntergeladen, per offizieller SHA-256-Pruefsumme geprueft, sicher entpackt und
als lokale Bildengine eingerichtet. NVIDIA-, NVIDIA-CUDA-12.6-, AMD- und Intel-
Pakete sind waehlbar. Der Fortschritt erscheint im Download-Menue. Mit
Nach Installation suchen werden typische lokale Ordner begrenzt durchsucht;
abweichende Installationsorte koennen weiterhin manuell ausgewaehlt werden.
Custom Nodes oder weitere Modellabhaengigkeiten werden nicht installiert.

Neu in 0.32.0: Ist die automatische Update-Suche aktiv, erscheint bei einem
verfuegbaren Release selbststaendig der Update-Dialog mit Abbrechen und Update
installieren. Andere geoeffnete Dialoge werden abgewartet. Die bisherige lokale
18+-Sperre samt Altersabfrage, PIN/Passwort, Neustartsperre, gesperrten Ansichten
und Desktop-IPC wurde entfernt. Alte Kennzeichnungen bleiben lesbar, haben aber
keine Sperrwirkung mehr. 18+-Angaben in Modellkatalogen sind nur Metadaten und
Suchfilter.

Neu in 0.31.0: Civitai-Modellsuche direkt in der Modellbibliothek mit Typ,
Basisfamilie, Sortierung, Zeitraum, Versionen, Dateigroessen und Scanstatus.
Oeffentliche, primaere Safetensors-Dateien mit SHA-256 und erfolgreichen Civitai-
Scans koennen ueber die Downloadwarteschlange in passende ComfyUI-Ordner oder den
lokalen Modellordner geladen werden. Geschuetzte/API-Key-Downloads bleiben offen.
Fehlende Desktop-Berechtigungen fuer ComfyUI, GIF, Civitai und Danbooru behoben.

Neu in 0.29.0: portable ComfyUI-Erkennung, Einrichtung, lokaler Start und SDXL-
Bildauftraege ueber 127.0.0.1. LoRAs im Studio auswaehlen und gewichten.
Rechte Studiogalerie mit gemeinsamem Speicherordner, zufaellige Seeds und Strg+F
in Prompt/Negativ-Prompt. Modelle nach erneuter Warnung vom Datentraeger loeschen.
Civitai-Bildvorlagen und LoRA-Suche im eingeschraenkten internen Browser.
Danbooru-Post-Tags uebernehmen, lokal formatieren und optionale Zensur-Tags filtern.
GIF-Studio, lokaler Programmiermodus, kompakte Navigation und Downloaduebersicht.
Keine automatische Installation von Custom Nodes, Modellcode oder Abhaengigkeiten.
Lose Encoder/UNet/VAE-Dateien brauchen weiterhin familienbezogene Workflows.

Neu in 0.28.0: Update-Popup beim Start, Projektuebersicht auf der Startseite,
korrigierter Neues-Projekt-Dialog, Prompt-Feld und aufgeklappter Negativ-Prompt.
Sichtbare Hugging-Face-Bereichsfilter mit 20 Aufgaben und Unterstuetzungshinweisen.
Eigene SDXL-Groessen: 256-4096 je Seite in 64er-Schritten, maximal 4.194.304 Pixel.
Ungueltige Groessen werden erklaert; passende Masse nur nach Uebernehmen-Klick.
Grosse Generierungen brauchen mehr VRAM; reale 4-MP-Inferenz noch nicht abgenommen.

Neu in 0.27.0: gemeinsame Titel-/Menueleiste mit zentriertem Projektnamen.
Projektverwaltung ueber Datei; Details und Medien ueber Ansicht.
Bildstudio mit Canvas, Zoom/Pan, seitlichen Eingaben und eigener Aufloesung.
SDXL-Modellwahl strukturell gefiltert, Vorpruefung automatisch.
Bibliothek nach Einsatzzweck, Erweiterungen und technischen Komponenten.
Lokale 18+-Metadaten werden erkannt; unmarkierte Inhalte bleiben unbekannt.
Die damalige Aufloesungsgrenze wurde in 0.28.0 erweitert (siehe oben).
Lose Encoder/VAEs und andere Modellfamilien sind noch nicht ausfuehrbar.

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
