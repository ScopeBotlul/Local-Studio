# Testergänzung 0.4.1

- Frontend-/Desktop-/NSIS-Build erfolgreich, **10 Frontendtests** und **35 Rusttests** bestanden.
- [Native Prüfung der paketierten EXE](.artifacts/native-1789611130190/report.json): **34/34 bestanden**. Echte HF-Suchkarte zeigt GB; Gesamtgröße stimmt mit der Summe der Dateien des konkreten GPT-2-Commits überein. Kleine echte Download-Auswahl und lokale Liste zeigen **< 0.01 GB**. Keine unbemerkten Frontendfehler.
- Fehlende/teilweise Größen, leere Listen, Null, kleine Werte, DE/EN und Überlaufgrenzen getestet. Neuer Größen-Command ist für Website-WebView gesperrt; vorhandene Download-/Browser-/Login-Abbruch-/Core-Prüfungen erneut durchlaufen.
- Erster Rustlauf: vorhandener lokaler HTTP-Test `cancellation_keeps_partial_and_bad_hash_never_publishes` schlug mit `download_resume_unsupported` statt `download_hash` fehl. Ohne Codeänderung bestand der vollständige Wiederholungslauf. Ursache dieses sporadischen Testfehlers ist nicht geklärt; nicht als behoben gewertet.
- ZIP-Inhalt/CRC, Identität der getesteten EXE, UTF-8-Anleitung und Paketprüfsummen verifiziert. Installer gebaut, Installation/Deinstallation nicht geprüft. Private/gated Größen und große Repositories gesondert offen; bei Abruffehlern wird unbekannt angezeigt.

---

# Testergänzung 0.4.0

[Native Prüfung der ausgelieferten EXE](.artifacts/native-1789610509905/report.json): **34/34 bestanden**. **34 Rusttests**, **8 Frontendtests**, Frontend-/Desktop-/NSIS-Build erfolgreich. Paket-CRC/Inhalt, EXE-Identität, UTF-8-Anleitung und SHA-256 bestätigt.

| Bereich | Implementierung | Verifikation |
| --- | --- | --- |
| Integrierter App-Login | Seit 0.3.1 vorhanden | Nutzer bestätigt tatsächlichen Login; keine automatische Refresh-/SSO-Abnahme |
| Dateiauswahl/Vorschau | Nativ aufgelöste Auswahl, Commit/Größe/Lizenz/Ziel, einmalige Plan-ID | Native UI ohne Vorauswahl; Vorschau startet keinen Transfer |
| Reale HF-Dateien | Rust HTTPS/Redirect-Allowlist/Streaming | 454.671 Bytes aus zwei Dateien auf festem Commit; LFS-SHA-256 und Git-Blob geprüft |
| Pause/Fortsetzung | Asynchroner Abbruch, tatsächliche Dateilänge und strikte Content-Range-Prüfung | Kontrollierter verzögerter HTTP-Transfer stoppt; Wiederöffnung setzt per Range am korrekten Byte fort |
| Fehler/Abbruch/Retry | Teile behalten, expliziter Neustart bei fehlender Resume-Unterstützung | HTTP 200 auf Range verwirft keine Teile; Cancel/Restart/Hashfehler nativ in Rust geprüft |
| Queue/Priorität/Recovery | Ein aktiver Transfer, priorisierte nächste Auswahl, Neustart pausiert | HTTP-Prioritätsfolge und erhaltene vorhandene Zieldaten geprüft |
| Veröffentlichung/lokale Liste | Geprüfte Kopie + Rename, keine Runtime-Behauptung | Tatsächliche Dateien/Hashes, erkennbare gleich große Korruption, erneute Prüfung und App-Neustart nativ bestanden |
| Sicherheitsgrenzen | Lokale Commands, validierte Windows-Pfade, HF-HTTPS-Hosts, keine CDN-Bearer-Header im Codepfad | Pfad-/Host-/Range-/Hash-Rusttests, reale Remote-ACL-Ablehnung für Downloadliste, manipulierte/verbrauchte Plan-Eingaben abgewiesen |

Offen: tatsächliche private/gated Transfers, Netz-/Disk-Fault-Injection aller Betriebssystemfälle, große Mehrdatei-Repositories, plattformübergreifende Laufwerksmigration, volle Accessibility, kompletter Rust-Audit und Installer-Lebenszyklus. Kein Modell ausgeführt. Ein früher Suchlauf am nicht indexierten internen HF-Repo ist kein bestandener Downloadnachweis; der finale Lauf nutzt dessen echte Website-Übergabe.

---

## Historische Prüfungen bis 0.3.1

# Testergänzung 0.3.1

[Native Prüfung der ausgelieferten EXE](.artifacts/native-1789609392616/report.json): **29/29 bestanden**, dazu **26 Rusttests**, **8 Frontendtests**, erfolgreicher Frontend-/Desktop-/NSIS-Build. ZIP-Inhalt/CRC, EXE-Identität, Anleitung und Paketprüfsummen bestätigt. Originalprompt/SPEC unverändert.

- Echte HF-Loginseite öffnet durch den App-Button im isolierten integrierten WebView; S256-PKCE vorhanden, privilegierte IPC abgewiesen, Passwortformular sichtbar. Keine Zugangsdaten eingegeben.
- Gefälschter Callback-State abgewiesen. Gültige Ablehnung über tatsächliche WebView-Navigation an den exakten Loopback-Endpunkt verarbeitet; Konto bleibt leer, App kehrt zur Kontoansicht zurück.
- Neuer Versuch hat frischen State; Abbruch beendet den Listener. Rusttest schließt falschen Host, Port, Pfad, Schema, Credentials und Fragment von der Callback-Ausnahme aus.
- Alle bisherigen Core-/öffentlichen HF-API-/Website-Prüfungen weiterhin bestanden. Vollständiger erfolgreicher Login mit einem echten Konto im integrierten Browser, Refresh und Drittanbieter-SSO nicht automatisiert geprüft.

---

## Historische Prüfungen bis 0.3.0

# Aktuelle Testergänzung 0.3.0

Stand: 17. September 2026. [Native Release-Prüfung](.artifacts/native-1789608618032/report.json) **26/26 bestanden**, Frontend **8/8**, Rust **25/25**. Die geprüfte EXE ist bytegleich im portablen Paket enthalten.

| Bereich | Implementierung | Prüfung |
| --- | --- | --- |
| App-OAuth | Public Client/PKCE/Windows Credential Manager | Nutzer bestätigt Login und Konto-Prüfung; automatischer Refresh/Kontowechsel weiter offen |
| Echte Website im Studio | Nativer separater Kind-WebView | Reale HF-Seite geladen; getrennte Screenshots visuell geprüft |
| Privilegiengrenze | Nur `webviews: ["main"]`, kein Fenster-/Remote-Match; Caller-Check | Vier echte Remote-IPC-Aufrufe durch ACL abgewiesen; Rust-Capability-Test |
| Navigation/Modellübergabe | HF-Allowlist, kontrollierte Popups/externe Links, Repo-Erkennung | Adressablehnung und echte Detailübergabe nativ; Domain-/Scheme-/IPv4-/IPv6-Grenzen in Rust |
| Lebenszyklus/Layout | Serialisierte Mutationen, Owner-ID, native Bounds | Tabwechsel, stale Owner, Zoom/Bounds nativ bestanden |
| Website-Sitzung | Eigener persistenter Profilordner | Synthetischer Cookie nach App-Neustart im isolierten WebView2-Testprofil vorhanden; kein echter Website-Login-Test |
| Core/API-Regression | Vorhandene reale Implementierung | 15 Core- und sechs HF-Prüfungen bestanden; keine unbehandelten UI-Fehler |
| Distribution | Portable ZIP/NSIS 0.3.0 | CRC, drei Dateien, UTF-8, EXE-Identität und Prüfsummen geprüft; Installer-Lebenszyklus offen |

Der frühere Lauf `.artifacts/native-1789608502556/report.json` gilt nicht als bestanden: Ein bereits autorisierter Standardbrowser schloss den Test-OAuth ab, bevor der erwartete offene Port gemessen wurde. Das Test-Credential wurde entfernt. `LOCAL_STUDIO_TEST_OAUTH_BROWSER=1` ist jetzt ein gesonderter Test für einen nicht angemeldeten Systembrowser; die routinemäßigen API-/Website-Tests starten ihn nicht. Keine Änderung an den Nutzerdaten oder der eigentlichen Nutzeranmeldung.

---

## Historische Testmatrix bis 0.2.1

# Testmatrix

Stand: 17. September 2026, Core + Hugging Face 0.2.1. „Geplant“, „implementiert“ und „getestet“ sind getrennte Aussagen. Ohne konkreten Nachweis gilt ein Test als ausstehend. Die Tabelle bündelt Anforderungen aus `SPEC.md`; Einzelprüfungen werden beim jeweiligen Arbeitspaket ergänzt. Der erste M0-Core-Pfad ist implementiert und nativ geprüft; M0 bleibt teilweise offen. Der erste M1-HF-Pfad ist implementiert; Medien-/KI-Inferenzbereiche bleiben offen.

| ID | SPEC | Meilenstein | Anforderung / erforderlicher Nachweis | Implementierung | Teststatus |
| --- | --- | --- | --- | --- | --- |
| DOC-01 | 0–123 | Alle | Vollständigen Masterprompt unverändert bewahren | `SPEC.md` vorhanden | Bestanden: SHA-256 gleich `prompt`, siehe Status |
| CORE-01 | 0–6 | M0 | Native Windows-App startet; ein Tauri/React/Rust/SQLite-Stack; keine simulierten Funktionen | Core implementiert | Bestanden: Frontend-/Tauri-/NSIS-Build 0.2.1, Release-E2E 22/22; Gesamt-M0 offen |
| CORE-02 | 7–8 | M0 | System/Light/Dark, Akzentfarbe, DE/EN/Fallback, UI-Zoom/Shortcuts; Einstellungen über Neustart | Implementiert | DE/EN/Theme, Zoom/Reset, Neustartpersistenz sowie konkurrierendes Speichern/Zoom und Schließen während Speicherung/Pfadaktualisierung nativ bestanden; Frontendtests; vollständige UI-Variantenabnahme offen |
| CORE-03 | 9 | M0/M3/M4 | Frei belegbare zentrale Shortcuts, Konflikte, Undo-Limit, Save/Undo/Redo/Ansicht | Skalierung und gespeichertes Undo-Limit vorhanden; übrige Funktionen geplant | Frontendtests bestanden; Shortcut-Neubelegung/Medien-Undo ausstehend |
| CORE-04 | 10–11 | M0/M1/M5 | Einrichtung, Datenstamm und alle Einzelpfade; portable/Windows-Nutzerpfade; keine ungefragten großen Downloads | Einrichtung/Datenstamm implementiert; Einzelpfade/HF/Modelle offen | Einrichtung, Stammwechsel mit Originalerhalt/stabiler DB und portable Releasebetrieb nativ bestanden; Einzelpfade ausstehend |
| CORE-05 | 1, 26 | M0/M2 | CPU/RAM/GPU/VRAM/Disks real erkennen; unbekannte Werte; Kapazitätswarnung und optionale Livewerte | Inventar implementiert; Kapazitäts-/Livefunktionen geplant | Nativ Ryzen 9 9950X3D, RTX 4080, AMD/virtuelle Adapter und Laufwerke erkannt; unbekanntes VRAM korrekt unbekannt |
| CORE-06 | 4, 23, 112 | M0 | Echter SHA-256-Job: Referenzhash, Queue, Status, Fehler, Abbruch; UI bleibt bedienbar | Implementiert | Bestanden: echter Worker-Hash, sichtbares Ergebnis, wartender/laufender Abbruch und fehlende Datei; zwei Rusttests für aktive Jobs über dem 250er-Historienlimit und Sortierung; kein Inferenznachweis |
| CORE-07 | 52–54, 107–108 | M0/M3 | Sauberer Exit, Crashmarker/unterbrochene Jobs, Logs; später Sitzungs-/Medien-Recovery, Bereinigung mit Schutzregeln | Marker/Jobs/Logs implementiert; Medien-Recovery/Bereinigung geplant | Bestanden: sauberer Neustart und harter Abbruch mit unterbrochenem Job; vollständige Medien-Recovery ausstehend |
| MOD-01 | 12, 14–16 | M1 | HF-Suche/Pagination/Filter/Model Cards/Lizenzen, Repo/Revision/Variante; verwaltbar vs. ausführbar | Suche/Details/Commit/Dateien implementiert; Download/Verwaltung offen | Reale Suche (40 Treffer), Pagination, Aufgabenfilter, GPT-2-Details/Model Card, fehlende Revision und keine behauptete Runtime nativ bestanden |
| MOD-02 | 12–13 | M1 | OAuth/Logout/Kontowechsel, geschützter Token, falscher Token/gated ohne Zugang; HF-WebView-Sitzung ohne privilegierte IPC | Tokenpfad/Windows-Store und OAuth-Code implementiert; Client registriert, Browserlogin aktiviert; WebView offen | Falscher Token real abgewiesen; synthetischer Store-/Callback-Test bestanden. Echte positive Kontoanmeldung, Refresh, gated/private und WebView ungeprüft. Registrierung nach ausdrücklicher Freigabe abgeschlossen; nativer Browserstart und Callback-Abbruch geprüft |
| MOD-03 | 17–19 | M1 | Manueller Scan, Kandidat vs. erkannt, Import ohne Verschieben, fehlendes Modell/Laufwerk; Kopieren-Prüfen-Umschalten-Löschen | Geplant | Ausstehend; Crash/Abbruch während Verschieben testen |
| MOD-04 | 20–22 | M1/M2 | Updates nur auf Aktion, alte Daten bis Prüfung; Erweiterungen kontextbezogen; Downloads Pause/Resume/Abbruch/Retry/Priorität/Recovery | Geplant | Ausstehend; Netzwerkausfall und voller Datenträger testen |
| JOB-01 | 23–25 | M2/M7 | A läuft, B wählen/einreihen: A bleibt A; A/B/A-Einstellungen; kein Re-Download; ausgewählt vs. geladen | Geplant | Ausstehend |
| JOB-02 | 24, 26–27, 112–113 | M2/M7 | Echter Fortschritt/Phasen, Tabstatus, Ressourcenprüfung, OOM/Timeout/Worker-Shutdown, lokale Benchmarks, verständliche Fehler | Geplant; Core-Fehlerbasis implementiert | Ausstehend |
| GAL-01 | 34–39 | M2/M3 | Echte Galerieordner, sichere Pfadänderung, Explorer-Watcher, Drag/Strg+Drag, eindeutige Namen, temporäre Ergebnisse und Werkzeugtransfer | Geplant | Ausstehend; andere Laufwerke/Kollisionen testen |
| GAL-02 | 40–43, 117 | M3 | Metadaten/Einstellungen wiederherstellen, Tags/Favoriten/Suche, Raster/Liste, virtualisierte Thumbnails, Medienviewer | Geplant | Ausstehend |
| GAL-03 | 44–47 | M3 | Original erhalten, Herkunft/Versionsgruppen/Vorher-Nachher, bestätigter Papierkorb/Restore/Endlöschung | Geplant | Ausstehend; Original löschen darf andere Versionen nicht löschen |
| PRJ-01 | 48–51 | M3 | `.localstudio`-Container inkl. Medien, ohne Gewichte, exakte Referenzen; manuell speichern; projektlos arbeiten | Geplant | Ausstehend; auf zweitem PC ohne Modelle öffnen und Installation ablehnen |
| PRJ-02 | 50, 52–54, 107–108 | M3 | Undo-bezogene Containerbereinigung, Sitzungsrestore, Crash-Recovery, letzte 3 Recovery-Punkte, 7-Tage-Temporärregeln, ein Exitdialog | Geplant | Ausstehend; App während Arbeit hart beenden |
| ADULT-01 | 55–58 | M1/M2/M3/M5/M6 | Standard aus, gehashte PIN/Passwortsperre, Neustartsperre, modellbasierte Blur-/Metadatensperre, keine neue Analyse/Indextreffer gesperrter Medien | Geplant | Ausstehend; laufende Jobs bei Sperren erhalten |
| IMG-01 | 21, 59 | M2/M4 | Reale lokale T2I/I2I/Referenzbild-Ausgabe, echte Capability-Parameter, Seed, Auflösung, Varianten, LoRA/ControlNet/VAE | Geplant | Ausstehend; echtes Modell und Ergebnisdatei erforderlich |
| IMG-02 | 60–66 | M4 | Ebenen/Blend Modes, Transformation, Auswahl/KI-Auswahl/Maskenkorrektur, Pinsel/Korrektur, Text/Vektor, Navigation/Undo | Geplant | Ausstehend |
| IMG-03 | 67–70 | M4 | PNG/JPEG/WebP/TIFF/Alpha, RAW-Entwicklung und Quellerhalt, ICC, 16/32 Bit, HDR/SDR-Tone-Mapping | Geplant | Ausstehend; Farben und Bittiefe tatsächlich prüfen |
| IMG-04 | 71–73 | M4 | Echte Denoise/Restore/Colorize/Upscale/Cutout/Erase/Extend-Modelle; Batch je Datei, Retry, Ziel/Dateinamenvorschau/Kollisionen | Geplant | Ausstehend |
| IMG-05 | 74–78 | M4 | Formatoptionen/Presets, EXIF/GPS/Profile/Gen-Metadaten, Resize, Wasserzeichen aus, mehrere Exportvarianten | Geplant | Ausstehend; Original unverändert |
| AST-01 | 10, 28–31 | M5 | Lokales DE/EN-Chatmodell, definierte Toolaufrufe, zusammengehörige Aktionen, gesonderte Löschbestätigung, Präferenzen verwalten | Geplant | Ausstehend; explizite Modell-/Nutzerwahl und 18+-Sperre testen |
| ANA-01 | 32–33, 58 | M6 | Optionale lokale Bild-/Video-/Audioanalyse, Mehrmedienvergleich, bedarfsabhängiger Index, Änderungserkennung, Modi/geschützte Treffer | Geplant | Ausstehend |
| VID-01 | 79–80, 101 | M7 | Reale Videoerzeugung, zwei wechselbare Modelle gleicher Aufgabe, Animate und tatsächlich verarbeitete Motionparameter | Geplant | Ausstehend; beide Ausgaben und A/B/A-Nachweis |
| VID-02 | 81–84 | M8 | Mehrspur, Schnitt/Trim/Übergänge, Keyframes/Effekte, Proxy-Schalter; Export nutzt Original | Geplant | Ausstehend |
| VID-03 | 85–90 | M8/M9 | Lokale Untertitel/SRT/Diarization, Burn-in, Szenengrenzen, Stille-Schnittliste, Reframe und Tracking | Geplant | Ausstehend |
| VID-04 | 91–95 | M9 | Video-Inpainting/Matting/Interpolation/Restore/Stiltransfer, zeitliche Konsistenz und geeignete Alpha-Ausgabe | Geplant | Ausstehend |
| VID-05 | 96–98 | M8/M9/M10 | Mehrspur-Audio/DSP/Stems, unterstützte Container/Codecs/Hardware-Encoding, Lipsync/Gesichtsauswahl/Tracking/Versatz | Geplant | Ausstehend |
| AUD-01 | 99–100 | M10/M9/M4 | Reale wechselbare Musikmodelle, Waveform/Mehrspur/Schnitt/DSP, Stems, Image/Video/Music-Extend erzeugt neue Inhalte | Geplant | Ausstehend |
| WF-01 | 102–105 | M11 | Ein Graph für Schritt-/Node-Ansicht, Modell-/Parameterwahl, `.lsworkflow`, Referenzen ohne Gewichte, fehlende Abhängigkeiten | Geplant | Ausstehend; importierter Code/Shellbefehl muss abgewiesen werden |
| WF-02 | 103, 106 | M11/M0 | Abschaltbare, nicht wiederholt störende Vorschläge; nützliche Startseite mit realem Status | Startbasis implementiert; Vorschläge geplant | Native UI ohne Laufzeitfehler; Vorschläge ausstehend |
| SEC-01 | 3, 109–111 | Alle | Keine Uploads/Telemetrie standardmäßig; untrusted Cards/Archive/Projekte/Workflows, Pfadtraversal/Überschreiben, kein remote code/eval/Shell | Core-Grenzen implementiert; weitere Eingänge geplant | Settingsvalidierung, fehlende Datei und Sessionlock geprüft; begrenzte Core-Prüfung dokumentiert, npm audit 0, vollständiger Rust-Abhängigkeitsaudit offen; Modell-/Archiv-/Workfloweingänge ausstehend |
| SYS-01 | 2, 48 | M12 | Installer/portable, Dateizuordnung, kryptografisch geprüfte GitHub-Updates, portable ohne Selbstupdate | Release-EXE/NSIS gebaut; portable Marker implementiert; weitere Teile geplant | Release-Core mit portable Marker und ZIP-Inhalt/CRC/EXE-Identität geprüft; Installieren/Deinstallieren, Dateizuordnung und signierte Updates ausstehend |
| SYS-02 | 115–117 | M12/je Feature | Offline nach vollständiger Runtimeinstallation, große Bibliotheken/Galerien, flüssige UI, Accessibility, progressive Optionen | Geplant | Ausstehend |
| DOC-02 | 118–123 | Alle | SPEC/Plan/Status/Kompatibilität/Tests pflegen; aktuelle Originalquellen, reale Tests, kein Scopeverlust oder erfundene Fertigmeldung | Dokumente vorhanden | Fortlaufend; Core-Nachweise unten dokumentiert |

## Prüfprotokoll für Core 0.1.1

| Prüfung | Implementierung / Ziel | Ergebnis und Nachweis |
| --- | --- | --- |
| Frontend-Build | Produktionsoberfläche | Bestanden: `npm run build` |
| Frontendtests | Bestehende Verhaltenstests | Bestanden: `npm test`, 8/8 |
| Rusttests | Core und zwei neue Joblisten-Regressionen | Bestanden: 12/12 über `scripts/desktop.ps1 test` / Cargo |
| Aktive Jobs trotz großer Historie | Alle aktiven Jobs plus höchstens 250 abgeschlossene Jobs; stabile gemeinsame Sortierung | Bestanden: zwei neue Rusttests, darunter 301 aktive und 300 abgeschlossene Jobs, eindeutige IDs und Abbruch aller sichtbaren aktiven Jobs |
| Speichern bei gleichzeitigem Zoom | Speicher-/Bearbeitungssperre und serialisierte Schreibvorgänge | Bestanden im nativen 0.1.1-Release; [alter 0.1.0-Release reproduziert den Fehler](.artifacts/native-1789605675623/report.json) mit verzögertem echtem IPC, ohne simulierte Backenddaten |
| Schließen während Speicherung | Speicherung und Pfadaktualisierung vor Exit abwarten | Bestanden nativ: tatsächliche Schreiboperation und Pfadaktualisierung verzögert, danach sauberer Neustart mit gespeicherten Werten |
| Native Windows-Release-App | Paketierte 0.1.1-EXE in isolierter portabler Kopie | Bestanden: [15/15 Prüfungen](.artifacts/native-1789605849765/report.json), einschließlich Hash, Abbruch, Persistenz, Crash-Recovery und ohne unbehandelte Frontendfehler |
| Native visuelle Prüfung | Neue Beschriftung „Calculate checksum“ | Bestanden im betrachteten Screenshot 04; lesbar ohne Überlappung; vollständige UI-/Accessibility-Abnahme offen |
| Release-/NSIS-Build | EXE und Setup 0.1.1 | Bestanden; Installation/Deinstallation weiterhin ausstehend |
| Portable ZIP | Drei Programmdateien, CRC, EXE-Identität, Anleitung | Bestanden: getestete EXE bytegleich, UTF-8-Anleitung für 0.1.1, ZIP 3.784.957 Bytes |
| Paketprüfsummen | ZIP und Setup 0.1.1 | Bestanden: [SHA256SUMS-0.1.1.txt](releases/SHA256SUMS-0.1.1.txt) entspricht beiden Paketen; Setup 2.864.161 Bytes |
| Begrenzte Sicherheitsprüfung | Vorhandener M0-Hash-Core | Kein hoher/kritischer Sicherheitsfehler im geprüften Umfang bestätigt; [Umfang und Grenzen](docs/SECURITY_REVIEW.md) |
| JavaScript-Abhängigkeiten | Bekannte Advisories | `npm audit` am 17. September 2026: 0 bekannte Schwachstellen |
| Rust-Abhängigkeiten | Vollständiger Dependency-Audit | Ausstehend; Rusttests sind kein Sicherheitsnachweis für alle Abhängigkeiten |
| Hugging-Face-Login / Modelle / KI | M1 und folgende Meilensteine | Nicht implementiert und nicht getestet |

Ein vorausgehender Timeout aufgrund des Testharness wurde nicht als Produktfehler gewertet. Die Core-0.1.0-Nachweise unten bleiben als historische Basis erhalten; die aktuellen Ergebnisse stehen oben.

## Historisches Prüfprotokoll für Core 0.1.0

| Prüfung | Ergebnis | Nachweis |
| --- | --- | --- |
| SPEC entspricht Originalprompt | Bestanden | Beide Dateien SHA-256 `2e3232ef5a607123326d3732679e28a5758106a5d8db61a7420b2e47b71d4bdd` |
| Frontend-Build | Bestanden | `npm run build`: Produktions-Frontend erfolgreich gebaut |
| Verhaltenstests Frontend | Bestanden | `npm test`: 8/8 Tests |
| Rust-Tests | Bestanden | `cargo test --manifest-path src-tauri/Cargo.toml`: 10/10 Tests, inkl. Sessionlock und fehlender-Datenstamm-Reparatur |
| Native Windows-App | Bestanden | [Debug-E2E](.artifacts/native-1789604766537/report.json): 11/11; [Release-E2E](.artifacts/native-1789604938456/report.json): 13/13; echte UI, keine unbehandelten Laufzeitfehler |
| Hash/Persistenz/Abbruch/Recovery | Bestanden | Beide nativen Berichte: echter Hash, fehlende Datei, Queue-/laufender Abbruch, Neustart, Crash-Recovery ohne falsches Resume; Release zusätzlich Zoom/Reset und Stammwechsel |
| Native visuelle Prüfung | Bestanden im geprüften Umfang | Setup- und Settings-Screenshots visuell geprüft; kein Überlauf in diesen Ansichten |
| Release- und NSIS-Build | Bestanden | Tauri erzeugt Release-EXE und NSIS-Setup; Installer nicht installiert/deinstalliert |
| Portable Core-Lauf | Bestanden | Tatsächliche Release-EXE in isoliertem Ordner mit portable Marker; 13/13 native E2E-Prüfungen |
| Portable ZIP | Bestanden | Drei erwartete Dateien; CRC sauber; eingebettete EXE bytegleich mit getesteter Release-EXE; UTF-8-Anleitung und SHA256SUMS geprüft |
| Installer-Installation/Deinstallation | Ausstehend | Setup erfolgreich gebaut, aber nicht installiert oder deinstalliert |

Prüfplattform: Windows 11 Pro, AMD Ryzen 9 9950X3D (32 logische Kerne), NVIDIA GeForce RTX 4080 mit gemeldeten 17171480576 VRAM-Bytes, Treiber 616.56. AMD- und virtuelle Adapter wurden erkannt; deren VRAM blieb unbekannt. Das bestätigt Hardwareerkennung, keine KI-Backend-Kompatibilität.

Technische Tests ersetzen die vollständige Windows-Abnahme nicht. Fehlende Voraussetzungen als konkretes Hindernis dokumentieren; nicht als bestandenen Test behandeln.

## Historischer Nachweis 0.2.0

[Release-E2E](.artifacts/native-1789607116990/report.json): 21/21 native Checks, 21 Rusttests und 8 Frontendtests bestanden. Der Test verwendet eine Kopie von `releases/Local Studio Hub 0.2.0/Local Studio.exe`, eigenes Profil und echte öffentliche API-Aufrufe. Testkonten wurden nicht simuliert. Der Windows-Secret-Store-Test nutzt ausschließlich einen eigenen synthetischen Credential-Eintrag, der danach gelöscht wird. ZIP-Inhalt/CRC/EXE-Identität und Paketprüfsummen geprüft; Setup nur gebaut, nicht installiert.

Im damaligen Build 0.2.0 war der OAuth-Browserlogin mangels Registrierung deaktiviert; diese Grenze ist in 0.2.1 behoben. Positive Kontoanmeldung und Refresh sind nicht Ende-zu-Ende geprüft; siehe [genaue Grenzen](docs/HUGGING_FACE.md). Frühere 0.1.x-Abschnitte sind historische Nachweise.

## Aktueller Nachweis 0.2.1

[Release-E2E](.artifacts/native-1789607515635/report.json): **22/22 native Checks**, **21 Rusttests**, **8 Frontendtests** bestanden. Geprüft wurde die paketierte EXE aus `releases/Local Studio Hub 0.2.1/Local Studio.exe`. Neu: aktiver OAuth-Anmeldebutton, erfolgreicher Windows-Browseraufruf, realer neuer Loopback-Port der isolierten App, HTTP 400 bei falschem State, weiter ausstehender Login, anschließend Abbruch und geschlossener Port ohne angemeldetes Konto. Die übrigen Core-/HF-Prüfungen bestehen weiterhin.

[Anonymer Provider-Check](.artifacts/oauth-provider-0.2.1.json): offizieller Autorisierungsendpunkt antwortet mit HTTP 302 zur Hugging-Face-Loginseite, deren Antwort HTTP 200 ist. Kein Konto, keine Cookies und kein Tokenaustausch verwendet. Die einmalige vorherige Registrierung ist durch ausdrückliche Nutzerfreigabe autorisiert; Antwort siehe [öffentliche Client-Metadaten](docs/huggingface-oauth-client.json).

ZIP-Inhalt/CRC, Anleitung, EXE-Identität und Prüfsummen geprüft. Vollständige echte Kontoanmeldung/Refresh, Installer-Lebenszyklus und Rust-Abhängigkeitsaudit bleiben offen. Der Browser-Automationsdienst war technisch nicht erreichbar; kein visueller Nachweis der externen Loginseite.
