# GitHub-Upload 0.27.0 vorbereitet (2026-09-19)

Neue Builds werden auf Nutzerwunsch standardmäßig auf GitHub hochgeladen; die Regel steht in AGENTS.md. Release-Entwurf 391865488 enthält Installer, portable ZIP, Prüfsummen und signierte Update-Metadaten. Alle fünf Assets sind vollständig hochgeladen; Größen und GitHub-SHA256 stimmen mit den lokalen Dateien überein. Der Nutzer hat zusätzlich Commit und direkten Push der 54 vorbereiteten Quellcode-/Dokumentationsdateien nach main sowie die Veröffentlichung ausdrücklich bestätigt. Der Release wird dem zugehörigen neuen Quellcode-Commit zugeordnet. Keine neuen Tests oder Builds ausgeführt. Die verbleibende RAM-bedingte Prüflücke ist in docs/RELEASE_0.27.0.md offengelegt.

# Lokaler Build 0.27.0 (2026-09-19)

Tests und neuer Build wurden ausdrücklich wieder freigegeben. Windows-EXE, Installer und portable ZIP sind unter `releases/` erstellt; in diesem Auftrag wurde kein GitHub-Release veröffentlicht. Die früheren Hinweise „nicht getestet/gebaut“ darunter beschreiben historische Zwischenstände. Aktuelle Ergebnisse und verbleibende Grenzen stehen oben in TEST_MATRIX.md.

Enthalten sind die gemeinsame Titel-/Menüleiste, der separate Projektbereich unter Ansicht, das Canvas-Bildstudio mit eigenen Auflösungen, automatische Modellvorprüfung und lokale 18+-Metadaten sowie die Bibliothek mit getrennten Modellen, Erweiterungen, technischen Komponenten und unbekannten Funden.

Bei der Abnahme behoben: ein JSX-Syntaxfehler in der Studio-Tastaturnavigation sowie durch native HTML-Dialoge blockierte Menü-/Fensterknöpfe. Die unveränderte Chrome-DOM wird während modaler Dialoge in den aktiven Dialog versetzt; Titelhöhe und Menükoordinaten bleiben auch bei 125 % UI-Skalierung erhalten. Die Menüs verwenden weiterhin native Windows-Popups und die bestehenden Aktionen. Die Testautomation wurde auf Popup-Tastennavigation, Datei-Menü-Wiederherstellung und einklappbare Referenzeinstellungen angepasst.

**Eingeschränkte Abnahme:** Die echte SDXL-Generierung und damit der vollständige native Sperrtest sind auf diesem Rechner aktuell durch `resource_memory` blockiert. Der Ressourcenwächter wurde nicht gelockert. Der lokale Build ist vorhanden, aber eine vollständige Freigabe der Generierungs-/Sperrstrecke wird nicht behauptet. Installer-/ZIP-Byteprüfung und native Funktionsabnahme werden getrennt dokumentiert.

# Historischer Arbeitsstand: Bibliothek nach Einsatzzweck (2026-09-19)

Quellcode umgesetzt, **nicht getestet oder gebaut**. Die Nutzersperre für Tests, Checks, App-Start und Build gilt weiter.

- Bibliotheksnavigation: Alle Modelle, Bild, Video, Sprache, Audio & Musik, Erweiterungen, Technische Komponenten, Nicht eindeutig erkannt, ausgeblendete Funde. Standardsicht enthält keine erkannten technischen Komponenten oder Erweiterungen. Suche nach Name/Familie/Format mit passenden Zählern und Pagination; Wartung/Updates/Benchmarks einklappbar.
- Neues optionales typisiertes Profil in bestehenden lokalen Modelleinträgen, rückwärtskompatibel beim Lesen. Rolle, Zweck, Familie, Basisfamilie für deklarierte LoRAs, Verpackung, Erkennungsgrund, Unterstützungsstatus und SDXL-Bestandteile. Alte Einträge werden beim Lesen beschrieben, kein Neuscan nötig.
- Passive Safetensors-Strukturerkennung (u. a. SDXL, LoRA, ControlNet, VAE, CLIP, T5, Transformer); begrenzte config.json-/modelspec-Metadaten und GGUF-Architekturmetadaten. Dateinamen/übliche Modellordner nur als ausdrücklich vorläufige Hinweise. GGUF wird nicht pauschal als Sprachmodell behandelt. Unbekannte Architekturen bleiben sichtbar.
- Begrenzter Profilcache, abhängig von Pfad, Status, erfassten Dateigrößen/-zeiten und config.json; 64-MiB-Lesebudget pro Snapshot, größere Bibliotheken werden schrittweise beschrieben. Kein Modellcode und keine Downloads. Die Profile sind Hinweise und ersetzen niemals die separaten Integritäts-/Runtime-/Zulassungsprüfungen.
- Studio-Aktion nur bei strukturell geprüftem vollständigem SDXL-Checkpoint. SDXL-UNet/CLIP-L/CLIP-G/VAE werden als eingebettet zugeordnet und vom bestehenden Worker verwendet. Fehlende eingebettete Teile werden konkret aufgelistet. Lose externe Komponenten werden weiterhin **nicht** zu einer vermeintlich ausführbaren Pipeline verbunden; dafür fehlt der passende Adapter. FLUX/Wan/Z-Image und Erweiterungen bleiben als nicht unterstützt bzw. bei reinen Hinweisen als ungeklärt sichtbar. LLM-GGUF verweist auf den separaten Assistentenimport, ohne Ausführbarkeit zu behaupten.

Beschreibung und Grenzen: docs/MODEL_LIBRARY.md. Frühere Einträge darunter dokumentieren Zwischenstände.

# Noch nicht veröffentlicht: Hilfskomponenten in der Modellbibliothek (2026-09-19)

Quellcodeänderung, nicht getestet/gebaut: Standardfilter „Modelle“ blendet erkannte Hilfskomponenten aus; eigener Filter „Hilfskomponenten · Encoder, VAE, LoRA“. Komponenten bleiben verwaltbar, haben aber keinen direkten Bildstudio-Startknopf. Bestehende LoRA-Strukturerkennung ergänzt um bekannte TAESD/TAEF-Dateinamen, VAE/T5-Namen und etablierte models-Unterordner (VAE, Textencoder, LoRA, ControlNet etc.). Gruppierung greift auch bei alten Bibliothekseinträgen ohne Neuscan. Keine vollständige semantische Architekturerkennung; unbekannte Komponenten können noch als Modelle erscheinen. Dateien bleiben erhalten.

# Noch nicht veröffentlicht: Bildstudio mit Canvas (2026-09-19)

Quellcode umgesetzt, **noch nicht getestet oder gebaut**. Nutzeranweisung gegen Tests/Builds bleibt bestehen; veröffentlichte Version weiterhin 0.26.0.

- Bildstudio: seitliche Modell-/Prompt-/Formatsteuerung, große Canvas-Ergebnisansicht mit Einpassen/Zoom/100 %/Ziehen, echter Auftragsfortschritt und horizontaler Verlauf. Technische Informationen, Referenz/Inpainting und erweiterte Parameter einklappbar. Vorhandene Speicher-/Projekt-/Abbruch-/Wiederherstellungsaktionen erhalten. Studio-Navigation mit Symbolen und Pfeiltasten/Home/End; Theme und schmale Fenster berücksichtigt, noch nicht visuell geprüft.
- Neuer typisierter, nur lokal für main erlaubter IPC `image_model_catalog`: vollständige SDXL-Tensorstruktur erforderlich, nicht nur Dateiendung. Komponenten/LoRAs/andere Familien bleiben im allgemeinen Bestand. Dateibindungsabhängiger, begrenzter Kompatibilitätscache ist nur eine Auswahlhilfe, keine Sicherheitsfreigabe.
- Modellwahl löst Vorprüfung automatisch aus; positive/negative UI-Auskunft bleibt bis zu 60 Sekunden im Speicher. Manueller Recheck in Details; Queue-Zulassung und Integritäts-/Runtimeprüfungen bleiben unverändert verbindlich. Windows-Pfadpräfix nur für die Darstellung entfernt; interne Dateireferenzen erhalten.
- Eigene SDXL-Abmessungen: 256–2048 je Seite, Vielfache von 64, höchstens 2.097.152 Pixel. Zahlenfelder, echte Formatvorlagen und Seitenwechsel; identische Grenzen im Backend, auch PNG-Referenzen/Masken. Mehr Speicherbedarf möglich, keine automatische Qualitätsreduktion und keine Zusage für andere Adapter.
- Lokale automatische 18+-Zuordnung anhand expliziter Safetensors-Metadaten und begrenzter JSON-/Civitai-Begleitdateien. Keine Onlineabfrage, kein Gewichtscode, keine Ableitung aus Prompts/Trainingscaptions oder pauschal aus Modellnamen. Fehlende Angaben sind unbekannt, keine SFW-Garantie. Manuelle Korrektur nur entsperrt, gespeichert mit Dateiidentität; bekannte geschützte Modelle bleiben geschützt. Vorhandene HF-Downloadtags bleiben berücksichtigt.

Weitere Abnahme und konkrete offene Prüffälle: TEST_MATRIX.md. Frühere ungetestete Titel-/Menüleistenänderungen bleiben enthalten.

# Unveröffentlichter Arbeitsstand: gemeinsame Titel-/Menüleiste

Nutzeranweisung vom 19.09.2026: **keine Tests und keinen Build bis zur erneuten ausdrücklichen Aufforderung**. Deshalb nur Quellcode geändert; keine neue Version, kein Installer, kein Paket und keine Veröffentlichung erstellt. Der zuletzt geprüfte und veröffentlichte Stand bleibt 0.26.0.

Implementiert, noch nicht ausgeführt/verifiziert: rahmenlose Windows-Hauptansicht mit gemeinsamer Titel-/Menüleiste, App-Logo links, Datei/Bearbeiten/Ansicht/Hilfe daneben, zentriertem „Projektname – Local Studio“ beziehungsweise „Local Studio“ und Fensterknöpfen rechts. Native Popupmenüs verwenden dieselben vorhandenen Aktionen; Windows-/Taskleisten-Titel, Sprach-/Themewechsel, Stern für ungespeicherte Änderungen, Tastaturzugriff und Ziehen/Doppelklick sind angeschlossen. Gesperrte Projekte geben ihren Namen nicht im Titel aus.

Die dauerhaft sichtbare Projektleiste entfällt. Erstellen/Öffnen/Speichern bleiben im Datei-Menü. Projektmedien, Umbenennen, Modellzuordnung und Wiederherstellung sind bedarfsweise über **Ansicht → Projektmedien und Details** erreichbar. Der bisherige Arbeitsbereich bleibt erhalten und ist über „Zurück zum Arbeitsbereich“ wieder sichtbar. Fehlermeldungen und Recovery-Hinweise bleiben auch bei geschlossener Detailansicht erreichbar. Eingebetteter HF-Browser wird während der Detailansicht ausgeblendet.

Ausstehende Verifikation steht oben in TEST_MATRIX.md. Gesamter SPEC-Umfang bleibt erhalten.

# Aktueller Stand 0.26.0: lokale 18+-Sperre

Der zusätzliche Nutzerauftrag zur 18+-Sperre ist implementiert. Der gesamte Masterprompt bleibt weiterhin in Arbeit. Bedienung und Grenzen stehen in [docs/PRIVACY.md](docs/PRIVACY.md); tatsächliche Prüfungen separat in TEST_MATRIX.md. [Version 0.26.0 mit Installer und portabler ZIP](https://github.com/ScopeBotlul/Local-Studio/releases/tag/v0.26.0) ist veröffentlicht und öffentlich verifiziert. Die vorherige 0.25.0 erkennt das signierte Update; die neue Version meldet den Kanal als aktuell. Bei portablen Updates alle vier Runtime-Ordner ersetzen und Local-Studio-Data vollständig behalten.

Standardmäßig ausgeschalteter Modus mit Altersbestätigung, lokaler PIN oder Passwort, gesalzener Ableitung und persistenter Versuchsbremse. Neustartsperre standardmäßig aktiv, optionales Merken, Passwortwechsel und manuelles Sperren/Ausschalten. Keine Klartext-Zugangsdaten. Lokale Modellkennzeichnung und Übernahme eindeutiger HF-Tags bei Downloadplanung; Kennzeichnung bleibt beim App-Modellumzug erhalten.

Gesperrte Bildarbeitsbereiche, Aufträge, Galerieeinträge, Suchresultate, Details und Protokollzugriffe liefern keine privaten Vorschauen oder Eingaben an die Oberfläche. Bereits laufende Generierungen bleiben aktiv. Herkunft bleibt bei App-Kopien, Editorvarianten, Kompositionen und Exporten erhalten; keine manuelle Kennzeichnung gewöhnlicher Medien. Projekte mit privatem Inhalt werden als Ganzes geschützt und im Format 5 transportiert. Einfache Chats bleiben nutzbar; gekennzeichnete Chatmodelle und betroffene Verläufe sind geschützt. Backend-Prüfung auch für direkte IPC und Medienprotokolle; verspätete Leseantworten werden bei einer Sperrung ungültig.

Die Sperre verschlüsselt keine Datenträgerdateien und ist keine Inhalts- oder Altersprüfung. Gemischte ältere Verwaltungs- und Verlaufsansichten verlangen vorsorglich eine Entsperrung, sobald geschützte Daten existieren. Noch nicht vorhandene Erweiterungsadapter, visuelle Analyse und semantischer Index bleiben offen; ihre spätere Einbindung in den Schutz ist Teil der unveränderten SPEC. Alle weiteren Meilensteine bleiben erhalten.

## Vorheriger Stand 0.25.0


## Arbeitsstand 0.25.0 – Gesamtauftrag weiterhin in Arbeit

Der Nutzerauftrag umfasst den gesamten verbleibenden Masterprompt. **Noch keine vollständige Umsetzung oder Abnahme.** Der unveränderte Umfang ist in [REQUIREMENTS_STATUS.md](REQUIREMENTS_STATUS.md) nachvollziehbar; alle offenen Meilensteine bleiben erhalten. [0.25.0 mit Installer und portabler ZIP](https://github.com/ScopeBotlul/Local-Studio/releases/tag/v0.25.0) ist veröffentlicht und öffentlich verifiziert. Die vorherige 0.24.0 erkennt das signierte Update, die neue Version meldet den öffentlichen Kanal als aktuell. Bei portablen Updates alle vier Runtime-Ordner ersetzen und Local-Studio-Data behalten.

Implementiert: gemeinsame begrenzte Ressourcenvergabe für Bild, Chat/Laden, Transkription, Videoexport und Medienvorbereitung; optional parallele CPU/GPU-Verarbeitung; optionale echte Live-Hardwarewerte, Windows-Akzentfarbe und Weiterlaufen im Infobereich. Bestätigtes Verschieben erfasster Modelldateien mit SHA-256-Kopierprüfung, exklusiven Handles, Abbruch, Konfliktschutz und nachgelagerter Entfernung der Originale; Chat-/Speech-Registrierungen werden nach erfolgreicher Prüfung umgestellt. Alte Projekt-/Studio-Modellpfade müssen bisher anschließend neu ausgewählt werden. HF-Cache-Verknüpfungen und nicht erfasste Begleitdateien werden nicht automatisch migriert.

Modellupdates: abschaltbare automatische Metadatenprüfung für abgeschlossene HF-Downloads gegen main, Vergleich der tatsächlich ausgewählten Dateien, fehlende Dateien als Prüfbedarf; bewusste gemeinsame Downloadvorschau und Queue-Start. Neue Revisionen erhalten eigene Ordner, alte Dateien bleiben erhalten. Keine automatische Installation oder Modellauswahl; externe unbekannte Quellen werden nicht erfunden. Bis 100 Varianten pro Prüfung und 16 gemeinsam vorbereitete Updates.

Bildgenerierungen schreiben lokale Messwerte (Modellhash, Parameter, Laufzeit ohne Ressourcenwartezeit, Windows-Worker-Spitzenspeicher). Unbekannte VRAM-Spitzen bleiben null. Separate lokale gelernte Modell-/Parameterpräferenzen, Bearbeiten, einzelnes Löschen und Reset; explizite aktuelle Nutzerwahl hat Vorrang. Der Assistent erhält nur begrenzte Lese-Tools für Messwerte und Präferenzen.

SDXL-Bild-zu-Bild mit geprüfter eigener Referenzkopie pro Auftrag, Stärke und Hashbindung; zunächst statische 8-Bit-PNG mit 512/768/1024 Pixeln je Achse. Projektformat 4 bettet Referenzbilder und Masken ein und stellt ihren Pfad nach Öffnen auf die Projektkopie um. Alte Formate bleiben lesbar; Referenzprojekte werden nicht als mit älteren Builds kompatibel ausgegeben. Maskiertes Inpainting erhält schwarze Bereiche einschließlich Alpha pixelgenau; graue Bereiche werden gewichtet überblendet. Stapel mit 1–20 Bildern und expliziter Seed-Folge, wählbare CPU-VAE und tatsächlich gemessene Sampling-Schrittzahl. Maskenexport nutzt denselben Ebenenrenderer; Galerie-PNGs können direkt als Referenz übernommen werden. Aufträge speichern eigene Eingabepfade; aktive oder pro Modell gemerkte Eingaben sind vor Temporärbereinigung geschützt. Weitere Eingabeformate und übrige SPEC-Funktionen bleiben offen. Zusätzlich: kleine Maskenvorschauen behalten ihre exakten Pixel; kanonische Windows-Pfade werden bei Galerieübergabe korrekt verbunden. Der lokale OAuth-Rückruf toleriert einzelne Read-Timeouts innerhalb seiner begrenzten Headerfrist und bleibt gegen falschen State geschützt. Tatsächliche Prüfungen stehen ausschließlich in TEST_MATRIX.md.


## Weiterhin enthalten: 0.24.0

- Echter lokaler DE/EN-Chat mit Qwen-GGUF und llama.cpp auf CPU. Modellauswahl, explizites Laden/Entladen, Abbruch, persistenter Verlauf und bestätigtes Leeren. Nach Neustart keine automatische Modellaktivierung.
- Vier fest definierte Werkzeuge: tatsächliche Hardware, vorhandene Modellbibliothek, Galerie-Metadaten und Vorbereitung geprüfter SDXL-Parameter. Explizite Übernahme ins Bild-Studio, Generierung separat. Keine Shell, freien Skripte oder autonomen Downloads.
- Lokale Whisper-Transkription des ausgewählten Audio-/Videoclips; Zeiten berücksichtigen Ausschnitt, Clipstart und Geschwindigkeit. Text-/Zeitprüfung, Übernahme als ein Undo-Schritt, SRT und Speicherung im Projekt. Fortschritt/Abbruch und persistente Aufträge.
- Gemeinsamer vorhandener Downloadmanager mit bewusstem Start, Katalogrevision/Größe/SHA-256, Import eigener Modelle und tatsächlicher Dateiprüfung vor Nutzung. Separate CPU-Prozesse mit Windows-Jobobjekten, Speichergrenzen und Beendigung mit dem Elternprozess. Installer/Portable enthalten jetzt vier gepinnte Runtimes mit Lizenzhinweisen; keine Modellgewichte beigepackt.

[Release 0.24.0 mit Installer und portabler ZIP](https://github.com/ScopeBotlul/Local-Studio/releases/tag/v0.24.0) ist veröffentlicht und öffentlich verifiziert. Die vorherige 0.23.0 erkennt das signierte Update; die neue Version meldet denselben Kanal als aktuell. Portable: alle vier Runtime-Ordner ersetzen und Local-Studio-Data behalten.

[Einrichtung, Bedienung und Grenzen](docs/ASSISTANT.md). Prüfnachweise separat in [TEST_MATRIX.md](TEST_MATRIX.md). M5/M6/M8 bleiben teilweise umgesetzt: kein visueller Inhaltsindex, keine Diarization/KI-Videoerzeugung, keine Online-Modellberatung oder gelernten Präferenzen. Kleine Modelle können falsche Antworten/Transkripte liefern. CPU ist der erste Ausführungspfad; vollständige Ressourcenkoordination und 18+-Schutz bleiben offene SPEC-Anforderungen. Unübernommene Transkript-Korrekturen sind nur im aktuellen UI-Entwurf gespeichert. Der gesamte zukünftige Umfang bleibt erhalten.

## Vorheriger Stand 0.23.0: direkter Schnitt und Medienvorbereitung

- Einzelbilder am Abspielkopf aus echten Originalen oder geprüften Proxies, einschließlich Effekten, Keyframes und Untertiteln. Veraltete Anfragen werden abgebrochen; die bisherige gerenderte Ton-/Videowiedergabe bleibt verfügbar.
- Lokale, abbrechbare Proxy-Erzeugung (automatisch/halb/viertel/eigene Breite) und Audio-Wellenformen aus dekodiertem Ton. Persistente Caches, Status unter Aufträge, Originale für sämtliche Exporte.
- Mausgriffe an Clipkanten, Strg/Umschalt-Mehrfachauswahl, gemeinsame Bewegung/Duplizieren/Entfernen, Snapping und ein Undo pro Geste. Sperren und Quellzeit-Zuordnung bleiben erhalten.
- Bildeditor: Skalier-/Drehgriffe, Pinselkontur; beide Editoren übernehmen vorhandene Projektmedien ohne erneute Quellkopie.
- Bestehende Speicherbereinigung erfasst registrierte Render-, Proxy- und Wellenform-Dateien. Aktive Projektcaches, laufende Aufträge, Originale und Galerieausgaben geschützt; Identität, Größe und SHA-256 vor Löschung geprüft. Hauptfenster-exklusive typisierte IPC und isolierte FFmpeg-Prozesse.

[Release 0.23.0 mit Installer und portabler ZIP](https://github.com/ScopeBotlul/Local-Studio/releases/tag/v0.23.0) veröffentlicht und öffentlich verifiziert. Die vorherige 0.22.0 erkennt das signierte Update; die neue Version meldet denselben Kanal als aktuell. Beim portablen Update Local-Studio-Data behalten. [Bedienung und Grenzen](docs/CREATIVE_EDITORS.md), Prüfnachweise separat in [TEST_MATRIX.md](TEST_MATRIX.md). Einzelbildvorschau ist keine garantierte Echtzeitwiedergabe; Proxies dienen dieser Vorschau. Keine automatische Transkription, KI-Videoerzeugung, HDR/ICC-Abnahme oder vollständige M4/M8-Abnahme. Unregistrierte Altcaches/Absturzreste werden nicht pauschal gelöscht. Übriger SPEC-Umfang bleibt erhalten.

## Vorheriger Stand 0.22.0: Ebenen/Masken und erste Video-Timeline

- Drei Studio-Bereiche: Bildgenerierung, Ebeneneditor, Videoschnitt. Lokaler gemeinsamer Arbeitsstand mit Undo/Redo, Projektformat 3, Originalschutz und expliziter Recovery.
- Echte Ebenenkomposition mit Maskenpinsel/Rechteck/Ellipse, Gruppenauswahl, Sperre, freier Transformation, Ausrichtung, Mischmodi und relativen Korrekturen; PNG/JPEG-Export und Vorschau.
- Klassischer lokaler Video-/Audio-Mehrspurschnitt mit Trimmen/Teilen, Geschwindigkeit, Fades, linearen Keyframes, Effekten und manuellen Untertiteln/SRT. Echte FFmpeg-Vorschau sowie MP4/H.264/AAC und WebM/VP9/Opus; begrenzte, abbrechbare isolierte Prozesse und unveränderliche Renderaufträge.
- Gepinnte separate Video-Runtime mit Integritätsprüfung, Entwickler-Bootstrap und Lizenz-/Quellhinweisen; in Installer und Portable enthalten. Typisierte Hauptfenster-IPC; keine frei übergebenen Filter/Shellbefehle.

[Release 0.22.0 mit Installer und portabler ZIP](https://github.com/ScopeBotlul/Local-Studio/releases/tag/v0.22.0) veröffentlicht und öffentlich verifiziert. Update über Hilfe; portable Programmdateien einschließlich beider Runtime-Ordner ersetzen und Local-Studio-Data behalten.

[Bedienung und Grenzen](docs/CREATIVE_EDITORS.md). Implementierung hier, Nachweise in [TEST_MATRIX.md](TEST_MATRIX.md). Erste 8-Bit-Pfade: kein RAW/HDR-Farbmanagement, keine KI-Videoerzeugung, keine automatische Transkription/Proxies/Tracking. Video-Renderordner werden noch nicht automatisch bereinigt. M4 und M8 sind teilweise umgesetzt, der gesamte weitere SPEC-Umfang bleibt erhalten.

## Vorheriger Stand 0.21.0: Bildkorrekturen, bearbeitbare Projekte und Stapelverarbeitung

- Helligkeit, Kontrast, Sättigung und relative Farbtemperatur von −100 bis +100 im Bildeditor. Angewendete Korrekturen sind rückgängig/wiederholbar; Vorher/Nachher zeigt Original und aktuellen Stand nebeneinander. Vorschau und Export verwenden dieselbe begrenzte Rust-Verarbeitung.
- **Bearbeitung ins Projekt übernehmen** kopiert das Original samt aktiven Bearbeitungsschritten in den lokalen Projektarbeitsstand. Weitere Übernahmen aus demselben geöffneten Editor aktualisieren diesen Eintrag. Nach Schließen des Editors schreibt Projekt speichern/Strg+S die transportable Datei. Projektmedien lassen sich direkt im Editor weiterbearbeiten und als neue Galerievarianten exportieren, auch ohne ursprüngliche Galeriedatei.
- Projektcontainer mit Rezepten verwenden Version 2; Projekte ohne Rezepte bleiben Version 1. Alte Anwendungen weisen Version 2 ausdrücklich zurück. Quellen werden gegen SHA-256 geprüft, veraltete Rezeptänderungen abgewiesen. Rezepte enthalten ausschließlich typisierte passive Operationen.
- Galerie-Mehrfachauswahl → **Bilder korrigieren …**: gemeinsame Korrekturen, Vorschau des ersten Bildes, PNG/JPEG und Qualität, sequentielle Verarbeitung, Fortschritt und Einzelfehler. Abbruch endet nach dem aktuellen Bild; fertige Varianten bleiben erhalten, Originale werden nicht überschrieben. App-Beenden wartet auf das aktuelle Bild und startet kein weiteres.

[Release 0.21.0 mit Installer und portabler ZIP](https://github.com/ScopeBotlul/Local-Studio/releases/tag/v0.21.0) veröffentlicht. Ab 0.20.0 über Hilfe → Nach Updates suchen erreichbar; portable Programmdateien manuell ersetzen und Local-Studio-Data behalten. [Bildeditor](docs/IMAGE_EDITOR.md), [Projekte](docs/PROJECTS.md). Implementierung hier; tatsächliche Prüfnachweise separat in [TEST_MATRIX.md](TEST_MATRIX.md). Relative 8-Bit-RGB-Korrekturen sind keine Kelvin-Weißabgleich-/ICC-Farbmanagementlösung. Lokaler Undo/Redo-Entwurf bleibt getrennt vom transportierten aktiven Rezept. Keine automatische Fortsetzung abgebrochener Stapel. Ebenen, Masken, RAW/HDR, KI-Bearbeitung und übrige M3–M12-Anforderungen bleiben offen.

## Vorheriger Stand 0.20.0: Menüleiste und signierte GitHub-Updates

- Native Windows-Menüleiste Datei / Bearbeiten / Ansicht / Hilfe; Sprache, Theme und sichtbare Tastenkürzel folgen den Einstellungen. Projektaktionen verwenden den vorhandenen Controller einschließlich Konflikten, Recovery und gemeinsamen Speicherbestätigungen. Zuletzt geöffnete Projekte als Untermenü. Bildansicht und Undo/Redo arbeiten im aktiven Galerie-/Editorkontext; während anderer Dialoge sind unpassende Aktionen deaktiviert.
- Lokale Anleitung und Versionsdialog. Die bisherige Projektleiste bleibt für Status, Medien und Details erhalten; häufige Dateiaktionen stehen zentral im Menü. Neue Projektkürzel sind in Einstellungen anpassbar.
- Abschaltbare automatische Updateprüfung einmal nach dem Start und manuelle Prüfung unter Hilfe. Fester öffentlicher GitHub-Kanal, Ed25519-signiertes Manifest, exakte Repository-/Tag-/Asset-URLs, begrenzte Größen und SHA-256. Manipulierte Signaturen/Pakete und Downgrades werden abgewiesen. Typisierte IPC ausschließlich im lokalen Hauptfenster.
- Installierte Ausgabe: bewusster Download mit Fortschritt/Abbruch; anschließend bestätigter Neustart über den vorhandenen Beenden-/Speicherpfad. Kopierter Helfer wartet auf das Prozessende, prüft Paket und Originalziel erneut, führt den Inno-Installer aus und startet die App am bisherigen Ort. Portable erkennt Releases und öffnet den manuellen Download, installiert aber selbst nichts.
- Lokales Signierwerkzeug; privater Schlüssel durch Windows DPAPI geschützt und aus Git/Release ausgeschlossen. Öffentlicher Schlüssel in der App. Installer kennzeichnet die installierte Distribution. Medien, Prompts, Modelle und App-Daten werden nicht übertragen oder in Pakete aufgenommen.

[Bedienung, Grenzen und Release-Verfahren](docs/UPDATES.md). Erste Nutzung: ältere Builds einmal manuell durch 0.20.0 ersetzen. **Local-Studio-Data behalten.** [GitHub-Downloads](https://github.com/ScopeBotlul/Local-Studio/releases/latest). Implementierung und Verifikation sind getrennt; Prüfnachweise in [TEST_MATRIX.md](TEST_MATRIX.md).

Grenzen: kein Resume abgebrochener Update-Downloads, keine automatische Bereinigung alter Updateordner und kein Windows-Authenticode-Zertifikat. Installationsfehler stehen im lokalen Updateprotokoll; der Helfer startet nach einem fehlgeschlagenen Installationsversuch die bestehende App erneut, soweit diese noch vorhanden ist. Zweit-PC-, umfassende Accessibility- und Langzeitabnahme bleiben Teil von M12; alle übrigen offenen Meilensteine bleiben erhalten.

## Vorheriger Stand 0.19.0: erster Bildeditor

- Galerie → **Bild bearbeiten** öffnet einen deckenden Editor mit großer Vorschau. 90° links/rechts, horizontal/vertikal spiegeln, rechteckiger Zuschnitt per Ziehen oder Pixelwerten, Resize mit optionalem Seitenverhältnis. Gemeinsame begrenzte Rust-Transformation für Vorschau und Export auf einem Hintergrundthread.
- Undo/Redo mit einstellbarem Verlaufslimit, Standard Strg+Z/Y und zentral änderbaren Kürzeln. Verzweigte Bearbeitungen entfernen den verworfenen Redo-Zweig. Mausradzoom, mittlere Maustaste zum Verschieben, Einpassen und Ansicht in Bildpixelgröße.
- Angewendete Rezepte bleiben als lokale Entwürfe an die genaue Quelle gebunden; nach Neustart ausdrücklich fortsetzen oder verwerfen. Editor-Schließen schützt noch nicht exportierte Änderungen. Gemeinsamer App-Beenden-Dialog berücksichtigt den Entwurf und wartet auf laufende Verarbeitung.
- PNG-/JPEG-Export als neue Datei im Originalordner; kein Überschreiben. PNG-Alpha, JPEG-Qualität und weißer Transparenzhintergrund. Quelle bleibt unverändert. Herkunft und Bearbeitungsschritte werden vor Veröffentlichung in SQLite registriert; Varianten bleiben eigenständige Medien.
- Statische 8-Bit-PNG/JPEG/BMP, begrenzte Dateigröße/Pixel/Schritte/Arbeitsmenge. EXIF-Orientierung wird eingerechnet, EXIF/GPS entfernt, RGB-ICC-Profil bytegleich erhalten; andere Profile/Bit-Tiefen und animiertes PNG werden ausdrücklich abgewiesen. Haupt-WebView-exklusive typisierte IPC, Pfad-/Versions-/Identitätsprüfung und gesperrte Quell-/Elternpfade.

[Bedienung und Grenzen](docs/IMAGE_EDITOR.md). M4 bleibt teilweise umgesetzt: Ebenen/Masken, Text/Vektor, RAW/HDR, umfassendes Farbmanagement, KI-Bearbeitung, Batch und transportable Projektrezepte bleiben offen. Entwürfe liegen lokal im WebView-Profil, nicht im Projektcontainer; externe Umbenennung/Pfadwechsel werden noch nicht für Entwürfe migriert. Vorschau maximal 1.600 Pixel; Export volle eingestellte Auflösung.

[Portable ZIP](releases/Local-Studio-0.19.0-hub-portable.zip), [Installer](releases/Local-Studio-0.19.0-hub-setup.exe). Verifikation separat in [TEST_MATRIX.md](TEST_MATRIX.md). **Local-Studio-Data beim Update behalten.**

## Vorheriger Stand 0.18.0: Video-Miniaturen, Herkunft und Projektzugriff

- Lokale Video-Miniaturen aus einem tatsächlich decodierten Frame, höchstens 320 Pixel, über die vorhandenen WebView/Windows-Codecs. Nur sichtbare Karten, begrenzte Parallelität, Zeitlimit und Abbruch. Der vorhandene Cache prüft Quellversion und PNG-Grenzen; Originalmedien bleiben unverändert. Kein zusätzlicher Video-Runtime-Download.
- Echte rekursive Windows-Ordnerbenachrichtigungen aktualisieren die Galerie mit kurzem Debouncing. 30-Sekunden-Abgleich ergänzt den Watcher; bei Ausfall greift die bisherige 3-Sekunden-Prüfung. Änderungen während einer Auswahlaktualisierung sperren Karten sichtbar, statt Klicks still zu ignorieren.
- **Kopie als Variante anlegen** erstellt eine separate geprüfte Datei und speichert ihre tatsächliche Quelle/Versionsgruppe. Herkunftskette mit Original/Import, bekannter lokaler Generierung und Kopie; vorhandene Modell-/Prompt-/Auftragsdaten bleiben nachvollziehbar. Neueste verfügbare Version als Standard-Hauptversion, bewusste andere Hauptversion und direkte Navigation. Umbenennen wird über Dateiidentität erkannt; fehlende/geänderte/endgültig gelöschte Quellen bleiben reine Metadateneinträge ohne versteckte Medienkopien. Andere Varianten werden nicht mitgelöscht.
- Bis zu zwölf zuletzt erfolgreich geöffnete/gespeicherte Projektdateien, nicht erreichbare Dateien gekennzeichnet, Listeneintrag unabhängig von Datei entfernbar. Installer registriert `.localstudio` im Benutzerbereich ohne eine bestehende andere Standardzuordnung zu überschreiben; Deinstallation räumt eigene Zuordnungen auf. Portable Ausgabe registriert nichts.
- Dateiargumente werden sowohl beim Kaltstart als auch über eine bereits laufende Instanz angenommen. Genau eine Projektdatei, keine Ausführung anderer Argumente. Die bestehende Archivprüfung und Bestätigung ungespeicherter Änderungen bleiben wirksam. Begrenzte Öffnungswarteschlange wartet während Dialogen; portable Installationsordner bleiben voneinander isoliert.

[Portable ZIP](releases/Local-Studio-0.18.0-hub-portable.zip), [Installer](releases/Local-Studio-0.18.0-hub-setup.exe). **Local-Studio-Data beim Update behalten.** [Galerie](docs/GALLERY.md), [Projekte](docs/PROJECTS.md), [Installer](docs/INSTALLER.md). Verifikation separat in [TEST_MATRIX.md](TEST_MATRIX.md).

Grenzen: Video-Codecs hängen von Windows/WebView ab; unlesbare Clips behalten Symbolkarten. Herkunft wird aus bekannten Aufträgen und bewussten Variantenkopien erfasst, nicht aus Bildähnlichkeit erfunden. Das Raster zeigt weiterhin einzelne echte Dateien; automatische Gruppenkarten, weitere Editor-/KI-Ableitungen und Transport zusätzlicher Galeriebeziehungen in Projektcontainern bleiben offen. Externe Identitätssuche ist begrenzt. M3 ist nicht vollständig abgenommen; Bildeditor und M4–M12 bleiben erhalten.

## Vorheriger Stand 0.17.0

### 0.17.0: Medienübernahme, Recovery und Bereinigung

- Studio-Ergebnisse und aktuelle Galerieauswahl direkt ins Projekt kopieren; ohne aktives Projekt wird ein unbenanntes angelegt. Galeriequelle wird anhand Wurzel, Dateiidentität und Version geprüft; fertige Bildaufträge anhand Status und gebundener Ausgabe. Originalbytes bleiben erhalten. Modellgewichte werden weiterhin nicht eingebettet.
- Native Datei-Drops auf Projektleiste und Galerie. Ziel anhand tatsächlicher Position und Windows-Skalierung; während Dialogen/gesperrten Aktionen blockiert. Ein nachfolgendes Drag-Leave verwirft keine schon eingegangene Drop-Aktion. Galerie verwendet den bestehenden Kopierimport mit Teilergebnis; Projektimport bleibt begrenzt.
- Projekte umbenennen, entfernte Projektmedien lokal zurückholen (Standard 100, einstellbar bis 1.000); gespeicherter Container enthält nur aktive Medien. Bis zu 20 lokale Recovery-Stände über alle Projekte, automatische Vorgänger und explizite Wiederherstellungspunkte. Wiederherstellen prüft Medienhashes und schreibt die Projektdatei erst beim nächsten Speichern.
- Speicherübersicht für bekannte Projektarbeitskopien und temporäre Bildaufträge: konkrete Pfade, freigebbare/geschützte Bytes, Bestätigung, sichtbare Teilfehler. Kurzlebige einmal verwendbare Vorschau; Schutzstatus, Pfad und gebundene Dateiidentität werden direkt vor dem Löschen erneut geprüft. Keine rekursive Ordnerlöschung.
- Automatische Bereinigung standardmäßig aktiv, sieben Tage Aufbewahrung (1–365), nach Start und höchstens stündlich. Aktives Projekt, letzte drei Recovery-Stände, jüngere Stände, laufende/pausierte Aufträge und ungespeicherte fertige Bilder geschützt. Temporäre PNG-Duplikate benötigen eine unveränderte byteidentische Galerieausgabe. Modelle, Downloads, Galerieoriginale, Papierkorb und unbekannte Dateien bleiben erhalten.
- Typisierte IPC und Berechtigungen ausschließlich für die lokale Haupt-WebView. Bestehende Projekte bleiben Formatversion 1; alte Einstellungen erhalten den Cleanup-Standard, individuelle Werte bleiben erhalten.

[Portable ZIP](releases/Local-Studio-0.17.0-hub-portable.zip), [Installer](releases/Local-Studio-0.17.0-hub-setup.exe). **Local-Studio-Data beim Update behalten.** Bedienung und Grenzen: [Projekte](docs/PROJECTS.md), [Einstellungen](docs/SETTINGS.md). Prüfung separat in [TEST_MATRIX.md](TEST_MATRIX.md).

Grenzen: Speicherübersicht ist keine gesamte Laufwerksanalyse. Unregistrierte alte Arbeitskopien werden nicht pauschal entfernt; leere Arbeitsordner können verbleiben. Bis zu 1.000 Dateien pro Bereinigung. Recovery-Historie ist kein allgemeines Editor-Undo/Redo und direkte Übernahme überträgt noch keine zusätzlichen Herkunfts-/Tagbeziehungen. Watcher, Video-Thumbnails, Dateiassoziation, vollständige Modellreferenzen, Zweit-PC-Abnahme und M4–M12 bleiben offen.

## Vorheriger Stand 0.16.0

### 0.16.0: Lokale Projektdateien

- Echte `.localstudio`-Container mit aktuellen Bild-Studio-Eingaben und explizit eingebetteten Bild-/Video-/Audiomedien. Anlegen, Öffnen, Speichern, Speichern unter, Schließen, Strg+S und dessen zentrale Umbelegung. Genau ein aktives Projekt; projektloses Arbeiten bleibt möglich.
- Lokale Arbeitskopie im konfigurierten Recovery-Ordner, SQLite-Zustand und ausdrückliches Fortsetzen nach Neustart. Gemeinsamer Beenden-Dialog mit Projekt-Speicheroption. Der Projekte-Pfad dient als Vorgabe für Save As.
- Modellreferenz per Dateiname/SHA-256, keine eingebetteten Gewichte oder absoluten Modellpfade im Container. Originalmodell nach identischer Prüfsumme zuordnen; Öffnen startet weder Downloads noch Generierungen.
- Medienliste und echte Bild-/Video-/Audiovorschau über ein auf Haupt-WebView und aktive Manifest-IDs beschränktes Protokoll. Medien entfernen und Container neu schreiben, Kopien mit ursprünglichen Namen in Galerie exportieren; Teilergebnisse sichtbar, Originale unverändert.
- Begrenzte ZIP32-Importe, feste Archivpfade, Größen-/Versions-/Prüfsummenprüfung. Geänderte und fremde Zieldateien werden nicht überschrieben. SQLite-Speicherjournal für Wiederherstellung nach unterbrochenem Dateiaustausch.

[Portable ZIP](releases/Local-Studio-0.16.0-hub-portable.zip), [Installer](releases/Local-Studio-0.16.0-hub-setup.exe).

Umfang, Bedienung und offene Grenzen: [Projekte](docs/PROJECTS.md). Verifikation separat in [TEST_MATRIX.md](TEST_MATRIX.md). M3 und M4–M12 bleiben offen; keine simulierten Medienfunktionen. Recovery-Historie/Bereinigung, Dateiassoziation, Undo, vollständige HF-Modellreferenzen und Editorstände sind weiterhin geplante Arbeit.

## Vorheriger Stand 0.15.0

### 0.15.0: Speicherorte, Tastenkürzel und Bildansicht

- Zehn einzeln einstellbare Speicherpfade mit Standard-Rücksetzung, Schreibprüfung, Pfad-/Overlap-/Junction-Prüfung und kompatibler Migration vorhandener Einstellungen. Galerie, Cache, Downloadplanung und drei Modellordner verwenden die effektiven Pfade. Keine automatische Dateiverschiebung. Projekte, Recovery und Proxies bleiben vorbereitete Ziele für noch offene Arbeitsabläufe; bestehende Datenbanken bleiben im Konfigurationsordner.
- Neue Bildaufträge erhalten einen eigenen unveränderlichen temporären Arbeitsordner. Queue, Recovery, Vorschau, Galerie-Speichern und gezieltes Verwerfen/Papierkorblöschen berücksichtigen alte und neue Pfade. Unbekannte Dateien werden nicht rekursiv entfernt.
- Zentrale, gespeicherte Belegung für elf aktive Aktionen: Bild generieren/einreihen, Galerieauswahl, Umbenennen, Papierkorb, Vergleich, Einpassen/100 %/Rücksetzen und UI-Zoom. Aufnahme per Tastendruck, Konflikt-/Reservierungsprüfung im Frontend und Backend, Entfernen und Gesamtrücksetzung. Eingabefelder/Modale behalten ihre kontextbezogene Bedienung; bestehende Löschbestätigungen gelten weiter.
- Galerie-Bildvorschau mit vollständigem Hochformat-Einpassen, tatsächlicher 100-%-Ansicht unter UI-Skalierung, Zoom und Mausverschiebung. Vergleich übernimmt konfiguriertes Einpassen/Rücksetzen und bleibt ein relativer synchroner Ansichtsvergleich.

[Portable ZIP](releases/Local-Studio-0.15.0-hub-portable.zip), [Installer](releases/Local-Studio-0.15.0-hub-setup.exe). **Local-Studio-Data beim Update behalten.**

Die Gesamtanforderung ist **noch nicht vollständig umgesetzt**. M0 ist wegen Einrichtung/Recovery und weiterer Worker-Abnahme weiter offen; M1–M3 behalten ihre Restpunkte. Projekte, vollständiger Bildeditor, Assistent, Analyse, Video, Audio und Workflows sind keine fertigen Funktionen dieses Builds. [Bedienung und Grenzen](docs/SETTINGS.md). Verifikation separat in [TEST_MATRIX.md](TEST_MATRIX.md).

---

# Aktueller Stand 0.14.0: Bildvergleich und Galeriebedienung

- Vergleich für zwei bewusst ausgewählte Originalbilder: nebeneinander, Wipe mit direkt ziehbarer bzw. per Tastatur bedienbarer Trennlinie, A/B-Tausch, gemeinsamer Zoom bis 800 % relativ zur eingepassten Ansicht und synchroner Ausschnitt per Maus/Tastatur. Deckender modaler Dialog mit Theme und UI-Skalierung; Esc erhält die Galerieauswahl. Keine Änderungen an Originalen.
- Neuer typisierter lokaler `gallery_compare`-Befehl prüft Galeriewurzel, zwei verschiedene Dateiidentitäten, aktuelle Versionen, Bildformat, Dateigröße und Bildmaße. Versionierte Medienadressen prüfen die Quelle am geöffneten Handle erneut. Grenzen: je 32 MiB/32 Megapixel/16.384 Pixel pro Achse. Fehler werden angezeigt; keine simulierten Bilddaten.
- Globale Sortierung vor Pagination nach Name, Änderungsdatum oder Dateigröße, jeweils auf-/absteigend mit eindeutigem Pfad als Gleichstandsregel. Auswahl/Seite werden bei Wechsel zurückgesetzt; Sortierpräferenz lokal über Neustarts gespeichert. Die bisherigen Such-/Typ-/Tagfilter bleiben kombinierbar.
- Galerie-Tastaturaktionen: Strg+A, Esc, Links/Rechts mit optionaler Umschalt-Bereichsauswahl, F2 zum Umbenennen und Entf für den bestehenden Papierkorb-Bestätigungsdialog. Eingabefelder, Mediensteuerung und offene Dialoge werden nicht abgefangen.

Grenzen: bewusst gewählter A/B-Vergleich, keine automatische Herkunft/Versionsbeziehung, pixelgenaue Registrierung, HDR-/ICC-Referenz oder Animationssynchronisierung. AVIF-/Video-Vergleich und vollständige Versionsgruppen bleiben offen. Sortierung bleibt innerhalb der bisherigen Scan-/Seitengrenzen. Kein frei konfigurierbares Shortcut-System. Watcher, Drag-and-drop, Video-Miniaturen, Projekte und weitere M3-Funktionen bleiben im Plan.

[Portable ZIP](releases/Local-Studio-0.14.0-hub-portable.zip), [Installer](releases/Local-Studio-0.14.0-hub-setup.exe), [Bedienung](docs/GALLERY.md). **Local-Studio-Data beim Update behalten.** Verifikation separat in [TEST_MATRIX.md](TEST_MATRIX.md).

---

# Aktueller Stand 0.13.0: Größeres Galeriepaket

- Strg-Klick für einzelne Auswahl, Umschalt-Klick für Bereiche, Strg/Umschalt für additive Bereiche; normale Klicks öffnen die Vorschau. Bis zu 50 sichtbare Medien, Raster-/Listenwechsel erhält Auswahl.
- Suche nach Modell sowie positivem/negativem Prompt; Viewer mit verfügbaren Bildmaßen, Modellhash, Runtime, Datum, Auftrag und vollständigen bisherigen Bildparametern. Neue gespeicherte Bilder haben eine Identitäts-/Zeitstempelbindung; Migration alter Zuordnungen ist ausdrücklich pfadbasiert. Keine erfundenen Metadaten für Importe.
- Echtes Umbenennen, Sammelverschieben innerhalb der Galerie und Sammelverschieben in den lokalen App-Papierkorb. Windows-Dateihandles und Elternordner bleiben gepinnt, Ziele werden nicht überschrieben. Auswahl wird vorab geprüft; spätere Teilfehler werden ausdrücklich gemeldet. SQLite-Vorgangsprotokoll gleicht bereits ausgeführte Schritte nach Abbruch ab.
- Papierkorbliste mit Vorschau, Originalpfad, Löschzeit und Größe; Wiederherstellen ohne Überschreiben; separate Bestätigung für endgültiges Löschen und Leeren. Keine automatische Leerung. Galerieansichten schließen den reservierten Papierkorb aus; dessen Medienprotokoll erlaubt nur registrierte Dateien.
- Bildauftragspfade folgen app-internen Dateiaktionen. Endgültiges Löschen entfernt bekannte temporäre Bildkopien und Miniaturcache; Metadaten bleiben erhalten, Bildausgabe und erneutes Speichern werden gesperrt. Unbekannte Zusatzdateien und externe Originalquellen bleiben erhalten.
- Vier neue typisierte IPC-Befehle mit Berechtigungen ausschließlich in der lokalen Haupt-WebView. Modale Dialoge mit deckendem, an hell/dunkel/System angepasstem Hintergrund. Bestehende Datenbanken werden um Katalog-/Papierkorb-/Journal-Tabellen ergänzt.

Grenzen: keine laufwerksübergreifenden Dateiaktionen, keine reine Groß-/Kleinschreibungsumbenennung, keine seitenübergreifende normale Auswahl; keine atomare Transaktion über mehrere Dateisystemänderungen. Wiederherstellen benötigt den ursprünglichen vorhandenen Ordner. Leeren bis 20.000 Einträge. Externe Verschiebungen, Datenstammwechsel und alte pfadbasierte Metadatenzuordnungen bleiben eingeschränkt. Watcher, Drag-and-drop, Video-Miniaturen, Aufbewahrungsregeln, Versionen/Vergleich und Projekte bleiben im M3-Plan.

[Portable ZIP](releases/Local-Studio-0.13.0-hub-portable.zip), [Installer](releases/Local-Studio-0.13.0-hub-setup.exe), [Bedienung](docs/GALLERY.md). **Local-Studio-Data beim Update behalten.** Verifikation separat in [TEST_MATRIX.md](TEST_MATRIX.md).

---

# Aktueller Stand 0.12.0: Mehrfachauswahl und Sammelmarkierungen

- Auswahlkästchen für Bilder, Video und Audio in Raster und Liste. **Alle auf dieser Seite auswählen**, Auswahlzähler und **Auswahl aufheben**. Die große Medienvorschau bleibt eine getrennte Einzelauswahl. Raster-/Listenwechsel erhält die Mehrfachauswahl.
- Für 1–50 ausgewählte Medien gemeinsam Favoriten setzen/aufheben oder einen Tag hinzufügen/entfernen. Individuelle andere Tags bleiben erhalten. Tag-Normalisierung und Grenzen gelten wie bei Einzelaktionen; doppelte Tags werden nicht angelegt.
- Eine SQLite-Transaktion für die ganze Auswahl. Jede Datei wird bis zum Abschluss mit ihren Vorfahren gepinnt; Galeriepfad, Datei-ID, vollständige Dateiversion und Markierungsrevision müssen noch stimmen. Konflikt, fehlende/ersetzte Datei, Taglimit oder Schreibfehler brechen die gesamte Aktion ohne Teiländerungen ab. Neuer typisierter IPC ausschließlich für die lokale Haupt-WebView.
- Auswahl wird nach Aktion, Such-/Filter-/Ordner-/Seitenwechsel aufgehoben. Geänderte oder nicht mehr angezeigte Dateien fallen aus der Auswahl. Während noch aktuelle Listendaten fehlen oder eine Sammelaktion läuft, sind Auswahlkästchen gesperrt. Dateien ohne stabile Kennung bleiben sichtbar, aber nicht für Sammelmarkierungen auswählbar.
- Originalmedien und Generierungsmetadaten werden nicht geschrieben. Bestehende Favoriten/Tags bleiben in derselben lokalen Datenbank; keine Migration oder Löschung erforderlich.
- Erfolgsmeldungen werden bei einer neuen Auswahl ausgeblendet. Vorschaufehler sind an das jeweilige Medium und seine Dateiversion gebunden; verspätete Fehler eines vorherigen Mediums erscheinen nicht bei der neuen Vorschau.

Grenzen: Auswahl auf die aktuelle Seite mit maximal 50 Medien begrenzt; keine Auswahl über Seiten/Filter hinweg, keine Shift-Bereichsauswahl und keine Sammel-Dateiaktionen (Verschieben, Kopieren, Löschen). Mehrere Hardlinks auf dieselbe Datei nicht gemeinsam ändern; Alle auswählen berücksichtigt je Identität nur einen Eintrag. Modell-/Prompt-Suche, Video-Thumbnails, Watcher, Papierkorb, Herkunft und Projekte bleiben offen. M3 ist nicht abgeschlossen.

[Portable ZIP](releases/Local-Studio-0.12.0-hub-portable.zip), [Installer](releases/Local-Studio-0.12.0-hub-setup.exe), [Bedienung](docs/GALLERY.md). **Local-Studio-Data beim Update behalten.** Prüfnachweise separat in [TEST_MATRIX.md](TEST_MATRIX.md).

---

# Aktueller Stand 0.11.0: Rasteransicht und lokale Bildminiaturen

- Galerie zwischen **Rasteransicht** und **Listenansicht** umschalten; Auswahl bleibt erhalten, Ansichtspräferenz lokal über Neustarts gespeichert. Rasterkarten zeigen Dateiname, Pfad, Typ/Größe und vorhandene Favoriten/Tags. Die große Vorschau lädt nur das ausgewählte Medium.
- PNG, JPEG, WebP, GIF und BMP werden tatsächlich auf höchstens 320 × 320 Pixel verkleinert, mit Seitenverhältnis, Alpha und EXIF-Ausrichtung. Kleine Bilder werden nicht vergrößert. GIF-/WebP-Miniaturen sind statisch; AVIF, Video und Audio haben vorerst Symbolkarten. Diese können weiterhin im bestehenden Viewer geöffnet werden.
- Sichtbare Karten fordern Miniaturen über IntersectionObserver an. Frontend und Backend begrenzen die parallele Arbeit auf zwei; ausgehängte Karten verwerfen wartende Anfragen und späte Antworten. 50 Medien pro Seite statt Laden sämtlicher Originale. Dekodierung läuft außerhalb des UI-Threads. Grenzen: 64 MiB Quelldatei, 16.384 Pixel je Achse, 32 Megapixel und 128 MiB dekodierte Bilddaten; zusätzliche Codec-Allokationsgrenze ist best effort.
- Neu aufbaubarer SQLite-Cache unter cache/gallery-thumbnails-v1.sqlite3. Schlüssel enthält Galeriewurzel, Pfad, Windows-Dateiidentität, Dateigröße und vollständigen Änderungs-/Erstellungszeitstempel. Jede Anfrage prüft zuerst die bestehende Quelldatei und Version. Beschädigte Cacheeinträge werden neu erzeugt; ein nicht beschreibbarer Cache verhindert die Miniatur nicht. Höchstens 2.000 Einträge und 64 MiB PNG-Nutzdaten, ältere zuletzt benutzte Einträge werden verworfen. SQLite-Verwaltungs-/Freiseiten können zusätzlichen Platz belegen.
- **Vorschaubilder neu aufbauen** leert nur den Miniaturcache und erzeugt sichtbare Miniaturen neu. Originale und Favoriten/Tags bleiben unberührt. Fehlerhafte oder zu große Bilder bleiben als anklickbare Karten mit Hinweis sichtbar. Thumbnail-IPC nur für die lokale Haupt-WebView.

Grenzen: keine Video-Frames/Audio-Wellenformen, AVIF-Miniaturen, vollständige Listenauflösung/Modellspalten, ICC-/HDR-Farbabnahme, Dateisystem-Watcher oder unbegrenzte Galerievirtualisierung. Pagination und sichtbarkeitsabhängiges Laden sind die derzeitige Begrenzung. Gezielte Inhaltsänderungen unter Beibehaltung aller Dateikennungen/Zeitstempel brauchen manuellen Cache-Neuaufbau; der Cache ist kein Integritätsnachweis. Modell-/Prompt-Suche, Sammelaktionen, Papierkorb, Drag-and-drop, Herkunft und Projekte bleiben im Plan. M3 ist nicht abgeschlossen.

[Portable ZIP](releases/Local-Studio-0.11.0-hub-portable.zip), [Installer](releases/Local-Studio-0.11.0-hub-setup.exe), [Bedienung](docs/GALLERY.md). **Local-Studio-Data beim Update behalten.** Verifikation separat in [TEST_MATRIX.md](TEST_MATRIX.md).

---

# Aktueller Stand 0.10.0: Favoriten und Tags

- Alle erkannten Galerie-Medien haben **Zu Favoriten hinzufügen/entfernen** und frei vergebene Tags. Hinzufügen per Enter/Button und Entfernen am Tag werden sofort lokal gespeichert. Eine noch nicht hinzugefügte Eingabe ist kein gespeicherter Tag.
- **Nur Favoriten**, ein genauer Tagfilter und der Medientyp lassen sich kombinieren. Die Textsuche findet zusätzlich zu Dateiname/Pfad auch Tags, ohne Beachtung der Groß-/Kleinschreibung. Filter greifen vor der Pagination; die Tagauswahl enthält nur Tags tatsächlich gefundener Dateien im aktuellen Ordnerumfang.
- Markierungen liegen in config/gallery.sqlite3, getrennt von Originalmedien und Generierungsmetadaten. Bis zu 32 Tags je Datei, jeweils 64 Zeichen. Äußere/mehrfache Leerzeichen und doppelte Tags werden normalisiert; Steuerzeichen und Kommas sind ausgeschlossen.
- Zuordnung nutzt den vollständigen 128-Bit-Windows-Dateibezeichner mit Volume- und Erstellungskennung, getrennt nach Galeriepfad. Umbenennen/Verschieben innerhalb derselben Galerie auf NTFS erhält die Markierungen. Ersetzen des alten Pfads durch eine neue Datei übernimmt sie nicht. Datei-ID und Revision werden beim Schreiben erneut geprüft; Schreibfehler oder veraltete Änderungen überschreiben den gespeicherten Stand nicht.
- Der neue Schreibbefehl ist ausschließlich für die lokale Haupt-WebView erlaubt. Dateien ohne verwendbare Identität bleiben sichtbar/abspielbar, ihre Markierungen sind mit einem Hinweis deaktiviert.

Grenzen: keine automatische Übertragung der Tags auf Kopien, andere Laufwerke, andere PCs oder geänderte Galerie-Wurzelpfade. Änderungen am Inhalt derselben Datei behalten ihre Dateimarkierungen; diese sind kein Inhalts-/Hashnachweis. Hardlinks derselben Datei im gleichen Galerieordner teilen die Markierungen. Suche nach Modell/Prompt, Thumbnail-Raster, Sammelaktionen, Papierkorb, Drag-and-drop und Projekte bleiben separat offen. M3 ist nicht abgeschlossen.

[Portable ZIP](releases/Local-Studio-0.10.0-hub-portable.zip), [Installer](releases/Local-Studio-0.10.0-hub-setup.exe), [Bedienung](docs/GALLERY.md). **Local-Studio-Data beim Update behalten.** Prüfnachweise separat in [TEST_MATRIX.md](TEST_MATRIX.md).

---

# Aktueller Stand 0.9.0: gemeinsame Galerie auf echten Dateien

- Die Galerie hat eine eigene Medienansicht für Bilder, Video sowie Audio/Musik. Sie liest den tatsächlichen Galerieordner einschließlich optionaler Unterordner; gespeicherte Bildgenerierungen und extern abgelegte Dateien erscheinen gemeinsam. Audio und Musik bleiben eine gemeinsame Kategorie, ohne erfundene Inhaltsklassifikation.
- Ordner öffnen/anlegen, zum übergeordneten Ordner wechseln, Dateiname/Pfad suchen, nach Medientyp filtern und Seiten mit jeweils 50 Dateien sind vorhanden. Während die Galerie sichtbar ist, wird alle 3 Sekunden erneut gelesen. Das ist begrenztes Polling, noch kein Dateisystem-Watcher oder dauerhafter Inhaltsindex.
- **Dateien kopieren …** importiert 1–100 bewusst ausgewählte Medien. Originale bleiben erhalten. Temporär kopieren, SHA-256 gegen Quelle prüfen, dann atomar ohne Überschreiben veröffentlichen; Namenskollisionen erhalten einen eindeutigen neuen Namen. Teilfehler werden mit Dateinamen angezeigt. Ordner und enthaltene Dateien lassen sich im Explorer öffnen.
- Ein Medienviewer lädt nur die ausgewählte Datei. Bilder: Zoom, Einpassen, Scrollen und Vollbild. Video/Audio: native WebView-Wiedergabesteuerung, kein Autoplay; unterstützte Codecs hängen von Windows/WebView ab. Beschädigte oder nicht abspielbare Dateien bleiben in der Liste mit verständlicher Vorschaufehlermeldung.
- Lokal erzeugte Bilder behalten **Einstellungen wiederherstellen**, solange der Galeriepfad der gespeicherten Datei dem Auftrag zugeordnet bleibt. Der neue Dateibrowser hängt für normale Medien nicht von der Bildauftragshistorie ab.
- Medienzugriff erfolgt über ein eigenes lokales Protokoll, auf die Haupt-WebView und den aktuellen Galerieordner begrenzt. Relative Pfade werden geprüft; UNC, Pfadtraversal und Reparse-Points sind ausgeschlossen. Nur erlaubte passive Medienformate werden ausgeliefert, keine HTML-/SVG-Dateien. Audio/Video lesen begrenzte Bytebereiche statt ganze Filme in IPC-Nachrichten.

Grenzen dieses ersten Galeriepakets: maximal 20.000 besuchte Einträge pro Suche, 32 Unterordnerebenen, 200 Ordnerbuttons; Begrenzungen werden angezeigt. Bildvorschau bis 32 MiB Datei, Kopierimport bis 8 GiB pro Datei. Keine Raster-Thumbnails, Tags/Favoriten, Mehrfachaktionen, Umbenennen/Verschieben, Drag-and-drop, Papierkorb, Bildvergleich, Projekte oder dauerhafter Metadatenindex. Manuell verschobene/umbenannte Generierungen benötigen später eine stabile Metadatenzuordnung. Ein abgebrochener Import kann eine nicht veröffentlichte .tmp-Datei hinterlassen; Quellen bleiben erhalten. Der volle M3-Umfang bleibt im Plan erhalten.

[Bedienung](docs/GALLERY.md), [Portable ZIP](releases/Local-Studio-0.9.0-hub-portable.zip), [Installer](releases/Local-Studio-0.9.0-hub-setup.exe). Beim Update **Local-Studio-Data behalten**. Konkrete Verifikation separat in [TEST_MATRIX.md](TEST_MATRIX.md).

---

# Aktueller Stand 0.8.0: Bildwarteschlange und Einstellungen wiederherstellen

- Während einer Generierung lassen sich weitere Bilder vorbereiten und einreihen. Ein gemeinsamer FIFO-Scheduler führt genau einen Bildauftrag gleichzeitig aus; bis zu 20 weitere dürfen warten. Jeder Auftrag hält eigene Modellwahl, Prompts, Parameter, Modellhash und Runtime-/GPU-Angaben. Einreihen prüft die Modellbytes; die Datei bleibt bis zum Auftragsende gegen Schreiben und Löschen geöffnet. Kein automatischer Modelldownload.
- Studio und Aufträge zeigen wartende Positionen, laufenden Fortschritt und Abbruch. Abbruch eines wartenden Auftrags entfernt nur diesen. Änderungen im Formular verändern vorhandene Aufträge nicht. Fehler einer Generierung beenden diesen Auftrag und lassen den nächsten beginnen; ein Datenbankfehler hält die Warteschlange an.
- Beim regulären Beenden werden wartende Bilder pausiert; nach unerwartetem Ende geschieht das beim Start. Keine automatische Wiederaufnahme. **Erneut einreihen** prüft Modellhash und Runtime/GPU gegen den ursprünglichen Auftrag, bevor dieser wieder warten darf. Unterbrochene Inferenz wird nicht fortgesetzt. Ein durch Speicherfehler angehaltener Scheduler erfordert einen App-Neustart nach Beheben des Speicherproblems.
- **Einstellungen wiederherstellen** übernimmt aus einem Bildauftrag oder gespeicherten Galeriebild Modellpfad, beide Prompts und sämtliche Parameter ins Studio. Es entsteht kein Auftrag. Vor der neuen Generierung muss die aktuelle Modelldatei erneut geprüft werden.

Die vom Nutzer in der Entwicklungsphase erzeugten Bilder gelten als entbehrliche Testdaten. Das ändert weder den Schutz von Modellen/Einstellungen noch die regulären Speicher-/Verwerfen-Optionen des Programms. Keine pauschale Löschung bestehender Medien.

[Portable ZIP](releases/Local-Studio-0.8.0-hub-portable.zip), [Installer](releases/Local-Studio-0.8.0-hub-setup.exe). App schließen, Programmdateien samt `image-runtime` ersetzen und **Local-Studio-Data behalten**. Verifikation separat in [TEST_MATRIX.md](TEST_MATRIX.md). Vollständige M2-/M3-Abnahme, weitere Adapter, Galerie-Dateiverwaltung, automatische Bereinigung, Benchmarks, 18+-Sperre und alle späteren Meilensteine bleiben offen.

---

# Aktueller Stand 0.7.1: deckender Beenden-Dialog

Der Bild-Beenden-Dialog verwendete die nicht definierte CSS-Variable `--surface`, wodurch der Hintergrund transparent blieb. Er verwendet jetzt die vorhandene deckende Theme-Farbe `--panel` und den gemeinsamen Dialogschatten. Gleichartige Verweise in Promptfeldern, Loganzeige und Bildverlauf verwenden nun die passenden vorhandenen Feld-/Panel-Farben. Hell, Dunkel und Windows-Systemmodus bleiben verfügbar; Beenden-/Speicherlogik unverändert.

[Portable ZIP](releases/Local-Studio-0.7.1-hub-portable.zip), [Installer](releases/Local-Studio-0.7.1-hub-setup.exe). Vor dem Update eigene Bilder speichern und die App regulär schließen. Programmdateien einschließlich `image-runtime` ersetzen; **Local-Studio-Data behalten**. Prüfnachweise separat in [TEST_MATRIX.md](TEST_MATRIX.md).

---

# Aktueller Stand 0.7.0: Studio-Auswahl, Bildaufträge und Ergebnis-Lebenszyklus

- Das Studio bietet erkannte lokale Safetensors mit Größe in GB zur Auswahl. Fehlende/defekte Dateien sind gesperrt; Ausführbarkeit wird am ausgewählten Modell geprüft. Auswahl allein lädt keine Gewichte und bestätigt keine unterstützte Architektur.
- Prompt und Formular bleiben beim Ansichtswechsel erhalten. Modellwechsel über Auswahl oder Dateiauswahl merkt sich die jeweiligen Parameter und behält den Prompt. Arbeitsstand und bis zu 256 Modellpräferenzen liegen lokal in SQLite.
- **Aufträge** führt Bildgenerierungen und SHA-256-Prüfungen gemeinsam nach Erstellungszeit auf: tatsächlicher Phasenfortschritt, Fehler, Abbruch, unveränderliche Parameter und Öffnen des konkreten Ergebnisses im Studio. Navigation und globale Anzeige berücksichtigen laufende Bilder. Die Historie lässt sich schrittweise erweitern; ältere ungespeicherte Bilder fallen nicht mehr hinter einer 50-Aufträge-Grenze weg.
- Ein gemeinsamer Beenden-Dialog bietet Speichern aller fertigen Bilder, Verwerfen oder Abbrechen und berücksichtigt laufende Bilder, Downloads, Dateiprüfungen sowie ungespeicherte Einstellungen. Abbrechen verändert keine laufende Generierung. Nach Bestätigung wird der Worker zuerst beendet/abgewartet; inzwischen fertig gewordene Bilder werden ebenfalls behandelt. Speicherfehler verhindern den Abschluss.
- **Einstellungen → Studio beim Start**: leere Sitzung ist Standard. Bei Wiederherstellung kann der Bild-Arbeitsstand samt ungespeicherten Ergebnissen ausdrücklich behalten werden. Nach unerwartetem Beenden wird unabhängig davon Recovery angeboten. Keine Fortsetzung unterbrochener Inferenz. Ein vollständig geschriebenes, validiertes PNG wird auch erkannt, wenn der Prozess vor dem finalen Auftragseintrag abstürzte.
- Verwerfen betrifft nur bekannte temporäre Dateien im eigenen UUID-Auftragsordner. Löschabsicht wird vorher persistiert und kann nach einem Fehler wiederholt werden. Keine rekursive Löschung; Galerie, Modelle und unbekannte Benutzerdateien bleiben erhalten. Auftragsmetadaten bleiben im Verlauf. Der eingebettete Website-WebView wird während des Beenden-Dialogs verborgen und bei Abbrechen wieder eingeblendet.

Weiter offen: automatische 7-Tage-Bereinigung und drei geschützte Recovery-Punkte, vollständige Projekt-/Medien-Recovery, Galerie-Dateiverwaltung, Warteschlange für mehrere Bildgenerierungen, Erweiterungen, Benchmarks, 18+-Sperre und weitere Adapter. Bildhistorie wird noch vollständig aus SQLite gelesen; große Bestände benötigen später Datenbank-Pagination/Thumbnails. M2/M3 sind insgesamt weiterhin offen. Konkrete Prüfnachweise stehen separat in [TEST_MATRIX.md](TEST_MATRIX.md).

[Portable ZIP](releases/Local-Studio-0.7.0-hub-portable.zip), [Installer](releases/Local-Studio-0.7.0-hub-setup.exe). Alte App schließen, Programmdateien einschließlich `image-runtime` ersetzen und **Local-Studio-Data behalten**.

---

# Aktueller Stand 0.6.1: weniger falsche Treffer bei der Modellsuche

Die bisherige Vollsuche übernahm Dateiendungen zu großzügig. Dadurch erschienen unter anderem Python-Pfaddateien, übersetzte Vim-Texte, ONNX-Dateien installierter Programme, Bibliothekstests und lokale Testartefakte gemeinsam mit eigenen Modellgewichten.

- Schnell- und Vollsuche überspringen Papierkorb/System Volume Information, Windows-/Program-Files-Verzeichnisse an der Laufwerkswurzel sowie `site-packages`, `dist-packages`, `node_modules`, `.git`, `.svn`, `__pycache__` und `.artifacts`. Übersprungene Bereiche werden mit Grund gezählt/protokolliert. Eine gezielte Ordnersuche bleibt für bewusst ausgewählte Spezialablagen möglich.
- Als PyTorch-Kandidaten gefundene reine Textdateien werden anhand eines begrenzten Dateianfangs ausgeschlossen. Keine Deserialisierung oder Ausführung von Python/Pickle. Es gibt keinen pauschalen Mindestgrößenfilter, der kleine LoRAs oder echte kleine Modelle aussortiert.
- Safetensors erhalten zusätzlich eine konservative Erkennung üblicher Gewichts-/Bias-/LoRA-Schlüssel. Strukturell gültige Tensorcontainer ohne solche Schlüssel bleiben unbestätigte Kandidaten, etwa Trainings-Zwischendaten. Andere Namensschemata bleiben zur Prüfung erreichbar; dies ist keine universelle Architekturvalidierung.
- Die lokale Liste zeigt standardmäßig erkannte Modellformate. Unbestätigte Kandidaten und ausgeblendete alte Treffer sind getrennt auswählbar, mit maximal 50 Karten pro Seite. Bekannte Modelldateien bleiben auch bei späterer Beschädigung/Abwesenheit sichtbar.
- Beim ersten Start wird die bestehende Modellbibliothek anhand der neuen Regeln eingeordnet. Alle vorhandenen Referenzen bleiben gespeichert; kein Modell und keine andere Quelldatei wird gelöscht oder verschoben. Alte Vollsuchen werden am gespeicherten Laufwerkswurzelpfad erkannt. Bewusst gewählte Modellordner bleiben berücksichtigt.

[Portable ZIP](releases/Local-Studio-0.6.1-hub-portable.zip), [Installer](releases/Local-Studio-0.6.1-hub-setup.exe). Portable App schließen, Programmdateien samt `image-runtime` ersetzen und **Local-Studio-Data behalten**. Die Einordnung der alten Liste erfolgt beim nächsten App-Start. [Prüfnachweise](TEST_MATRIX.md). M1/M2 und alle späteren Anforderungen bleiben im dokumentierten Umfang offen.

---

# Aktueller Stand 0.6.0: Schnellsuche und echte lokale SDXL-Bilder

- **Modelle → Lokal gespeichert → Schnellsuche** durchsucht bekannte Modellablagen und HF-Caches, entfernt überlappende Wurzeln und erhält die vorhandene Bibliothek bei Abbruch. Eine eng begrenzte HF-Snapshot-zu-Blob-Dateiverknüpfung wird unterstützt; allgemeine Verzeichnislinks bleiben ausgeschlossen.
- **Im Studio prüfen → Ausführbarkeit prüfen** prüft ein einzelnes SDXL-Safetensors-Modell auf Struktur, UNet/CLIP-L/CLIP-G/VAE, eine unveränderte Bildruntime und NVIDIA/Vulkan. Fehlende Bestandteile werden konkret benannt. Die Vorprüfung verspricht keinen erfolgreichen Modelllauf und keine geklärte Modelllizenz.
- **Studio** führt reale Text-to-Image-Aufträge im separaten, an den App-Prozess gebundenen `sd-cli.exe` aus. Modellbytes werden gehasht und während des Auftrags schreibgeschützt geöffnet. Prompts/Parameter werden als unveränderlicher Auftrag gespeichert; Fortschritt stammt aus Hashbytes und Denoising-Schritten. Abbruch, begrenzte Logs, validierte PNG-Vorschau und unterbrochene Aufträge nach Neustart sind vorhanden.
- **In Galerie speichern** legt eine separate PNG-Datei plus vollständige Auftragsmetadaten im Galeriepfad an. Die erste Galerieansicht zeigt gespeicherte Bilder der letzten 50 Aufträge und liest gespeicherte Kopien unabhängig vom temporären Original. Jede Generierung startet/lädt einen eigenen Worker und beendet ihn anschließend.
- Installer und Portable enthalten die festgelegte, hashgeprüfte Vulkan-Runtime einschließlich Lizenzhinweisen. Kein Modell wird mitgeliefert. Keine Python- oder modellseitige Abhängigkeitsinstallation. Lokale Main-Window-IPC für alle neuen Aktionen; der HF-Website-WebView erhält keine Bild- oder Scanberechtigung.
- Auftragsreihenfolge richtet sich auch nach Neustart nach der Erstellungszeit. Der lokale OAuth-Callback setzt akzeptierte Windows-Verbindungen explizit auf blockierendes Lesen mit Zeitlimit, damit kurz verzögerte Requestbytes nicht vorzeitig als ungültiger Callback behandelt werden.

M1/M2/M3 bleiben teilweise offen: weitere Architekturen/Komponenten, allgemeine Runtime-Verwaltung, unabhängige vollständige Galerie, automatische Entwurfsbereinigung, individuelle Medien-Recovery, 18+-Sperre, lokale Benchmarkverwaltung und breitere Hardware-/Fehlerabnahme. Aktuelle Entwürfe bleiben erhalten; dies wird im Studio ausdrücklich angezeigt. Nur der hier beschriebene SDXL-Einzeldatei-Pfad ist implementiert. [Bedienung und Grenzen](docs/IMAGE_GENERATION.md).

[Installer](releases/Local-Studio-0.6.0-hub-setup.exe), [Portable ZIP](releases/Local-Studio-0.6.0-hub-portable.zip). Beim portablen Update `image-runtime` mit übernehmen und **Local-Studio-Data behalten**. Implementierung und konkrete Verifikation sind getrennt: [TEST_MATRIX.md](TEST_MATRIX.md), [Modellnachweise](MODEL_COMPATIBILITY.md). SPEC und Originalprompt bleiben unverändert.

---

## Historische Implementierungsstände

# Aktueller Stand 0.5.1: Button für vollständige PC-Suche

**Modelle → Lokal gespeichert → Vollständige Suche** startet bewusst die Suche über alle von Windows gemeldeten lokalen Laufwerksbuchstaben. Interne, Wechsel-, optische und RAM-Laufwerke werden berücksichtigt; gemappte Netzlaufwerke ausgeschlossen. Die bisherige gezielte Ordnersuche bleibt verfügbar.

Die Oberfläche zeigt Modus, Laufwerke, bearbeitete Laufwerke, aktuellen Pfad, besuchte Pfade, Funde und übersprungene Pfade. Abbruch verwirft den laufenden Suchlauf und erhält die bisherige Bibliothek. Unzugängliche Orte und Reparse-Points werden übersprungen; bis zu 50 konkrete Hinweise werden aufbewahrt. Originaldateien bleiben am Speicherort. Der neue Command ist ausschließlich für die lokale Hauptansicht erlaubt.

Die Traversierung hält Ordneriteratoren statt ganzer Verzeichnisbäume im Speicher. Die Vollsuche hat keine 100.000-Pfade-Grenze der Ordnersuche; Ressourcenlimits sind 10.000 Funde, 20.000 Dateireferenzen und 128 Unterordnerebenen. Erreichte Grenzen werden ausdrücklich als **Suche unvollständig** dargestellt. Die bestehende begrenzte Format-/Strukturprüfung gilt unverändert; vollständige Suche bezeichnet den Suchbereich, keine vollständige Modellvalidierung.

Schnellsuche, Ersteinrichtungsfrage, Volumes ohne Laufwerksbuchstaben, HF-Cache-Symlinks, Verschieben und Runtime-/Modellvalidierung bleiben offen. [Details](docs/MODEL_IMPORT.md), Verifikation getrennt in [TEST_MATRIX.md](TEST_MATRIX.md).

[Installer](releases/Local-Studio-0.5.1-hub-setup.exe), [Portable ZIP](releases/Local-Studio-0.5.1-hub-portable.zip). SPEC und Originalprompt unverändert.

---

# Aktueller Stand 0.5.0: lokale Modellordner einbinden

- **Modelle → Lokal gespeichert → Modelle vom PC einbinden** durchsucht einen bewusst gewählten lokalen Ordner samt Unterordnern. Originaldateien bleiben unverändert; nur Referenzen und Prüfergebnisse werden lokal in SQLite gespeichert.
- Einträge zeigen vorhandene Dateigröße in GB, Format, Pfad, gegebenenfalls Modellfamilie und erkannte Zusatzkomponenten. Safetensors erhalten eine begrenzte Strukturprüfung. Gewichtsindizes bündeln Teil-Dateien und melden fehlende Teile. GGUF wird am Kopf erkannt; andere unterstützte Dateiendungen bleiben ungeprüfte Kandidaten.
- Wiederholte Suche aktualisiert dieselben Referenzen. Solange die Ansicht offen ist, werden bekannte Pfade regelmäßig auf fehlende oder veränderte Dateien geprüft. **Erneut prüfen** aktualisiert die Strukturprüfung; **Aus Liste entfernen** lässt Originaldateien erhalten.
- Suche im Hintergrund mit Abbruch, sichtbaren Grenzen und übersprungenen Pfaden. Abgebrochene Suchen übernehmen keine neuen Ergebnisse. Keine Modellcode-Ausführung, keine Netzwerkanfrage, keine automatische Abhängigkeitsinstallation. Neue IPC nur für die lokale Hauptansicht.
- Buildskript ergänzt den installierten Windows-SDK-Pfad für den Ressourcencompiler, wenn `rc.exe` im gestarteten Prozess fehlt.

[Prüfumfang und Grenzen](docs/MODEL_IMPORT.md). **Dateistruktur geprüft** bestätigt keine vollständige Runtime, Lizenz oder Ausführbarkeit. M1 bleibt teilweise offen: PC-Schnellsuche/alle Laufwerke, weitere Quellen, sichere Modell-/Runtime-Installation, Verschieben/Reparatur/Updates und private/gated Fälle. M2-Inferenz ist noch nicht implementiert. Der vollständige Umfang von M0–M12 bleibt erhalten.

[Installer](releases/Local-Studio-0.5.0-hub-setup.exe), [Portable ZIP](releases/Local-Studio-0.5.0-hub-portable.zip). Für ein portables Update die App schließen, Programmdateien ersetzen und **Local-Studio-Data behalten**. Installer übernimmt den Windows-App-Modus beim Start wie in 0.4.2.

Verifikation separat in [TEST_MATRIX.md](TEST_MATRIX.md). SPEC.md und Originalprompt bleiben unverändert.

---

# Aktueller Stand 0.4.2: Installer folgt Windows

Inno Setup ersetzt NSIS im regulären Desktop-Build. Setup und Deinstallation wählen beim Start den Windows-App-Modus. Der Build prüft den offiziellen Compiler und den Microsoft-WebView2-Bootstrapper. Portable Ordner und alte NSIS-Installationen werden vor Änderungen erkannt; eine alte NSIS-Installation benötigt den dokumentierten manuellen Wechsel. Keine automatische Datenmigration oder rekursive Datenlöschung.

[Installer-Verhalten, Build und Grenzen](docs/INSTALLER.md). Testnachweise separat in TEST_MATRIX.md. App-Inferenz und signierte automatische Updates bleiben offen.

---

# Aktueller Stand 0.4.1: Modellgrößen in GB

- Suchkarten laden die Gesamtgröße der Dateien der aufgelösten Revision im Hintergrund nach (drei Abrufe pro aktiver Warteschlange). Beschriftung **Repository gesamt**; umfasst auch alternative Varianten. Keine Schätzung aus Parameterzahlen oder HF-Speicherverbrauch einschließlich Historie.
- Details zeigen dieselbe Gesamtgröße. Dateiauswahl, Downloadvorschau und lokale Dateiliste zeigen die Größe ihrer konkreten Dateien in dezimalen GB (1 GB = 1.000.000.000 Bytes); Dateizeilen behalten zusätzlich die feinere bisherige Einheit.
- Fehlende/unvollständige Größen oder fehlgeschlagene Abrufe erscheinen als unbekannt. Kleine positive Dateien erscheinen als **< 0,01 GB**, niemals irreführend als null. Leere Auswahl bleibt null. Lokale Größe bezeichnet die gespeicherte Dateiauswahl laut Download-Metadaten, keine aktuelle Dateisystem-Belegungsmessung.
- Größenabruf verwendet vorhandene HF-Host-/Token-/Antwortgrößengrenzen. Keine Model Card und keine Gewichtsdatei wird dafür geladen. Queued Requests werden bei Ansicht-/Suchwechsel verworfen; bereits laufende Antworten ändern die neue Ansicht nicht. Neuer Command nur für lokale Hauptansicht freigegeben.

[Portable Update](releases/Local-Studio-0.4.1-hub-portable.zip), [Installer](releases/Local-Studio-0.4.1-hub-setup.exe). Programm schließen, drei Programmdateien ersetzen, **Local-Studio-Data behalten**. M1 und Inferenz bleiben im unten dokumentierten Umfang offen.

Prüfergebnisse separat in TEST_MATRIX.md. Originalprompt/SPEC unverändert.

---

# Aktueller Stand 0.4.0: Downloads und lokale Dateiliste

Stand: 17. September 2026. Der Nutzer bestätigt den integrierten App-Login aus 0.3.1 ausdrücklich mit „okay login klappt“. Diese Bestätigung gilt dem echten App-Login; Refresh, Kontowechsel und gesonderte Website-Sitzungspersistenz bleiben eigene Prüffelder.

## Implementiertes Arbeitspaket

- **Modelle → Details**: ausdrückliche Dateiauswahl ohne Vorauswahl, native Downloadvorschau mit exaktem Commit, Größe, konservativem zusätzlichem Speicherbedarf, Lizenz/Zugang und Ziel. Erst **Auswahl herunterladen** startet die Übertragung.
- **Downloads**: echte Bytes/Fortschritt/Geschwindigkeit/Restmenge/abschätzbare Restzeit, Queue, Pause, unterstützte Range-Fortsetzung, Abbruch mit behaltenen Teil-Dateien, Retry, bestätigter Neustart ab Byte 0, Priorität und pausierte Wiederherstellung nach App-Neustart. Beim App-Schließen wird auf laufende Downloads hingewiesen und sauber pausiert.
- **Modelle → Lokal gespeichert**: heruntergeladene Dateiauswahlen mit Quelle, Lizenz/Aufgabe, Commit, Dateien, Ziel und Prüfsummen. Erneute lokale Prüfung erkennt Korruption oder fehlende Dateien. Die Liste behauptet keine Runtime-Vollständigkeit oder Ausführbarkeit.
- Hashprüfung gegen HF-Quellhashes, separate geprüfte Zielkopie und abschließender Rename. Keine fremden Modellskripte/Abhängigkeiten ausgeführt. Tokens nur für den exakten HF-Host, keine Weitergabe an CDNs; keine Tokens/signierten URLs in SQLite. Teil-Dateien bleiben bei Fehler/Abbruch erhalten.

## Verifikation

- Frontend-/Tauri-/NSIS-Build, **34 Rusttests**, **8 Frontendtests** bestanden.
- [Native Prüfung der paketierten EXE](.artifacts/native-1789610509905/report.json): **34/34 bestanden**. Alle bisherigen Core-/HF-/Browser-/OAuth-Prüfungen plus fünf Downloadprüfungen.
- Echte HF-Dateien aus `hf-internal-testing/tiny-random-gpt2`, Commit `71034c5d8bde858ff824298bdedc65515b97d2b9`: `config.json` (807 Bytes, Git-Blob-Prüfung) und `model.safetensors` (453.864 Bytes, SHA-256 `8111d5afb0715dbf5a31396d31432cb56370ba23f6650a035ea0fc8a20b4e500`). Nur Byte-/Downloadtest; kein Inferenzmodell integriert oder ausgeführt.
- Native UI-Vorschau ohne automatische Auswahl, geprüfte Übernahme, lokale Dateiliste, Erkennung einer gleich großen beschädigten Testkopie, Wiederherstellung/erneute Prüfung, ungültige Pfade/verbrauchte Plan-IDs/Duplikate und Persistenz nach Neustart bestanden. Der Website-WebView kann auch `download_list` nicht aufrufen; ACL-Ablehnung nativ bestätigt.
- Kontrollierte HTTP-Tests: Pause mit stabiler Teil-Datei, echte Range-Fortsetzung nach Wiederöffnung, nicht unterstützte Fortsetzung ohne Teilverlust, bestätigter Neustart, Cancel, Hashfehler ohne Veröffentlichung, Recovery und Queue-Priorität. Diese Tests sind vom echten öffentlichen HF-Download klar getrennt.
- Download-Vorschau und lokale Dateiliste anhand nativer Screenshots visuell geprüft; keine Überlappung in der geprüften Größe. Keine vollständige Accessibility-/Fenstergrößenabnahme.
- Ein früher nativer Lauf fand das öffentliche interne HF-Testrepo nicht über den Suchindex; der Test verwendet jetzt die vorhandene Website-zu-Modell-Übergabe. Der fehlgeschlagene Suchlauf wird nicht als bestandener Downloadtest gewertet.
- ZIP-CRC, genau drei Programmdateien, UTF-8-Anleitung, Identität mit der getesteten EXE und Paketprüfsummen bestätigt. Keine Modelle/Secrets/Testsitzungen mitgeliefert. Installer gebaut; Installation/Deinstallation nicht geprüft. Kein vollständiger Rust-Abhängigkeitsaudit.

## Pakete und nächste Grenze

[Portable ZIP](releases/Local-Studio-0.4.0-hub-portable.zip), [EXE](<releases/Local Studio Hub 0.4.0/Local Studio.exe>), [Installer](releases/Local-Studio-0.4.0-hub-setup.exe), [SHA-256](releases/SHA256SUMS-0.4.0.txt). App zum Update schließen, Programmdateien im bisherigen Ordner ersetzen und **Local-Studio-Data behalten**. Bestehende Nutzermodelle und Nutzeranmeldung wurden nicht verändert.

M1 ist weiter offen: PC-Suche/Import, vollständige Runtime-/Komponenten-Erkennung, Modellreparatur, Entfernen/Verschieben/Updates, weitere Quellen und private/gated Downloadfälle benötigen eigene Umsetzung/Tests. Downloads laufen aktuell einzeln; die Dateiliste ist keine vollständige Modellverwaltung. Inferenz und alle späteren M0–M12-Anforderungen bleiben erhalten. Details: [DOWNLOADS.md](docs/DOWNLOADS.md). SPEC und Originalprompt sind unverändert und bytegleich.

---

## Historischer Stand bis 0.3.1

# Aktueller Stand 0.3.1: App-Login im integrierten Browser

Auf ausdrücklichen Nutzerwunsch öffnet **Mit Hugging Face anmelden** jetzt die echte OAuth-Anmeldeseite im integrierten Browser von Local Studio. Ein noch nicht vorhandener WebView startet direkt mit dem Loginziel; ein vorhandener WebView wird dafür wiederverwendet. Abschluss und Abbruch führen zur Kontoansicht zurück. Während der Anmeldung nennt die Oberfläche ausdrücklich die Verbindung des App-Kontos.

PKCE, zufälliger State, temporärer Loopback-Listener, Windows Credential Manager und die strikte lokale WebView-Capability bleiben erhalten. Nur der exakte aktive Callback-Endpunkt darf während der Anmeldung zusätzlich navigiert werden. Abschluss/Abbruch entfernt diese Ausnahme; Website-Cookies und App-Tokens bleiben technisch getrennt. Keine Anmeldedaten werden aus der Website gelesen.

**Nachweise:** Frontend-/Tauri-/NSIS-Build, 8 Frontendtests, 26 Rusttests und [29 native Prüfungen](.artifacts/native-1789609392616/report.json) bestanden. Drei neue native Prüfungen belegen den Button-Wechsel zur echten HF-Passwortseite mit PKCE und verweigerter privilegierter IPC, die Ablehnung eines gefälschten States sowie eine gültige Ablehnungsantwort über den echten WebView-Callback, erneuten Versuch mit frischem State und Abbruch mit geschlossenem Port. Der native Screenshot der HF-Anmeldeseite wurde visuell geprüft. Keine erfolgreiche Kontoanmeldung simuliert.

Die paketierte EXE ist bytegleich mit der geprüften Kopie; ZIP-CRC/Inhalt, Anleitung und Prüfsummen stimmen. [Portable ZIP](releases/Local-Studio-0.3.1-hub-portable.zip), [EXE](<releases/Local Studio Hub 0.3.1/Local Studio.exe>), [Installer](releases/Local-Studio-0.3.1-hub-setup.exe), [SHA-256](releases/SHA256SUMS-0.3.1.txt). Für das Update App schließen, Programmdateien im bisherigen Ordner ersetzen und **Local-Studio-Data behalten**.

**Grenzen:** Den vollständigen internen Login samt echter Zustimmung/Tokenaustausch muss der Nutzer noch abschließen; seine vorherige Bestätigung galt dem externen Browserpfad. Refresh, Drittanbieter-SSO, echte Kontowechsel und Installer-Lebenszyklus sind weiterhin ungeprüft. Der gewünschte eingebettete User-Agent weicht von RFC 8252 §8.12 ab; Sicherheitsabwägung siehe [HUGGING_FACE.md](docs/HUGGING_FACE.md). Die Original-SPEC bleibt unverändert; die neue Nutzeranweisung ist hier als Verhaltensänderung festgehalten. Downloadmanager, lokale Modellverwaltung und Inferenz bleiben offen.

---

## Historischer Stand bis 0.3.0

# Projektstatus

Stand: 17. September 2026, **0.3.0 Core + Hugging Face mit integriertem Website-Browser**. M0 und M1 bleiben teilweise offen; keine KI-Inferenz und keine Modelldownloads. SPEC.md und Originalprompt bleiben bytegleich.

## Neues Ergebnis und Kontobestätigung

- Der Nutzer hat den erfolgreichen App-Login und die Konto-Prüfung bestätigt. Das ist ein Nutzerbericht, keine automatisierte Abnahme von Refresh, Kontowechsel oder privaten/gated Repositories.
- **Hugging Face → Website im Studio** zeigt die echte Website in einem nativen Kind-WebView. Eigener persistenter Profilordner, Navigation/Zurück/Vorwärts/Neuladen, externe HTTPS-Links im Standardbrowser, Übergabe erkannter Modellseiten an die bestehende Modellansicht.
- App-OAuth und Website-Login sind getrennt. Keine Token-/Cookie-Übertragung. Die Website hat keine privilegierten App-Capabilities. Website-Dateidownloads bleiben bis zum Downloadmanager deaktiviert.

## Verifikation 0.3.0

- Frontend-/Tauri-/NSIS-Build erfolgreich; **25 Rusttests und 8 Frontendtests bestanden**.
- [Native Prüfung der paketierten EXE](.artifacts/native-1789608618032/report.json): **26/26 bestanden**. 15 Core-Prüfungen, sechs reale öffentliche HF-/Kontostatus-Prüfungen, fünf Browser-Prüfungen. Isolierte Profile; keine Nutzermodelle oder Medien verwendet.
- Browser-Nachweise: echte HF-Website gerendert, vier privilegierte IPC-Aufrufe aus der Website tatsächlich durch ACL abgewiesen, externe Adresse in der Adressleiste abgewiesen, echte Modell-Details aus Website-Übergabe, Verbergen/Wiederöffnen und veraltete Owner-Bereinigung geprüft, Bounds bei UI-Zoom passend, synthetischer Cookie nach Neustart vorhanden bei weiterhin getrenntem App-Kontostatus.
- Hauptansicht und Website wurden im vorangegangenen Browserlauf separat visuell geprüft; CDP bildet native Kind-WebViews nicht im Screenshot der Hauptansicht ab. Keine vollständige Accessibility-/Fenstergrößenabnahme.
- Ein früher Gesamtlauf scheiterte an einer Testannahme: Der bereits angemeldete Standardbrowser schloss OAuth schneller ab als die Portprüfung. Die ausschließlich im isolierten Testprofil entstandene Anmeldung wurde wieder gelöscht. Der Systembrowser-Login-Test ist jetzt separat opt-in; der finale öffentliche API-Lauf verwendet ihn nicht. Historische Callback-/Abbruchnachweise aus 0.2.1 bleiben dokumentiert.
- Paket-CRC, genau drei Programmdateien, UTF-8-Anleitung, EXE-Identität und SHA-256 geprüft. Keine Profile/Secrets im ZIP. Installer gebaut, Installation/Deinstallation weiterhin ungeprüft; kein vollständiger Rust-Abhängigkeitsaudit.

## Aktuelle Pakete

[Portable ZIP](releases/Local-Studio-0.3.0-hub-portable.zip), [Direktstart](<releases/Local Studio Hub 0.3.0/Local Studio.exe>), [Windows-Setup](releases/Local-Studio-0.3.0-hub-setup.exe), [Prüfsummen](releases/SHA256SUMS-0.3.0.txt).

Für ein Update App schließen, die drei Programmdateien im bisherigen Ordner ersetzen und **Local-Studio-Data behalten**. Die bestehende Nutzerversion und deren Anmeldung wurden nicht verändert.

## Weiter offen

Echte Website-Anmeldung und deren Neustartpersistenz wurden nicht mit einem Konto getestet; der Cookie-Test belegt nur die Persistenzmechanik. Refresh, echter Kontowechsel, private/gated Zugriffe und erfolgreicher manueller Token-Login bleiben gesonderte Tests. Nächstes M1-Arbeitspaket: Downloadmanager und lokale Modellverwaltung. Alle weiteren Anforderungen aus M0–M12 bleiben in PLAN.md erhalten.

---

## Historischer Stand 0.2.1


Stand: 17. September 2026, **0.2.1 Core + Hugging Face**. M0 bleibt teilweise offen; der erste M1-Pfad für Suche, Modell-Details und Kontoanbindung ist implementiert. M1 ist insgesamt nicht abgenommen. SPEC.md und Originalprompt sind unverändert und bytegleich.

## Aktuelles Ergebnis 0.2.1

- Unter **Modelle**: echte Hugging-Face-Suche, Aufgaben-/Sortierfilter, Pagination, Repo-Metadaten, Lizenz-/Zugangshinweise, exakte Commit-Auflösung, Dateiliste mit bekannten Größen/Hashes und Model Card als sicherer Originaltext.
- Unter **Hugging Face**: Kontoanzeige, erweiterter Token-Anmeldepfad mit echter Online-Validierung, Windows-Anmeldespeicher, lokale Abmeldung und erneute Konto-Prüfung. Der native Testspeicher ist vom Nutzerprofil getrennt.
- OAuth-Codepfad mit Public Client, PKCE, zufälligem State, temporärem Loopback-Callback, Abbruch, Timeout und Refresh ist implementiert. **Browserlogin ist aktiviert.** Nach ausdrücklichem „ja“ wurde Local Studio einmalig bei Hugging Face registriert: Client-ID `322a729f-1375-4456-8b0e-d3dbea8b8d31`, Scope `openid profile read-repos`, Authentifizierung `none`, kein Client-Geheimnis. [Registrierungsdaten](docs/huggingface-oauth-client.json). Die zuvor abgelehnte Registrierung ist damit abgeschlossen.
- Keine Modelle heruntergeladen, installiert oder ausgeführt. Der Ausführbarkeitsfilter zeigt korrekt keine kompatiblen Modelle. Website-Links öffnen den Standardbrowser; der geforderte eingebettete Website-WebView bleibt offen.

## Nachweise 0.2.1

- Frontend-/Tauri-/NSIS-Build erfolgreich; **21 Rusttests**, **8 Frontendtests** bestanden.
- [Native Release-Prüfung](.artifacts/native-1789607515635/report.json): **22/22 bestanden**, an einer isolierten Kopie der tatsächlich paketierten EXE. Alle bisherigen 15 Core-Prüfungen plus sieben HF-Prüfungen.
- Echte HF-API: 40 unterschiedliche Suchtreffer über zwei Seiten, Aufgabenfilter für Spracherkennung, Modell `openai-community/gpt2` auf Commit `607a30d783dfa663caf39e06633721c8d4cfcd7e`, MIT-Lizenz, bekannte Dateigrößen und Model Card. Fehlende Revision und falscher Token abgewiesen. Keine Testtokens in SQLite oder Logs. Kein erfolgreicher Login simuliert.
- Windows Credential Manager: reale Speicherung/Wiederöffnung/Löschung eines synthetischen Credentials in separatem Testprofil. Lokaler HTTP-Callback nimmt passende Antworten an und weist gefälschte States/Hosts, Duplikate und falsche Methoden ab. Test für späten Login nach Abbruch, HTTPS-Redirect-Verweigerung, Antwortgrenzen und geheimnisfreie Fehler bestanden.
- Screenshots 05/06/07 des vorangegangenen 0.2.0-Laufs wurden visuell geprüft; das Layout blieb in 0.2.1 unverändert: Konto-/Such-/Dateiansicht lesbar, keine Überlappung in der geprüften Fenstergröße. Noch keine vollständige Accessibility-/Layoutabnahme.
- In der 0.2.0-Entwicklung scheiterte ein früher Lauf am Testvergleich von Playwright-Fehlern; ein weiterer fand beim Pfadtest eine andere geöffnete Ansicht vor. Testvergleich und explizite Navigation wurden korrigiert; diese Läufe gelten nicht als bestandene Produktprüfungen. Ein beim Bearbeiten entstandener UTF-8-Fehler wurde vor dem finalen Build behoben; der originale Hash-Referenzwert ist wieder bestätigt.
- `npm audit`: keine bekannten JavaScript-Schwachstellen. Vollständiger Rust-Abhängigkeitsaudit und Installer-Installation/-Deinstallation bleiben offen.

## Pakete 0.2.1

- [Portable ZIP](releases/Local-Studio-0.2.1-hub-portable.zip): 5.165.626 Bytes, SHA-256 `3187bc96b281ca4b15790758f245096695cd975715df265298f8e81eb25fe9b9`.
- [Windows-Setup](releases/Local-Studio-0.2.1-hub-setup.exe): 3.908.458 Bytes, SHA-256 `582f371370970ae828d7bece359e3a435fe69a7d579eb37e1bbb6e8a6a0ff1f0`.
- [Direktstart](<releases/Local Studio Hub 0.2.1/Local Studio.exe>) und [Prüfsummendatei](releases/SHA256SUMS-0.2.1.txt). ZIP-CRC, genau drei Programmdateien, UTF-8-Anleitung und EXE-Identität geprüft. Keine Profile oder Credentials im Paket.
- Update: bisherige App schließen, drei Programmdateien im bisherigen Ordner ersetzen, **Local-Studio-Data behalten**. Laufende Nutzerversionen wurden nicht beendet oder überschrieben.

## Nächste Schritte und Grenzen

Die Registrierung und das neue Paket sind abgeschlossen. Als nächsten Kontotest in 0.2.1 unter Hugging Face auf „Mit Hugging Face anmelden“ klicken und den Login/Zustimmung im Standardbrowser selbst abschließen. Der automatisierte Testlogin wurde abgebrochen und verwendet ein separates Profil. Positive Token-/OAuth-Anmeldung, echter Kontowechsel, Token-Erneuerung und gated/private Zugriffe sind noch nicht Ende-zu-Ende bestätigt. Keine Zugangsdaten im Chat anfordern.

Der native Test belegt den erfolgreichen Windows-Browseraufruf, einen während des Logins aktiven Loopback-Port, die Ablehnung eines gefälschten Callbacks und das Schließen des Ports bei Abbruch. Ein [anonymer Provider-Check](.artifacts/oauth-provider-0.2.1.json) belegt HTTP 302 zur HF-Loginseite und dort HTTP 200. Die Browsersteuerung war technisch nicht erreichbar; eine visuelle Prüfung der externen Seite wird daher nicht behauptet.

Danach Downloadmanager und lokale Modellverwaltung gemäß M1. Einzelpfade, Shortcut-Konfiguration und übrige offene M0-Anforderungen bleiben erhalten. Das komplette M0–M12-Ziel bleibt in PLAN.md und SPEC.md bestehen. Details der neuen Grenze: [HUGGING_FACE.md](docs/HUGGING_FACE.md).

---

# Historischer Stand 0.1.1 (nachfolgend unveränderte frühere Nachweise)

Stand: 17. September 2026, Core 0.1.1. Der erste M0-Core-Pfad ist implementiert und auf Windows nativ geprüft. M0 bleibt insgesamt teilweise offen; M1–M12 sind nicht abgenommen. Die vollständige Spezifikation wurde unverändert aus `prompt` nach `SPEC.md` übernommen; SHA-256 beider Dateien: `2e3232ef5a607123326d3732679e28a5758106a5d8db61a7420b2e47b71d4bdd`.

## Ergebnis dieses Arbeitspakets

Eine echte Windows-Anwendung mit Tauri 2, React/TypeScript, Rust und SQLite startet und bietet Einrichtung, Einstellungen, Hardwareinventar und Datei-Hashjobs. Der Hashjob läuft in einem eigenen Prozess, meldet tatsächlichen Fortschritt und unterstützt Abbruch. Einstellungen und Jobs bleiben über Neustarts erhalten; ein harter Abbruch wird als Recovery-Fall erkannt. Es wurden keine KI-Modelle heruntergeladen, integriert oder als kompatibel getestet.

| Bereich | Geplant | Implementierung | Prüfung |
| --- | --- | --- | --- |
| Vollständige SPEC und Meilensteine | Ja | Vorhanden | SPEC bytegleich mit Quelle per SHA-256 geprüft |
| Typisierter IPC-Vertrag | Ja | Rust und `src/types.ts` | Native Einrichtung, Settings und Jobbefehle im E2E geprüft |
| Windows-App und lokale Oberfläche | Ja | Implementierter Core-Pfad | Frontend-/Tauri-/NSIS-Build 0.1.1; 15/15 native Release-Prüfungen bestanden |
| SQLite-Einstellungen, Sprache, Theme, Akzent, Skalierung | Ja | Implementiert | DE/EN/Theme, Zoom/Reset, ungültiges Speichern und Neustartpersistenz nativ bestanden; neue Regression für konkurrierendes Speichern/Zoom und Schließen während Speicherung/Pfadaktualisierung bestanden |
| Datenstamm und abgeleitete Unterordner | Ja | Implementiert; Einzelpfade noch offen | Rusttests für Speicherbaum/Reparatur; native Einrichtung und Stammwechsel bei erhaltenem Originalordner/stabiler DB geprüft |
| CPU/RAM/GPU/VRAM/Laufwerksinventar | Ja | Implementiert | Windows 11, Ryzen 9 9950X3D und RTX 4080 nativ erkannt; unbekannte VRAM-Werte als unbekannt |
| Hintergrund-Hashjob, Status, Abbruch, Fehler | Ja | Eigener Prozess, Queue, Abbruch, Timeout, Fehler; alle aktiven Jobs bleiben sichtbar | Echter SHA-256 gegen Referenz; fehlende Datei abgewiesen; wartender und laufender Job abgebrochen; Ergebnis in UI; zwei neue Rusttests für Listenbegrenzung und Sortierung |
| Sessionmarker, unterbrochene Jobs, Logs | Ja | Implementiert | Sauberer Neustart und Crash-Recovery geprüft; keine vorgetäuschte Fortsetzung |
| Vollständige M0-Anforderungen | Ja | Teilumfang offen | Nicht vollständig abgenommen |
| Modelle, Downloadmanager und Hugging-Face-Login | Ja | Nicht implementiert | Nicht getestet |
| Bild/Video/Audio, Galerie/Projekte, Assistent/Analyse, Workflows | Ja | Nicht implementiert | Nicht getestet |
| 18+-Sperre und App-Updates | Ja | Nicht implementiert | Nicht getestet |
| Release-Installer/portable Ausgabe | Ja | Release-EXE und NSIS-Installer gebaut; portable Core-Verpackung | Release-EXE mit portable Marker nativ geprüft; ZIP-Inhalt/CRC/EXE-Identität geprüft; Installer nicht installiert/deinstalliert |

## Korrekturen und Nachweise für 0.1.1

Version 0.1.1 behebt zwei konkrete Zuverlässigkeitsfehler: Ältere aktive Aufträge bleiben auch bei mehr als 250 Einträgen sichtbar; nur abgeschlossene Historie wird auf 250 begrenzt. Während manuellem Speichern werden weitere Einstellungsänderungen und Zoom gesperrt, damit kein veralteter Stand gerade gespeicherte Einstellungen überschreibt. Das Schließen wartet außerdem auf die Speicherung und die Aktualisierung der Speicherpfade. Der Button heißt nun „Prüfsumme berechnen“; er berechnet SHA-256 und ist kein Virenscan oder Modell-Sicherheitscheck.

- `npm run build` und Tauri-/NSIS-Build 0.1.1: erfolgreich.
- `npm test`: 8/8 Frontendtests bestanden.
- Rusttests über `scripts/desktop.ps1 test` / `cargo test --manifest-path src-tauri/Cargo.toml`: 12/12 bestanden. Zwei neue Datenbanktests prüfen unter anderem 301 aktive plus 300 abgeschlossene Jobs, erhaltene aktive IDs/Abbruchfähigkeit, die Begrenzung der Historie und stabile Sortierung.
- [Release-E2E 0.1.1](.artifacts/native-1789605849765/report.json): 15/15 bestanden. Die paketierte EXE wurde als Kopie mit portable Marker, eigener Datenbank und eigenem WebView-Profil geprüft. Neu sind verzögertes Speichern mit konkurrierendem Zoom sowie Schließen während echter Speicherung und verzögerter Pfadaktualisierung, einschließlich Neustartpersistenz.
- [Reproduktion am alten 0.1.0-Release](.artifacts/native-1789605675623/report.json): Der gezielt verzögerte echte IPC-Transport reproduziert den Einstellungsverlust (`dark` statt gespeichertem `light`). Backenddaten werden nicht simuliert. Ein vorausgehender Testharness-Timeout zählt nicht als Produktfehler.
- Das portable ZIP enthält genau drei Programmdateien mit gültiger CRC, bytegleicher getesteter EXE und UTF-8-Anleitung für 0.1.1. Beide Paketprüfsummen stimmen mit `SHA256SUMS-0.1.1.txt` überein.
- Keine unbehandelten Frontend-Laufzeitfehler im nativen Test. Screenshot 04 des neuen Release-Laufs wurde visuell geprüft: die Beschriftung „Calculate checksum“ ist ohne Überlappung lesbar. Eine vollständige UI-/Accessibility-Abnahme bleibt offen.
- Der NSIS-Installer ist gebaut; Installation und Deinstallation sind weiterhin ungetestet.

Die [begrenzte Sicherheitsprüfung](docs/SECURITY_REVIEW.md) hält Umfang und Grenzen fest. Im geprüften Core wurde kein hoher oder kritischer Sicherheitsfehler bestätigt. `npm audit` meldete am 17. September 2026 keine bekannten JavaScript-Abhängigkeitsschwachstellen. Ein vollständiger Rust-Abhängigkeitsaudit wurde nicht durchgeführt. Hugging-Face-Login und KI-Funktionen sind weiterhin nicht implementiert.

## Historische Nachweise für 0.1.0

- `npm run build`: erfolgreich; Produktions-Frontend gebaut.
- `npm test`: 8 Frontendtests bestanden.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 10 Rusttests bestanden, einschließlich Single-Instance-Sperre und Reparatur eines nicht verfügbaren Datenstamms.
- [Debug-E2E](.artifacts/native-1789604766537/report.json): 11/11 erfolgreich. [Release-E2E](.artifacts/native-1789604938456/report.json): 13/13 erfolgreich am 17. September 2026; zusätzlich Tastaturzoom/Reset und Datenstammwechsel mit erhaltenem Originalverzeichnis und stabiler Datenbank. Die tatsächliche Release-EXE lief als Kopie in einem isolierten Ordner mit `portable.marker`. Beide Berichte enthalten Hardware und den tatsächlich berechneten Referenzhash.
- Tauri-Release-Build und NSIS-Erstellung erfolgreich: `src-tauri/target/release/local-studio.exe` und `src-tauri/target/release/bundle/nsis/Local Studio_0.1.0_x64-setup.exe`. Der Installer wurde nicht installiert oder deinstalliert.
- Einrichtung und Einstellungen wurden anhand nativer Screenshots visuell geprüft; kein Überlauf in den geprüften Ansichten. Im E2E keine unbehandelten Frontend-Laufzeitfehler.

Die statische Backendprüfung führte zu Korrekturen: exklusiver Sessionlock vor DB-Öffnung, feste absolute NVIDIA-Probewege, serialisierter Jobstart/Abbruch, Dateiprüfung außerhalb des Worker-Locks, asynchrone native I/O-Commands, Worker-Bereinigung bei Fehlern und reparierbarer Start trotz fehlendem Datenstamm.

## Core 0.1.1 zum Ausprobieren

- [Portable ZIP](releases/Local-Studio-0.1.1-core-portable.zip), 3.784.957 Bytes: enthält ausschließlich `Local Studio.exe`, `portable.marker` und `LIESMICH.txt`. CRC-Prüfung erfolgreich; eingebettete EXE bytegleich mit der getesteten Release-EXE; UTF-8-Anleitung geprüft.
- [NSIS-Setup](releases/Local-Studio-0.1.1-core-setup.exe), 2.864.161 Bytes: erfolgreich gebaut; Installation und Deinstallation wurden nicht durchgeführt.
- [SHA-256-Prüfsummen](releases/SHA256SUMS-0.1.1.txt) für beide Pakete. Entpackter Direktstart: [Local Studio.exe](<releases/Local Studio Core 0.1.1/Local Studio.exe>).

Die vorhandene laufende 0.1.0-Anwendung und ihre Daten wurden nicht verändert. Zum Aktualisieren einer bestehenden portablen Kopie die Anwendung schließen und die drei Dateien aus dem ZIP im bisherigen Programmordner ersetzen; den Ordner `Local-Studio-Data` erhalten. Der separate Direktstartordner von 0.1.1 verwendet eine eigene portable Konfiguration.

Dies sind frühe Core-Builds ohne KI-Modelle. Die portable Anwendung benötigt Windows x64 mit WebView2 und einen beschreibbaren entpackten Ordner.

## Offene Grenzen und nächster Schritt

Der IPC-Vertrag umfasst noch keine Modelladapter, Inferenz, Downloadjobs, Projektcontainer oder Medieneditoren. Der Datei-Hashjob beweist diese Fähigkeiten nicht. `StoragePaths` beschreibt abgeleitete Verzeichnisse; freie Einzelpfadkonfiguration, frei belegbare Shortcuts mit Konflikterkennung sowie vollständige Einrichtung mit Hugging Face, Modellscan und Assistentenmodell bleiben offen. Undo- und Aufbewahrungswerte sind Einstellungen für spätere Funktionen; tatsächliche Medien-Undo-/Bereinigungsabläufe fehlen noch.

Die Recovery-Grundlage erkennt unterbrochene Jobs. Wiederherstellung von Ebenen, Medien, Prompts oder Timelinezuständen, periodische Recovery-Punkte und deren Schutzregeln sind noch nicht implementiert. Große Modelljobs, Offline-Inferenz, vollständige Ressourcenverwaltung, Accessibility-Abnahme und M12-Distribution sind nicht durch die Core-Tests abgedeckt.

Als Nächstes stehen die noch offenen M0-Punkte und anschließend M1 gemäß `PLAN.md` an. Die Core-Pakete sind erstellt und das portable Archiv ist geprüft. Installer-Installation/-Deinstallation, signierte Updates und vollständige M12-Abnahme bleiben offen. Es liegt derzeit keine dokumentierte Nutzerentscheidung als Blockade vor.
