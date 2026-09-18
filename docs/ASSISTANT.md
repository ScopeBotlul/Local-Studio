# Lokaler Assistent und automatische Untertitel · 0.24.0

## Modelle einrichten

Unter **Assistent → Modelle verwalten** steht Qwen2.5 1.5B Instruct Q4_K_M mit **1,12 GB** zur Auswahl. Im Videoschnitt unter **Automatische Untertitel · lokal → Modelle verwalten** steht Whisper Base multilingual mit **0,15 GB**. Die Angaben sind dezimale Dateigrößen, kein RAM-Bedarf. Modelle sind nicht im Installer enthalten.

1. **Download vorbereiten** zeigt Repository, Größe, zusätzlichen Speicherbedarf und Ziel des vorhandenen Downloadmanagers. Die Dateiauswahl und Revision sind für den Katalog festgelegt.
2. Download ausdrücklich starten. Unter Downloads sind Pause, Fortsetzen und Abbruch möglich.
3. Nach abgeschlossenem und geprüftem Download **Modell verwenden** wählen.
4. Alternativ eine vorhandene GGUF-Datei für Chat beziehungsweise ein whisper.cpp-GGML-Modell als Datei hinzufügen. Eigene Modelle werden registriert; ihre tatsächliche Kompatibilität wird erst beim Laden/Ausführen festgestellt.

Die in der App geprüften Katalogmodelle sind [Qwen2.5-1.5B-Instruct-GGUF](https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF) (Apache-2.0) und [Whisper Base aus ggerganov/whisper.cpp](https://huggingface.co/ggerganov/whisper.cpp) (MIT). Exakte Revisionen, Dateigrößen und SHA-256 stehen in `src-tauri/ai-models.json`. Vor jeder Nutzung werden Dateidentität, Größe und SHA-256 erneut geprüft. Eine Prüfsumme garantiert weder Fehlerfreiheit noch die Sicherheit beliebiger Modelldateien.

Downloads verwenden die eingestellten Assistenten- beziehungsweise Vision-Modellordner. Importierte Dateien bleiben am gewählten Ort. Nach Verschieben oder Verändern ist ein erneuter Import nötig. Die neue Auswahlliste und die allgemeine Modellbibliothek sind derzeit getrennte Referenzlisten.

## Chat und Studio-Werkzeuge

Chatmodell auswählen und **Modell laden** drücken. Nach „Bereit · lokal auf CPU“ eine Nachricht senden. Die App-Sprache bestimmt Deutsch/Englisch als Antwortvorgabe; Antworten erscheinen nach Abschluss der Berechnung. **Entladen** gibt den Arbeitsspeicher frei. Vor einem Modellwechsel zuerst entladen. **Abbrechen** beendet die laufende Antwort und entlädt ebenfalls das Modell.

Mit eingeschalteten Studio-Werkzeugen stehen genau vier lokale Aktionen bereit:

- Hardware abfragen: tatsächlich erkannte CPU, RAM und GPUs.
- Modelle auflisten: erkannte vorhandene Einträge aus der allgemeinen Modellbibliothek, ohne Online-Suche.
- Galerie durchsuchen: Dateinamen, Tags und gespeicherte Prompt-/Modellmetadaten. Es findet keine visuelle Bildanalyse statt.
- Bildauftrag vorbereiten: installiertes SDXL-Modell und geprüfte Parameter in eine Vorschlagskarte übernehmen. **Einstellungen ins Bild-Studio übernehmen** füllt das Studio; **Bild generieren** bleibt ein eigener bewusster Schritt.

Beispiele: „Welche Hardware ist auf meinem PC vorhanden?“, „Suche in meiner Galerie nach Strand“ oder „Liste meine Modelle auf und bereite mit meinem SDXL-Modell ein Bild von einem roten Boot vor.“ Tatsächlich ausgeführte Werkzeuge und ihre Ergebnisse sind aufklappbar. Ohne Studio-Werkzeuge erhält das Modell keine aktuellen Hardware-, Modell- oder Galeriedaten.

Der kleine Kandidat kann sachlich falsche Antworten und unpassende Modellvorschläge geben. Es gibt keine garantierte Hardware-Eignung oder allgemeine Qualitätsabnahme. Ein Bildvorschlag wird strukturell geprüft; die bestehende Bild-Vorprüfung entscheidet weiterhin über die tatsächliche Ausführung. Der Assistent kann keine Shellbefehle ausführen, Dateien löschen, Modelle installieren oder eigenständig Generierungen starten. Online-Hugging-Face-Beratung, gelernte Präferenzen, weitere Studio-Aktionen und vollständiges M5 bleiben offen.

## Sprache in Untertitel umwandeln

1. Projekt öffnen/erstellen und im **Studio → Videoschnitt** ein Video oder eine Audiodatei als Clip einfügen.
2. Den Clip auswählen, **Automatische Untertitel · lokal** öffnen und das Sprachmodell wählen.
3. Gesprochene Sprache auf Deutsch, Englisch oder automatische Erkennung setzen; **Clip transkribieren** starten.
4. Erkannten Text und Zeitintervalle prüfen und bearbeiten. Einzelne Vorschläge lassen sich aus dem Entwurf entfernen.
5. **Untertitel in Timeline übernehmen** fügt die geprüften Zeilen als einen rückgängig machbaren Schritt hinzu. **SRT speichern** schreibt eine neue UTF-8-Datei.

Erkannt wird der ausgewählte Quellausschnitt. Clipstart und Geschwindigkeit werden in die Untertitelzeiten eingerechnet. Bereits vorhandene Untertitel bleiben erhalten. Wurde der Clip oder seine Quelle inzwischen verändert, muss erneut transkribiert werden. Wiederholtes Übernehmen derselben Zeilen wird abgewiesen. Übernommene Zeilen sind im Projekt und in dessen Videoexport enthalten; danach lassen sie sich mit den bisherigen Untertitelwerkzeugen bearbeiten.

Status und Abbruch sind auch unter **Aufträge** sichtbar. Ein Transkriptionsauftrag läuft gleichzeitig. Fortschritt wird nur angezeigt, wenn Whisper ihn tatsächlich meldet; Prüfung und Tonvorbereitung haben keine erfundene Prozentanzeige. Tonlose Medien werden abgewiesen. Grenzen: höchstens 500 Untertitel in einer Timeline, eine Stunde Timeline und die bestehenden Projekt-/Mediengrenzen. Kein Sprecherwechsel-/Diarization-Modell, kein Inhaltsindex und keine Bild-/Videoanalyse. Dies ist der erste Transkriptionspfad aus M6/M8, keine vollständige Abnahme dieser Meilensteine.

Ergebnisse bleiben lokal gespeichert. **Noch nicht übernommene Textkorrekturen im Entwurf gehen beim Verlassen der Ansicht oder Wechsel des Auftrags verloren.** Daher erst übernehmen und das Projekt speichern. Die ursprüngliche automatische Erkennung bleibt im Auftrag erhalten. Spracherkennung kann Fehler produzieren, besonders bei Stille, Musik oder starkem Rauschen; Texte vor dem Export prüfen.

## Daten und Prozesse

Chatverlauf, Modellreferenzen und Transkriptionsaufträge liegen in `config/ai.sqlite3`. Der Verlauf ist lokal gespeichert; nur ein begrenzter jüngster Ausschnitt wird als Modellkontext verwendet (bis zu zwölf Nachrichten und 14.000 UTF-8-Bytes). „Chat leeren“ erfordert eine Bestätigung und ist keine garantierte forensische Datenlöschung. Die letzten 30 Transkriptionsaufträge werden angezeigt. Nach Neustart wird kein Modell automatisch geladen und kein Auftrag automatisch fortgesetzt.

Beide neuen Inferenzpfade laufen auf CPU in eigenen, abbrechbaren Prozessen. Windows-Jobobjekte beenden Kindprozesse mit der App und begrenzen ihren Speicherverbrauch. Chat braucht zusätzlich zum Modell Speicher für Kontext und Runtime. Die lokale Chat-Schnittstelle bindet ausschließlich an Loopback und verwendet einen zufälligen Schlüssel; dieser wird nicht an das Frontend übergeben. Modellskripte, frei übergebene Befehle und die eingebauten llama.cpp-Agentenwerkzeuge sind nicht aktiviert. Das ist Prozessisolierung, keine allgemeine Sandbox für beliebige native Parserfehler.

Für die Inferenz werden keine Nachrichten, Medien oder Modelldateien hochgeladen. Die normale Updateprüfung und ausdrücklich angeforderte Modelldownloads benötigen weiterhin Netzverbindungen. Nach der Modelleinstellung funktioniert die Inferenz offline.

Ton-Zwischendateien liegen vorübergehend unter dem eingestellten temporären Ordner in `speech-jobs/<ID>`. Eigene Arbeitsdateien werden nach normalem Abschluss/Abbruch entfernt. Unbekannte Dateien und Reste eines Prozessabsturzes werden vorsichtshalber nicht pauschal gelöscht. Originalmedien bleiben unverändert. Vollständige Ressourcenkoordination über alle Engines, 18+-Sperre/geschützte Analysemetadaten und allgemeine Aufbewahrung aller KI-Daten bleiben als SPEC-Anforderungen offen.

## Entwicklung und Nachweise

`node scripts/bootstrap-ai-runtimes.mjs` stellt ausdrücklich die gepinnten offiziellen Windows-CPU-Runtimes bereit: llama.cpp b11026 und whisper.cpp b5130. Archiv- und Dateiprüfsummen werden geprüft; Modelle werden dadurch nicht installiert. Installer und Portable enthalten `assistant-runtime` und `speech-runtime` zusätzlich zu Bild-/Video-Runtime. Lizenz-/Quellhinweise liegen in `docs/assistant-runtime` und `docs/speech-runtime`. Keine Runtime wird aus dem System-PATH geladen.

`scripts/prepare-ai-fixtures.ps1` erzeugt echte DE/EN-WAV-Dateien mit installierten Windows-SAPI-Stimmen. `LOCAL_STUDIO_TEST_AI24=1` aktiviert in `scripts/native-smoke.mjs` die reale Inferenzprüfung; sie benötigt das gepinnte Qwen-GGUF unter `.tools/test-models24/`, diese Sprachdateien und einen vorhandenen SDXL-Checkpoint (Pfade über `LOCAL_STUDIO_TEST_CHAT_MODEL`, `LOCAL_STUDIO_TEST_SDXL_MODEL`, `LOCAL_STUDIO_TEST_SPEECH_FIXTURES` überschreibbar). Der Test lädt Whisper bewusst über die echte Downloadwarteschlange. Keine Antworten, Tool-Aufrufe oder Transkriptionen werden simuliert. Exakte Prüfergebnisse und Grenzen stehen getrennt in [TEST_MATRIX.md](../TEST_MATRIX.md).
