# Local Studio 0.36.3

## Heruntergeladene SDXL-Modelle mit AMD-ComfyUI verwenden

- Die von Local Studio gestartete ComfyUI-Engine erhält den lokalen Modellordner jetzt über ComfyUIs offiziellen zusätzlichen Modellpfad.
- Heruntergeladene SDXL-Checkpoints wie WAI Illustrious werden dadurch direkt vom installierten NVIDIA-, AMD- oder Intel-ComfyUI-Paket geladen.
- Die mehrere Gigabyte großen Gewichte werden weder kopiert noch verschoben. ComfyUI liest die bereits geprüfte lokale Datei an ihrem bestehenden Speicherort.
- Verschachtelte Downloadordner werden unterstützt; ComfyUI erhält normale Windows-Laufwerkspfade ohne den internen `\\?\`-Präfix.
- Eine extern gestartete ComfyUI-Instanz wird nicht stillschweigend umkonfiguriert. Die Integration gilt für die von Local Studio verwaltete Engine und wird beim Programmstart aktiv.
- Die bisher missverständliche NVIDIA-Meldung erklärt jetzt, dass nur der alte native Ersatzadapter NVIDIA/Vulkan benötigt.

## Prüfumfang

Fünf gezielte ComfyUI-Tests und der TypeScript-/Vite-Produktionsbuild sind bestanden. Der neue Test prüft die sicher erzeugte YAML-Konfiguration, Apostroph-Escaping, verschachtelte Checkpointnamen und die Abgrenzung fremder Pfade. Ein echter WAI-Illustrious-Lauf auf dem ROG Ally bleibt auf dem Zielgerät zu prüfen.
