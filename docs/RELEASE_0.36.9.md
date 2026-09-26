# Local Studio 0.36.9

## Local-Studio-Modelle zuverlässig in ComfyUI eingebunden

- Die verwaltete ComfyUI-Engine erhält den Local-Studio-Modellordner jetzt als direkten absoluten Checkpoint-Suchpfad.
- Damit kann ComfyUI heruntergeladene Checkpoints auch aus verschachtelten Ordnern wie `models/hf-…/…safetensors` auflisten und laden.
- Windows-Pfade werden weiterhin ohne das interne `\\?\`-Präfix und mit sicherer YAML-Zeichenbehandlung übergeben.
- Modellgewichte werden weder verschoben noch kopiert.

## Prüfumfang

Der gezielte Rust-Test für die ComfyUI-Modellpfadkonfiguration ist bestanden. Er prüft den exakten erzeugten absoluten Pfad, verschachtelte Downloadordner und Sonderzeichen. Frontend-Produktionsbuild, optimierter Rust-/Tauri-Build, Installer, portables Archiv und signierte Update-Metadaten sind bestanden. Ein echter WAI-Illustrious-Lauf auf dem ROG Ally bleibt auf dem Zielgerät zu prüfen.
