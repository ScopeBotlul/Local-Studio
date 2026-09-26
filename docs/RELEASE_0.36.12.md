# Local Studio 0.36.12

## SDXL-Textencoder auf AMD-APUs stabilisiert

- GitHub-Issue #3 bestätigt, dass die Schutzschalter aus 0.36.11 aktiv waren und ComfyUI keinen gepinnten Speicher oder asynchrones Offloading mehr verwendete.
- Der verbliebene native Absturz ist nun eindeutig lokalisiert: eine Windows-Zugriffsverletzung in `torch.nn.functional.embedding`, während DynamicVRAM den SDXL-Textencoder nach teilweisem Modell-Offload auf der AMD-GPU ausführt.
- Verwaltete AMD-Installationen auf Geräten mit höchstens 16 GB Gesamtspeicher starten jetzt zusätzlich mit `--disable-dynamic-vram` und `--lowvram`.
- Laut ComfyUI-Verhalten läuft der Textencoder in dieser Kombination auf der CPU. Der fehlerhafte ROCm-Embedding-Pfad wird dadurch umgangen.
- AMD-Systeme mit mehr als 16 GB und NVIDIA-Installationen behalten ihre bisherigen Profile.

## Prüfumfang

Sechs gezielte ComfyUI-Tests, Frontend-Produktionsbuild, optimierter Rust-/Tauri-Build, Installer, portables Archiv und signierte Update-Metadaten sind bestanden. Der echte WAI-Illustrious-Lauf auf dem ROG Ally bleibt als Zielgerätetest offen; der konservative Modus kann langsamer sein.
