# Local Studio 0.36.5

## Verbleibende RAM-Sperre für ComfyUI entfernt

- Die in 0.36.4 verkleinerte RAM-Vorabreserve konnte auf dem ROG Ally weiterhin anschlagen, weil Windows bei gemeinsamem CPU-/GPU-Speicher zeitweise weniger als 1 GiB als unmittelbar frei meldet.
- Von Local Studio verwaltete ComfyUI-Aufträge besitzen deshalb keine pauschale Grenze für den gerade freien physischen RAM mehr. ComfyUI und ROCm entscheiden selbst über Laden, Auslagern und den tatsächlich benötigten Speicher.
- Der gemeinsame Ressourcenplaner behält die Auftragsreihenfolge und verhindert weiterhin parallele GPU-Aufträge.
- Andere lokale Laufzeiten behalten ihre bisherigen RAM-Grenzen. Der native `stable-diffusion.cpp`-Adapter muss seinen Checkpoint selbst laden und wird daher weiterhin vorab geschützt.
- Reicht der gemeinsame Speicher für eine konkrete Generierung tatsächlich nicht aus, zeigt Local Studio die reale ComfyUI-Laufzeitmeldung statt einer vorgelagerten pauschalen Sperre.

## Prüfumfang

Sechs gezielte Bild-Engine-Tests und zwei Tests des Ressourcenplaners sind bestanden. Sie bestätigen die unveränderte native RAM-Reserve, den RAM-unabhängigen ComfyUI-Zugang sowie Reihenfolge und GPU-Exklusivität. Ein echter WAI-Illustrious-Lauf auf dem ROG Ally bleibt auf dem Zielgerät zu prüfen.
