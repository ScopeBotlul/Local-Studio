# Local Studio 0.35.4

## ComfyUI-Start auf Windows-Handhelds

- Local Studio startet portable ComfyUI-Installationen jetzt mit dem offiziellen Windows-Standalone-Modus. Das zuvor fehlende Argument `--windows-standalone-build` wird ebenso wie `python.exe -s` verwendet.
- Das ComfyUI-Konsolenfenster bleibt unsichtbar. Standardausgabe und Fehlermeldungen werden stattdessen lokal gespeichert.
- Beendet sich ComfyUI bereits beim Start, wartet Local Studio nicht mehr die volle Frist ab. Unter **Einstellungen → ComfyUI → Startprotokoll anzeigen** steht die konkrete lokale Fehlermeldung, beispielsweise zu AMD ROCm oder einem Grafiktreiber.
- Das Protokoll wird bei jedem Start neu angelegt, auf die letzten 64 KiB begrenzt angezeigt und nicht übertragen.

## Prüfumfang

Frontend-Produktionsbuild, 12 direkt betroffene Frontendtests und Rust-/Tauri-Compilercheck sind bestanden. Der offizielle Startbefehl wurde mit den aktuellen ComfyUI-Windows-Skripten abgeglichen. Ein echter Start auf dem ROG Ally bleibt auf dem Gerät zu prüfen; ein verbleibender AMD-/ROCm-Fehler ist durch das neue Startprotokoll nun konkret sichtbar.
