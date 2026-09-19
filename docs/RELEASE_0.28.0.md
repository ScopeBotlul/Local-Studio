# Local Studio 0.28.0

- Automatische Updatesuche zeigt neue Versionen als Popup. **Abbrechen** schließt es; **Update installieren** lädt und prüft das Paket und startet anschließend den bestehenden sicheren Beenden-/Installationsablauf. Ungespeicherte Arbeit wird berücksichtigt. Portable öffnet den ZIP-Download auf GitHub.
- Projektübersicht auf der Startseite mit aktuellem Projekt und zuletzt geöffneten Dateien: Neues Projekt, Durchsuchen und ausgewähltes Projekt öffnen.
- Neues-Projekt-Dialog mit sauber angeordnetem, breitem Namensfeld.
- Bildstudio: Feld heißt **Prompt**, Negativ-Prompt standardmäßig aufgeklappt.
- Eigene SDXL-Größen bis **4.194.304 Pixel insgesamt**, maximal 4096 pro Seite, weiterhin mindestens 256 und 64er-Schritte. Ungültige Eingaben zeigen den Grund und bieten passende Maße zum ausdrücklichen Übernehmen an, z. B. 1920 × 1088 statt 1920 × 1080.
- Sichtbare Modellsuchfilter für **Bild, Video, Audio & Musik, Sprache, Text & Chat und Analyse** mit insgesamt 20 Unteraufgaben. Suche und lokale Ausführbarkeit werden getrennt ausgewiesen.

## Prüfstand und Grenzen

Gezielte Frontend- und Rust-Prüfungen für die geänderten Bereiche sowie TypeScript-/Frontend-Build erfolgreich. Fünf Prüfungen der tatsächlichen Windows-EXE bestanden: Projektübersicht mit echten Projektdateien, Dialoglayout, Größenfelder/Prompts, Modellfilter und keine unbehandelten Frontendfehler. Fünf zusätzliche Update-Dialogfälle in einem isolierten Browser mit simulierten Updateantworten bestanden; dabei wurde kein Installer ausgeführt. Unveränderte KI-, Editor- und Installer-Gesamtsuiten wurden entsprechend dem Nutzerwunsch nicht vollständig wiederholt. Die Suchfilter verwenden Hugging-Face-Metadaten und sind keine Kompatibilitätsgarantie.

Die tatsächliche SDXL-Generierung mit vier Megapixeln ist noch nicht abgenommen. Der PNG-Ausgabepfad wurde mit einer echten 2048 × 2048-RGBA-Datei geprüft. Größere Bilder benötigen mehr VRAM und können bei Speichermangel abbrechen. Die bereits bei 0.27.0 RAM-bedingt offene vollständige native Generierungs-/18+-Sperrprüfung bleibt offen. Keine vollständige Produktabnahme.

Bildgenerierung unterstützt vollständige SDXL-Checkpoints. KI-Video, Musikgenerierung und weitere geplante Modelladapter sind noch nicht integriert. Die entsprechenden Filter dienen bereits der Suche und dem Download.

## Installation

- Installer: `Local-Studio-0.28.0-hub-setup.exe`
- Portable: `Local-Studio-0.28.0-hub-portable.zip`
- Beim portablen Update App schließen, Programmdateien und alle vier Runtime-Ordner ersetzen. **Local-Studio-Data vollständig behalten.** Keine Modellgewichte oder persönlichen Daten in den Paketen.
- Update-Metadaten sind mit dem bestehenden Local-Studio-Schlüssel signiert. Windows-Dateien besitzen weiterhin keine Authenticode-Signatur; SmartScreen kann warnen.
