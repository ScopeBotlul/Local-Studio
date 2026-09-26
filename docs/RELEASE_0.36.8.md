# Local Studio 0.36.8

## Verwaltete und extern laufende ComfyUI-Instanzen klar getrennt

- Die Modellvorprüfung stützte sich bisher darauf, ob der ComfyUI-Prozess im aktuellen Local-Studio-Lauf gestartet worden war. Nach App-Neustarts oder bei einer anderen bereits laufenden Instanz konnte ein Local-Studio-Checkpoint deshalb fälschlich auf den nativen NVIDIA-Adapter zurückfallen.
- Checkpoints im Local-Studio- oder konfigurierten ComfyUI-Modellordner werden jetzt direkt gegen die tatsächliche `CheckpointLoaderSimple`-Liste der lokalen ComfyUI-API geprüft.
- Ist der Checkpoint dort vorhanden, kann auch eine bereits laufende kompatibel konfigurierte Instanz verwendet werden. Ist er nicht vorhanden, erscheint die konkrete ComfyUI-Checkpointmeldung statt der irreführenden NVIDIA-Meldung.
- Belegt eine außerhalb von Local Studio gestartete ComfyUI-Instanz Port 8188, zeigt die Einstellung jetzt „Läuft extern · Port 8188 belegt“ mit einer klaren Anleitung. Local Studio beendet fremde Prozesse nicht selbst.
- Sobald die externe Instanz beendet wurde, erscheint „ComfyUI starten“. Der Start über diesen Button bindet den Local-Studio-Modellordner ein und wird als „von Local Studio verwaltet“ angezeigt.

## Prüfumfang

Sechs gezielte ComfyUI-Tests und der TypeScript-/Vite-Produktionsbuild sind bestanden. Der Pfadtest bestätigt, dass nur Checkpoints innerhalb der konfigurierten ComfyUI- und Local-Studio-Modellwurzeln als Kandidaten gelten. Ein echter Start und WAI-Illustrious-Lauf auf dem ROG Ally bleiben auf dem Zielgerät zu prüfen.
