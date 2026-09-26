# Local Studio 0.34.0

## Änderungen

- Das Update-Center enthält jetzt getrennte Bereiche für **Local Studio** und **ComfyUI** mit installierter und verfügbarer Version sowie eigenen Aktionen.
- Die bestehende Einstellung **Beim Start automatisch nach Updates suchen** gilt für beide Programme. Ist für Local Studio oder ComfyUI eine neue Version verfügbar, öffnet sich das gemeinsame Update-Center. Die Suche installiert nichts automatisch.
- Die installierte ComfyUI-Version wird aus der lokalen Versionsdatei beziehungsweise der laufenden lokalen API gelesen, ohne Python-Code auszuführen. Die verfügbare Version stammt aus dem offiziellen ComfyUI-GitHub-Release.
- Ein ComfyUI-Update öffnet nur nach einem bewussten Klick den offiziellen Updater-Ordner der portablen Installation. Custom Nodes und deren Abhängigkeiten werden nicht automatisch aktualisiert.
- Die ComfyUI-Statusabfrage läuft außerhalb des Desktop-Hauptthreads, damit eine beendete lokale API das Update-Center nicht blockiert.

## Aktualisierung

Installierte Ausgaben können das signierte Local-Studio-Update im Update-Center herunterladen und installieren. Portable Ausgaben ersetzen die Programmdateien und alle vier Runtime-Ordner aus dem ZIP; `Local-Studio-Data` bleibt vollständig erhalten.

## Prüfumfang

TypeScript-/Frontend-Produktionsbuild, 14 gezielte Frontendtests, drei ComfyUI-Rusttests, Rust-/Tauri-Compilercheck und fünf Browser-Szenarien des Update-Centers sind bestanden. Die Browser-Prüfung umfasst beide Spalten, die gemeinsame automatische Einstellung, die explizite ComfyUI-Updater-Aktion, Local-Studio-Download/Abbruch/Integritätsfehler und portable Updates. Ein echtes ComfyUI-Update wurde nicht ausgeführt.
