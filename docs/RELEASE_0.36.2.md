# Local Studio 0.36.2

## Heruntergeladene Modelle automatisch übernehmen

- Erfolgreich heruntergeladene und geprüfte Modellgewichte werden direkt in die lokale Modellbibliothek eingetragen.
- Dadurch erscheinen neue Hugging-Face- und Civitai-Downloads ohne manuellen Ordnersuchlauf in der Modellansicht und in den passenden Studio-Auswahlen.
- Bereits mit früheren Versionen vollständig heruntergeladene Modelle werden beim ersten Start automatisch nachgetragen.
- Technische Komponenten und nicht unterstützte Dateien behalten ihre vorhandene Klassifizierung. Modelldateien werden weiterhin geprüft, bevor sie veröffentlicht und übernommen werden.
- Schlägt nur die Bibliotheksübernahme fehl, bleiben die bereits geprüften Dateien erhalten und der Download kann die Übernahme über **Erneut versuchen** wiederholen.

## Prüfumfang

Sechs gezielte Download-Integrationstests sind bestanden. Ein neuer Test überträgt eine echte minimale GGUF-Datei über den Downloadpfad und prüft sowohl die sofortige SQLite-Übernahme als auch das Nachtragen eines bestehenden abgeschlossenen Downloads beim nächsten Start. Der TypeScript-/Vite-Produktionsbuild ist ebenfalls bestanden. Ein mehrgigabytegroßes Modell wurde nicht erneut übertragen.
