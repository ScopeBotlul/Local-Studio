# Entwicklungsplan

Die vollständige und verbindliche Anforderung steht in `SPEC.md`. Der Umfang bleibt erhalten. Dieser Plan ordnet die Arbeit; spätere Funktionen werden erst sichtbar bedienbar, wenn ihr tatsächlicher Ausführungspfad vorhanden ist. `PROJECT_STATUS.md` beschreibt den aktuellen Stand, `TEST_MATRIX.md` trennt Implementierung und Prüfung.

| Meilenstein | Umfang | Abnahme | Stand |
| --- | --- | --- | --- |
| M0 — Core | Tauri/React/Rust, Navigation, Einstellungen, DE/EN, Theme/Skalierung, SQLite, Hardware, Worker/Jobs, Speicherpfade, Logs, Recovery-Grundlage | Echte Windows-App startet; Einstellungen über Neustart erhalten; Hintergrundarbeit, Fehler und Recovery nachvollziehbar | In Arbeit; erster Core-Pfad nativ geprüft, Rest offen |
| M1 — Modellmanager | Hugging-Face-API, OAuth/geschützte Secrets, isolierter Website-WebView, Suche/Filter/Revisionen, Downloads mit Pause/Resume, Scan/Import, sichere Installation/Verschiebung/Updates | Reales kleines Modell finden und herunterladen/importieren; Fehler-, Zugangs- und Speicherfälle testen | In Arbeit; Suche/Details und Tokenpfad implementiert, OAuth-Client registriert, App-Login nutzerbestätigt; isolierter Website-Browser und Downloadmanager mit lokaler Dateiliste implementiert; Scan/Import/Runtime-Installation weiter offen |
| M2 — Image End-to-End | Getesteter Bildadapter, Modellwechsel, echte Text-to-Image-Ausführung, unveränderliche Jobdaten, temporäre Ergebnisse, Galerie-Speichern, Metadaten, lokale Benchmarks | Reales lokal erzeugtes Bild; Modell A/B/A ohne erneuten Gewichtsdownload; laufender Job bleibt unverändert | Geplant |
| M3 — Galerie und Projekte | Echte Ordner/Watcher, Drag-and-drop, Viewer, Tags/Suche, Thumbnails, Versionen/Herkunft/Vergleich, Papierkorb, `.localstudio`, projektloser Betrieb, Medien-Recovery, konsolidierter Exit | Medien sicher speichern/verschieben/wiederherstellen; Projekt ohne Modelle auf zweitem PC öffnen; Crash-Recovery | Geplant |
| M4 — Bildeditor | Zerstörungsfreie Ebenen/Masken/Auswahl, Text/Vektor, Transformation/Korrektur, KI-Bildbearbeitung, RAW, ICC/HDR, Batch, Export/Metadaten/Wasserzeichen/Varianten | Reale Bearbeitungs- und Exportpfade; Originale erhalten; Alpha/Farben/Bit-Tiefe geprüft | Geplant |
| M5 — Assistent | Aktuell geprüfter kleiner GGUF-Kandidat, llama.cpp, definierte Tools, Aktionen, Hardwareberatung, Metadatensuche, lokale Präferenzen | Lokaler DE/EN-Chat und echte Tool-Aufträge; kein freier Shellzugriff; Löschbestätigung und aktuelle Nutzerwahl gelten | Geplant |
| M6 — Vision/Analyse | Optionale lokale Bild-/Video-/Audioanalyse, Medienvergleich, Transkription, lokaler Inhaltsindex und seine Modi | Analyse mit tatsächlich geprüftem Modell; unveränderte Dateien nicht erneut analysieren; geschützte Inhalte gesperrt | Geplant |
| M7 — Video Generation | Reale Videoerzeugung, mindestens zwei Modelle derselben Unteraufgabe, Animate/Motion, modellspezifische Parameter | Echte lokale Ausgaben und Modellwechsel mit beiden Modellen dokumentiert | Geplant |
| M8 — Timeline Editor | Video-/Audio-Mehrspur, Clips, Keyframes, Effekte/Übergänge, Proxy, Untertitel/Diarization, Tracking, Audio, Export | Zusammengeschnittenes Video mit Originalmaterial exportiert; Proxy-/Timing-/Codecverhalten geprüft | Geplant |
| M9 — Advanced Video | Inpainting, Matting, Reframe, Interpolation, Restaurierung, Stiltransfer, Lipsync, Video-Extend | Reale modellgestützte Ausführung und zeitliche Konsistenz geprüft | Geplant |
| M10 — Music/Audio | Wechselbare Musikmodelle, Waveform/Mehrspur, Schnitt/DSP, Stems, Music-Extend | Reale lokale Musik, Audio-Bearbeitung und neue Fortsetzungsinhalte | Geplant |
| M11 — Workflows | Ein gemeinsamer Graph, Schrittansicht/Node-Editor, `.lsworkflow`, Modellreferenzen, sichere Imports, zurückhaltende Vorschläge | Workflow vollständig lokal ausführen; fehlende Modelle melden; keine importierbaren Shell-/Code-Nodes | Geplant |
| M12 — Packaging | Windows-Installer und portable Ausgabe, Dateizuordnung, signaturgeprüfte GitHub-Updates, Crash-/Offline-/Performance-/Accessibility-Tests | Beide Distributionen auf Windows geprüft; portable Version aktualisiert sich nicht selbst | Geplant |

## Aktuelles Arbeitspaket: erster M1-Hugging-Face-Pfad

Version 0.2.0 ergänzt reale Suche, Details und sichere Kontoanbindung. Nach ausdrücklicher Genehmigung wurde die öffentliche OAuth-App registriert; Version 0.2.1 aktiviert den Browserlogin. Der Nutzer hat App-Login und Konto-Prüfung bestätigt. Version 0.3.0 ergänzt den isolierten persistenten Website-Browser samt Modellübergabe. Auf ausdrücklichen Nutzerwunsch öffnet ab 0.3.1 auch der App-OAuth-Button diesen integrierten Browser. Der Nutzer bestätigt jetzt auch den integrierten App-Login. Version 0.4.0 ergänzt explizite Dateiauswahl, Downloadvorschau, Pause/Resume/Abbruch/Retry/Priorität, Neustart-Recovery und lokale Integritätsprüfung. Refresh, Kontowechsel, Scan/Import und vollständige Modell-/Runtime-Installation stehen weiter aus. Einzelheiten: [Hugging Face](docs/HUGGING_FACE.md). Die noch offenen M0-Punkte bleiben erhalten.

## M0-Grundlage und offene Restpunkte

Ein einziger Tauri-Stack erhält typisierte IPC, lokale SQLite-Persistenz, konfigurierbaren Datenstamm, Hardwareinventar und einen echten SHA-256-Dateijob als überprüfbaren Hintergrundpfad. Der Hashjob ist keine KI-Inferenz. Einstellungen, DE/EN, System/Light/Dark, Akzentfarbe und Skalierung sowie Abbruch, Logs und unterbrochene Jobs bilden die erste Oberfläche.

Noch vor vollständiger M0-Abnahme separat schließen: Windows-Laufprüfung der noch offenen Funktionen, sämtliche individuell veränderbaren Speicherpfade, frei belegbare Shortcuts mit Konflikterkennung, vollständige Einrichtung und robuste Worker-Lebenszyklen. Der aktuelle Datenstamm ersetzt nicht die spezifizierten Einzelpfade. Ein gespeicherter Aufbewahrungswert ersetzt keine sichere Bereinigungsfunktion. Spätere Einrichtungsschritte für Hugging Face und Assistentenmodell werden mit M1/M5 ergänzt.

## Durchgehende Anforderungen

- Lokalität, keine automatischen Medien-/Prompt-Uploads und keine standardmäßige Telemetrie gelten in allen Meilensteinen.
- Eine gemeinsame Medien-/Job-/Adapterarchitektur bedient alle Schnellwerkzeuge; keine zweite Engine für denselben Medientyp.
- Modell-/Datei-/Workflowimporte validieren; keine Ausführung fremder Modellskripte; Lizenzen und Zugangsbeschränkungen berücksichtigen.
- 18+-Sperre aus §§55–58 mit Modellkennzeichnung in M1/M2, Galerie-Metadaten in M3 und Analyse/Assistent in M5/M6 umsetzen; Neustartsperre, geschützte Metadaten und laufende Jobs ausdrücklich testen. Bis dahin nicht als vorhanden ausgeben.
- Ressourcenregeln, unveränderliche Modelljob-Snapshots, verständliche Fehler, tatsächlich gemessener Fortschritt und sichere Originaldateien sind Abnahmekriterien der jeweils betroffenen Funktion.
- Nach jedem Arbeitspaket Status und Testnachweise aktualisieren. Blockaden konkret mit nächstem Schritt festhalten. Kein Meilenstein gilt allein durch sichtbare UI als fertig.
