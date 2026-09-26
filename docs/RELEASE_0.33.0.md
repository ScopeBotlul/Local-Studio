# Local Studio 0.33.0

## Änderungen

- ComfyUI Portable kann jetzt direkt im Einrichtungsdialog heruntergeladen und installiert werden. Zur Auswahl stehen die offiziellen Windows-Pakete für NVIDIA, NVIDIA CUDA 12.6, AMD und Intel.
- Local Studio ermittelt Paketgröße und SHA-256-Prüfsumme über das offizielle ComfyUI-GitHub-Release, begrenzt Weiterleitungen auf die erwarteten GitHub-Server und übernimmt nur ein vollständig geprüftes Archiv. Unsichere Archivpfade und Reparse Points werden abgewiesen.
- Download, Prüfung und Installation erscheinen mit Fortschritt, Geschwindigkeit und Phase im Download-Menü. Nach erfolgreicher Einrichtung wird der lokale ComfyUI-Pfad gespeichert und die Engine gestartet.
- Der neue Button **Nach Installation suchen** durchsucht typische lokale ComfyUI-Verzeichnisse und ausgewählte Benutzerordner mit fester Tiefen- und Eintragsgrenze. Die manuelle Ordnerwahl bleibt verfügbar.
- Local Studio installiert keine Custom Nodes oder zusätzlichen Modellabhängigkeiten.

## Aktualisierung

Installierte Ausgaben können das signierte Update im Update-Dialog herunterladen und installieren. Portable Ausgaben ersetzen die Programmdateien und alle vier Runtime-Ordner aus dem ZIP; `Local-Studio-Data` bleibt vollständig erhalten.

## Prüfumfang

Frontend-Produktionsbuild, Rust-/Tauri-Kompilierung und die neuen Rusttests für Paket-Allowlist und Archivpfade sind bestanden. Die offizielle Release-API wurde gegen die vier erwarteten Windows-Pakete einschließlich SHA-256-Digests geprüft. Ein echter Mehrgigabyte-Download, das Entpacken eines vollständigen ComfyUI-Pakets, der Start der frisch installierten Engine und reale Bildinferenz sind noch nicht Ende-zu-Ende ausgeführt. Unveränderte Galerie-, Projekt-, Assistent- und Installer-Lebenszyklus-Suiten wurden nicht wiederholt.
