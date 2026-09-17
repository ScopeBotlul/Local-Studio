# Architektur und aktueller Vertrag

## Grenzen und Datenfluss

Local Studio verwendet einen Tauri-2-Desktophost mit React/TypeScript für die Oberfläche und Rust für native Funktionen. SQLite speichert lokale Einstellungen und Jobzustände. `src/types.ts` definiert den gemeinsamen JSON-Vertrag. Die UI fordert native Arbeit über explizite Commands an; sie erhält keine freie Shell oder beliebiges System-API.

Das implementierte M0-Arbeitspaket führt einen realen SHA-256-Dateijob in einem eigenen Prozess aus: dieselbe ausführbare Datei startet im Modus `--worker`. Queue, tatsächlicher Fortschritt, Abbruch und Crash-Recovery wurden in der nativen Windows-App geprüft. Der Hash-Worker ist kein Modelladapter und kein Beweis für funktionierende Inferenz. Die vollständige Inferenzarchitektur mit separaten Modell-Runtimes, Laden/Entladen und Ressourcenregeln folgt vor dem jeweiligen KI-Pfad.

Ein gemeinsames zukünftiges Job-/Medien-/Adaptersystem bedient Image/Edit/Cutout/Upscale/Erase und Video/Animate/Motion/Lipsync/Extend. Ein Adapter beschreibt Familie, Aufgaben, Dateien/Revision, Runtime, Ein-/Ausgaben, Parameter, Ressourcen sowie Lade-, Ausführungs-, Fortschritts-, Abbruch- und Fehlerverhalten. Modellauswahl bleibt von tatsächlich geladenen Gewichten getrennt.

## IPC in M0

Die Schreibweisen der Argumente und Rückgaben entsprechen `src/types.ts`; `src-tauri/src/lib.rs` registriert diese Commands. Die native Windows-Prüfung verwendete den tatsächlichen IPC für Einrichtung, Einstellungen, Hardware, Jobs und Recovery. Teure native Arbeit läuft über `spawn_blocking`; Dateiprüfungen erfolgen außerhalb des Worker-Locks.

| Command | Argument | Rückgabe / Zweck |
| --- | --- | --- |
| `bootstrap` | keines | `AppSnapshot`: Version, Einstellungen, Pfade, Hardware, Jobs, Recovery, Datenbankpfad und portable Status |
| `save_settings` | `{ settings: Settings }` | `Settings`: persistierte Einstellungen |
| `get_hardware` | keines | `HardwareInfo`: CPU/Kerne/RAM/OS, GPUs, Laufwerke, Warnungen |
| `list_jobs` | keines | `Job[]`: persistierte Jobübersicht |
| `enqueue_hash_job` | `{ path: string }` | `Job`: Datei-Hashjob einreihen |
| `cancel_job` | `{ id: string }` | `void`: Abbruch anfordern |
| `dismiss_recovery` | keines | `void`: Recovery-Hinweis quittieren |
| `get_logs` | keines | `string`: lokale Logausgabe |
| `mark_clean_exit` | keines | `void`: regulären Sitzungsabschluss markieren |

`Settings` enthält Sprache (`de`/`en`), Theme (`system`/`light`/`dark`), Akzent, UI-Skalierung, Datenstamm, Sitzungsrestore, Einrichtungsstatus, Undo-Limit und temporäre Aufbewahrungsdauer. Das Vorhandensein eines Felds bedeutet nicht, dass alle zugehörigen späteren Workflows bereits ausgeführt werden.

`StoragePaths` nennt Modelle, Assistenten-/Visionmodelle, Downloads, Galerie, Projekte, temporäre Daten, Recovery, Cache und Proxys. Aktuell wird ein Datenstamm verwendet; die in der Spezifikation geforderte freie Konfiguration jedes Einzelpfads bleibt ein eigener offener Schritt. Datenmigrationen müssen Kopieren, Integritätsprüfung, atomare Zuordnung und erst anschließend Entfernen der alten Kopie berücksichtigen.

`Job` enthält ID, Art, Eingabepfad, Status, optionalen Fortschritt, Zeitpunkte, Ergebnis und Fehler. Zustände: `queued`, `running`, `completed`, `failed`, `cancelled`, `interrupted`. Dieser Vertrag ist noch kein vollständiger Generierungs-Snapshot. Vor M2 kommen unveränderliche Modellrevision, Erweiterungen, Parameter und Eingabemedien hinzu. Echte Pipeline-Fortsetzung wird nur bei nachgewiesenem Checkpoint-Resume angeboten.

GPU-VRAM/Treiber und Fortschritt dürfen `null` sein. Unbekannte Messwerte werden nicht geschätzt oder als getestete Kompatibilität ausgegeben. NVIDIA, AMD und Intel erkennen zu können ist von einem getesteten Inferenzbackend zu trennen.

## Job- und Sitzungslebenszyklus

`core.rs` hält vor Datenbanköffnung einen exklusiven Dateilock auf `config/session.lock`; eine zweite Instanz mit derselben Konfiguration wird abgewiesen. `database.rs` führt SQLite mit WAL und vollständiger Synchronisierung. Beim Start markiert es zurückgebliebene wartende/laufende Jobs als unterbrochen. Ein separater Clean-Exit-Marker unterscheidet regulären Abschluss und Crash. Die Konfigurationsdatenbank bleibt bei Änderung des Mediendatenstamms an ihrem bisherigen Ort; vorhandene Medien werden nicht automatisch verschoben.

Die Jobübersicht liefert seit 0.1.1 alle aktiven Jobs und höchstens die neuesten 250 abgeschlossenen Einträge in gemeinsamer stabiler Reihenfolge. Ein Scheduler bearbeitet die Hashqueue nacheinander. Auswahl, Start und Abbruch sind durch die Lockfolge Workerzustand → Datenbank serialisiert. Worker sprechen ein versioniertes JSON-Zeilenprotokoll über Pipes. Der Parent prüft Ready-/Fortschritts-/Ergebnisnachrichten und beendet einen 30 Sekunden nicht antwortenden Worker. Abbruch beendet und wartet auf den Kindprozess, bevor der Job bestätigt wird; verspätete Ergebnisse überschreiben keinen abgebrochenen Status.

Der Worker prüft Dateigröße und Änderungszeit vor/nach dem Lesen. Ein EOF-Wächter auf stdin beendet ihn, wenn der Parent verschwindet. Schedulerfehler bereinigen verbliebene Worker. Beim regulären Shutdown wird die Queue gestoppt, ein aktiver Prozess beendet und abgeholt, offene Jobs als unterbrochen markiert und anschließend der Scheduler gejoint. Der native Crash-Test bestätigt die Recovery-Markierung; eine Fortsetzung ab Zwischenstand wird nicht angeboten.

Die Oberfläche serialisiert Einstellungs-Schreibvorgänge. Während manuellem Speichern sperrt sie weitere Änderungen und Zoom; das Schließen wartet auch auf die zugehörige Pfadaktualisierung. Native Regressionstests für 0.1.1 prüfen beide konkurrierenden Abläufe.

Ein fehlender oder unbeschreibbarer gespeicherter Datenstamm verhindert den Core-Start nicht. Der gespeicherte Pfad bleibt erhalten, und `AppSnapshot.hardware.warnings` enthält derzeit den Speicherhinweis. Über Einstellungen lässt sich ein beschreibbarer Stamm speichern; erfolgreiche Reparatur entfernt die interne Warnung. Ein gesonderter typisierter Storage-Status kann den momentan gemeinsamen Warnungskanal später ersetzen.

## Sicherheitsgrenzen

- Privilegierte Capabilities gehören nur zum lokalen Hauptfenster. Der externe Hugging-Face-WebView erhält keine System-IPC; OAuth-App-Login und Website-Cookies bleiben getrennt.
- Einstellungen und Ergebnisse bleiben lokal. Keine standardmäßige Telemetrie; keine automatischen Medien-/Prompt-Uploads.
- Native Commands validieren ihre Argumente und behandeln Dateisystem-/Datenbankfehler verständlich. Noch nicht vorhandene Importformate erhalten vor Freigabe Pfad-, Archiv- und Inhaltsvalidierung.
- Tokens liegen im Windows Credential Manager, niemals in normale Einstellungen. Fremde Model Cards, READMEs und Workflowdateien sind Daten, keine Anweisungen zur Codeausführung.
- Abgeleitete Medien überschreiben keine Originale. Entfernen von Nutzerdaten sowie die spezifizierten Assistenten-Löschaktionen benötigen die vorgesehenen Bestätigungen.

## Dokumentation und Quellen

`SPEC.md` ist die unveränderte Produktspezifikation; `PLAN.md` ordnet M0–M12. Status, Testnachweise und tatsächlich getestete Modelle stehen getrennt in `PROJECT_STATUS.md`, `TEST_MATRIX.md` und `MODEL_COMPATIBILITY.md`. Nicht implementierte Anforderungen bleiben offen.

Offizielle Tauri-Dokumentation für den initialen Stack:

- [Windows-Voraussetzungen](https://v2.tauri.app/start/prerequisites/)
- [Rust aus dem Frontend aufrufen](https://v2.tauri.app/develop/calling-rust/)
- [Native Datei- und Ordnerdialoge](https://v2.tauri.app/plugin/dialog/)

Diese Quellen begründen die Plattformintegration. Die historischen Nachweise für Core 0.1.0 sind getrennt dokumentiert: Produktions-Frontend-/Tauri-Release-Build, 8 Frontendtests, 10 Rusttests, [11 Debug-E2E-Prüfungen](../.artifacts/native-1789604766537/report.json) und [13 Release-E2E-Prüfungen](../.artifacts/native-1789604938456/report.json). Die Release-EXE wurde als Kopie mit portable Marker nativ geprüft. NSIS-Setup und portable ZIP sind gebaut. Das Archiv enthält die bytegleiche geprüfte Release-EXE und hat die CRC-/Inhaltsprüfung bestanden. Installation/Deinstallation des Setups, signierte Updates und vollständiges M0/M12 bleiben offen.


Core 0.1.1 ergänzt zwei Rust-Regressionsprüfungen (insgesamt 12/12) und besteht [15/15 native Release-Prüfungen](../.artifacts/native-1789605849765/report.json) sowie 8/8 Frontendtests. Die aktualisierten Paket- und Sicherheitsnachweise stehen in [PROJECT_STATUS.md](../PROJECT_STATUS.md), [TEST_MATRIX.md](../TEST_MATRIX.md) und [SECURITY_REVIEW.md](SECURITY_REVIEW.md).


## Hugging Face ab 0.2.0

`src/HubPage.tsx`, `hub-api.ts`, `hub-types.ts` und `hub-i18n.ts` bilden Konto- und Suchoberfläche. `hub_commands.rs` kapselt die expliziten Commands `hf_status`, `hf_start_login`, `hf_cancel_login`, `hf_connect_token`, `hf_logout`, `hf_verify`, `hf_search`, `hf_model_detail`, `hf_open_page`. Sie sind ausschließlich in `main-local` freigegeben. Der Status liefert niemals Tokens. Blockierende HTTPS-/Windows-Operationen laufen auf dem nativen Hintergrundpool.

`hub.rs` begrenzt HTTPS-Ziele, Antworten, Weiterleitungen und Repo-Eingaben. `hf_auth.rs` trennt lokal gespeicherte Identität, Onlineprüfung und laufenden Login. Credential Manager ist der einzige persistente Token-Speicher. Generationen verhindern verspätete Login-/Refresh-Commits nach Abmeldung; ein Refresh-Lock verhindert doppelte Rotation. Beim App-Schließen wird ein offener OAuth-Callback abgebrochen.

`src-tauri/huggingface-oauth.json` enthält ab 0.2.1 die nach ausdrücklicher Genehmigung registrierte öffentliche Client-ID. Kein Client-Geheimnis wird eingebaut; die App registriert sich nicht bei jedem Start neu. Ab 0.3.0 kapselt `hf_browser.rs` einen persistenten nativen Child-WebView. `HfBrowserPanel.tsx` übermittelt seine sichtbaren Bounds und serialisiert Mount/Layout/Hide mit Owner-IDs gegen veraltete React-Cleanups. Die Capability matcht ausschließlich `webviews: ["main"]`, nicht das gemeinsame Fenster. Website und App-OAuth teilen keine Zugangsdaten. Siehe [HUGGING_FACE.md](HUGGING_FACE.md).


## Interner OAuth-Start ab 0.3.1

`HfAuth::start_login` übergibt die nativ erzeugte Autorisierungs-URL und den temporären Rückruf an `hf_browser`. Der Controller navigiert den bestehenden WebView oder speichert das Startziel bis zum Mount. Nur der aktive, generationengebundene Loopback-Endpunkt wird zusätzlich zugelassen. Abschluss/Abbruch bereinigt dieses Ziel; Frontend-Polling schaltet zurück zur Kontoansicht. Zugangsdaten und Codeaustausch bleiben im Rust-/Windows-Secret-Store-Pfad. Die ausdrückliche Nutzerentscheidung für den eingebetteten OAuth-User-Agent und ihre Abweichung von RFC 8252 sind in HUGGING_FACE.md dokumentiert.


## Downloadmanager 0.4.0

`downloads.rs` verwaltet typisierte native Vorschauen, eine serialisierte Download-Warteschlange und `downloads.sqlite3` im Konfigurationsordner. Ein asynchroner Transfer nutzt `watch`-Abbruchsignale für echte HTTP-Unterbrechung. Fortschritt/Status werden persistiert; aktive Zustände werden beim Neustart pausiert. Datenpfade sind pro Auftrag festgeschrieben. Nur lokale `main`-Capabilities erlauben die vier Download-Commands.

`DownloadSelection` holt eine Vorschau ohne Dateivorauswahl und bestätigt danach eine kurzlebige native Plan-ID. `DownloadsPage` zeigt Transfers oder abgeschlossene lokale Auswahlen. HF-Dateimetadaten enthalten nun zusätzlich Git-Blob-IDs. LFS-SHA-256 bzw. Git-Blob-Hash plus dauerhaft erfasster SHA-256 sichern die Byte-Integrität. Vor Veröffentlichung wird auf dem Modelllaufwerk eine separate Kopie geprüft und umbenannt. Kein Modell wird dadurch ausgeführt. Details: [DOWNLOADS.md](DOWNLOADS.md).
