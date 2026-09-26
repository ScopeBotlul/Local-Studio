# Local Studio 0.35.0

## Änderungen

- ComfyUI kann jetzt direkt im gemeinsamen Update-Center aktualisiert werden. Das Update startet ausschließlich nach einem Klick auf **Jetzt aktualisieren**; die automatische Startprüfung installiert weiterhin nichts.
- Local Studio verwendet die eingebettete Python-Laufzeit und den offiziellen stabilen Updater der ausgewählten portablen ComfyUI-Installation. Eine von Local Studio gestartete Engine wird vorher beendet und anschließend wieder gestartet.
- Während des Updates bleiben der laufende Status und das lokale Updater-Protokoll sichtbar. Der Dialog kann erst nach Abschluss geschlossen werden; der Prozess ist an Local Studio gebunden und auf 30 Minuten begrenzt.
- Laufende Bildaufträge und extern gestartete ComfyUI-Instanzen blockieren das Update, damit keine aktive Verarbeitung unterbrochen oder eine fremd verwaltete Engine beendet wird.
- Der offizielle Core-Updater kann erforderliche ComfyUI-Kernabhängigkeiten anpassen. Custom Nodes und deren eigene Abhängigkeiten werden nicht aktualisiert. Der bisherige **Update-Ordner öffnen**-Befehl bleibt in den Einstellungen als manueller Ausweichweg erhalten.

## Aktualisierung

Installierte Ausgaben können das signierte Local-Studio-Update im Update-Center herunterladen und installieren. Portable Ausgaben ersetzen die Programmdateien und alle vier Runtime-Ordner aus dem ZIP; `Local-Studio-Data` bleibt vollständig erhalten.

## Prüfumfang

TypeScript-/Frontend-Produktionsbuild, 14 gezielte Frontendtests, drei bestehende ComfyUI-Rusttests, Rust-/Tauri-Compilercheck und fünf Browser-Szenarien des Update-Centers sind bestanden. Die Browser-Prüfung umfasst den neuen direkten Updateablauf mit Status, Log und Schließsperre sowie die bestehenden Local-Studio-Updatefälle. Ein echtes Update der persönlichen ComfyUI-Installation und eine anschließende Bildinferenz wurden nicht ausgeführt.
