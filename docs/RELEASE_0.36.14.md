# Local Studio 0.36.14

## Zusätzlicher ROG-Ally-Kompatibilitätsmodus

- GitHub-Issue #5 reproduziert die native Windows-Zugriffsverletzung beim ersten SDXL-Sampling auf dem ROG Ally mit ROCm 7.2 und `gfx1103`.
- Der Bericht bestätigt erneut, dass WAI Illustrious, VAE und Textencoder vollständig geladen werden. Der Absturz folgt erst im Sampler.
- Bei verwalteten AMD-Paketen auf Geräten mit höchstens 16 GB Gesamtspeicher startet ComfyUI nun zusätzlich mit `--disable-mmap`.
- Derselbe eng begrenzte Gerätepfad setzt `COMFY_KITCHEN_DISABLE_HIP=1`. Dadurch verwendet ComfyUI für diese Geräte keine nativen `comfy-kitchen`-HIP-Kernel, sondern die vorhandenen sicheren Fallbacks.
- NVIDIA-Systeme, externe ComfyUI-Instanzen und AMD-Systeme mit mehr als 16 GB behalten ihr bisheriges Profil.

## Warum diese Änderung

ComfyUI führt einen offenen Fix für Windows-ROCm-/UMA-Zugriffsverletzungen bei Safetensors über 4 GB. `comfy-kitchen` dokumentiert außerdem ausdrücklich den Laufzeitschalter zum Entfernen des HIP-Backends aus der Dispatch-Auswahl. Beide Pfade passen zum Bericht: ein 6,94-GB-Checkpoint auf einer APU mit gemeinsamem Speicher und ein unmittelbar nach dem Modellladen auftretender nativer Sampling-Absturz.

## Prüfumfang

Sechs gezielte ComfyUI-Tests, der Frontend-Produktionsbuild, der optimierte Rust-/Tauri-Build, Installer, portables Archiv und signierte Update-Metadaten sind bestanden. Die Tests decken AMD-Paketerkennung, das Low-Memory-Profil, Modellpfade, Workflowwerte, Archivschutz und Versionsvergleiche ab. Der echte WAI-Illustrious-Lauf auf dem ROG Ally bleibt als Zielgeräteprüfung offen; die Fallbacks können langsamer sein.
