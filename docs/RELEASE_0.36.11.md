# Local Studio 0.36.11

## Stabilerer ComfyUI-Speichermodus für AMD-APUs

- Der Bericht aus GitHub-Issue #2 bestätigt, dass Checkpoint, Workflow und AMD-ROCm-Laufzeit korrekt erkannt werden.
- Auf dem ROG Ally meldete ComfyUI 8,6 GB gemeinsamen Grafikspeicher bei nur 12 GB Gesamtspeicher, aktivierte `NORMAL_VRAM`, zwei asynchrone Offload-Streams und reservierte 4,8 GB gepinnten Hostspeicher. Der native Prozess stürzte anschließend beim Laden von WAI Illustrious und der VAE ab.
- Von Local Studio verwaltete AMD-ComfyUI-Installationen starten deshalb jetzt zusätzlich mit `--disable-pinned-memory` und `--disable-async-offload`.
- NVIDIA-Installationen bleiben unverändert.
- Künftige Bugreports bewahren gezielt den Logbereich um `Windows fatal`, `access violation`, Speichermeldungen oder einen Traceback sowie Start und Stackende.

## Prüfumfang

Drei gezielte Bugreport-Tests, sechs ComfyUI-Tests, Frontend-Produktionsbuild, optimierter Rust-/Tauri-Build, Installer, portables Archiv und signierte Update-Metadaten sind bestanden. Der echte WAI-Illustrious-Lauf auf dem ROG Ally bleibt als Zielgerätetest offen.
