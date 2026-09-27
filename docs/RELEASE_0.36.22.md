# Local Studio 0.36.22

## ComfyUI-Start auf langsameren Geräten

- Der verwaltete ComfyUI-Start wartet jetzt bis zu drei Minuten auf die lokale API. Der bisherige Grenzwert von 30 Sekunden konnte auf Handhelds und PCs mit langsamem Datenträger oder längerer GPU-/Node-Initialisierung einen Fehlstart melden und den noch ladenden Prozess beenden.
- Gleichzeitige Startaufrufe werden serialisiert. Ein Klick auf „ComfyUI starten“ während des automatischen Starts startet dadurch keinen zweiten Server.
- Der Start bleibt im Hintergrund; erst ein tatsächlich beendeter oder nach drei Minuten weiterhin nicht erreichbarer Prozess wird als fehlgeschlagen angezeigt.

## Taskleisten-Icon

- Das zur Akzentfarbe eingefärbte Windows-Fenster- und Taskleisten-Icon verwendet jetzt dieselbe Logoform und dieselben Balkenproportionen wie Local Studio in der App.
- Das Icon wird beim nächsten Programmstart beziehungsweise beim Anwenden der Akzentfarbe aktualisiert. Statische Datei- und Verknüpfungsicons bleiben vom Windows-Iconcache abhängig.

## Gezielte Prüfung und Grenzen

- `npm.cmd run build`: bestanden.
- `npm.cmd test -- --run src/window-icon.test.ts`: 1/1 bestanden.
- `cargo test --manifest-path src-tauri/Cargo.toml comfy::tests`: 6/6 bestanden.
- Optimierter Windows-Build und Inno-Installer: erfolgreich erstellt.
- Portable ZIP enthält 80 erlaubte Dateien und alle vier geprüften Runtime-Manifeste. Installer und ZIP stimmen mit den signierten Größen und SHA-256-Werten überein; das Ed25519-Update-Manifest wurde gegen den eingebetteten öffentlichen Schlüssel geprüft.
- Ein realer ComfyUI-Start auf ROG Ally/Xbox Ally und die sichtbare Taskleistenabnahme unter Windows stehen noch aus. Die lokale Prüfung belegt Kompilierung und bestehende ComfyUI-Regeln, aber keinen Geräte-Start mit dem neuen Zeitlimit.
- Veröffentlicht als Latest: https://github.com/ScopeBotlul/Local-Studio/releases/tag/v0.36.22. Alle fünf Release-Dateien wurden öffentlich zurückgeladen und gegen lokale Größen und SHA-256-Werte geprüft.
