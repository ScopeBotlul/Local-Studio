# Local Studio 0.33.1

## Änderungen

- Der Button **Herunterladen und installieren** wird während der ComfyUI-Einrichtung selbst zur Fortschrittsanzeige. Er zeigt Release-Prüfung, echten prozentualen Downloadfortschritt, Prüfsummenprüfung und Installation direkt an.
- Der Status wird während des laufenden Desktop-Befehls regelmäßig aktualisiert. Das funktioniert sowohl im ersten Einrichtungsdialog als auch im ComfyUI-Bereich der Einstellungen.
- Nach erfolgreicher Prüfung und Installation speichert Local Studio den entpackten ComfyUI-Ordner weiterhin automatisch als lokale Engine und startet ihn. Eine zusätzliche Ordnerauswahl ist nicht erforderlich.

## Aktualisierung

Installierte Ausgaben können das signierte Update im Update-Dialog herunterladen und installieren. Portable Ausgaben ersetzen die Programmdateien und alle vier Runtime-Ordner aus dem ZIP; `Local-Studio-Data` bleibt vollständig erhalten.

## Prüfumfang

TypeScript-/Frontend-Produktionsbuild, gezielter Downloadübersichtstest sowie Rust-/Tauri-Compilercheck wurden für die betroffenen Pfade ausgeführt. Die bereits geprüfte Download-, Hash- und Installationslogik aus 0.33.0 blieb unverändert. Ein vollständiger Mehrgigabyte-ComfyUI-Download und reale Inferenz wurden nicht wiederholt.
