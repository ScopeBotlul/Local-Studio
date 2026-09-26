# Local Studio 0.36.4

## RAM-Vorprüfung für ComfyUI auf dem ROG Ally korrigiert

- Local Studio behandelt die Dateigröße eines Checkpoints bei ComfyUI nicht mehr als zwingend freien System-RAM. Ein WAI-Illustrious-Modell mit rund 7 GB wird deshalb nicht mehr schon vor dem Start mit „Nicht genügend freier RAM“ abgewiesen.
- ComfyUI-Aufträge reservieren im gemeinsamen Local-Studio-Planer weiterhin 512 MiB Arbeitsspeicher. Zusammen mit dessen bestehender Sicherheitsreserve müssen ungefähr 1 GiB frei sein.
- Die serielle GPU-Sperre bleibt aktiv, sodass Local Studio weiterhin nur einen GPU-Auftrag gleichzeitig startet.
- ComfyUI und ROCm entscheiden über das tatsächliche Laden, Auslagern und den gemeinsamen CPU-/GPU-Speicher. Falls der Speicher für eine konkrete Auflösung wirklich nicht reicht, erscheint nun die echte ComfyUI-Fehlermeldung.
- Der native `stable-diffusion.cpp`-Adapter behält seine strengere Prüfung, weil er den Checkpoint selbst laden muss.

## Prüfumfang

Sechs gezielte Bild-Engine-Tests und zwei Tests des Ressourcenplaners sind bestanden. Sie decken die neue ComfyUI-Reserve, die unveränderte native Reserve sowie Freigabe und Abweisung von Ressourcen ab. Ein echter WAI-Illustrious-Generierungslauf auf dem ROG Ally bleibt auf dem Zielgerät zu prüfen.
