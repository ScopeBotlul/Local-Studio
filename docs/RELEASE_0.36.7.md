# Local Studio 0.36.7

## ComfyUI-Workflow an die laufende Installation angepasst

- Local Studio hat verschachtelte Checkpointnamen intern mit `/` aufgebaut. ComfyUI führt dieselben Namen unter Windows in seiner erlaubten Node-Auswahlliste mit `\`; die exakte Wertprüfung konnte den Workflow deshalb bereits vor der Ausführung ablehnen.
- Vor dem Absenden liest Local Studio jetzt die tatsächlich von ComfyUI gemeldeten Werte für Checkpoint, LoRA, Sampler und Scheduler über die lokale Loopback-API.
- Pfadnamen werden zum Vergleich normalisiert, im Workflow aber exakt in dem Format verwendet, das die laufende ComfyUI-Installation zurückgibt.
- Ist der ausgewählte Checkpoint noch nicht bei ComfyUI registriert, erscheint eine eigene verständliche Meldung mit dem Hinweis, die verwaltete Engine neu zu starten und Installation beziehungsweise Modellpfad zu prüfen.
- Andere HTTP-400-Antworten werden begrenzt und ausschließlich lokal in den technischen Auftragsdetails sowie im ComfyUI-Protokoll gespeichert. Dadurch ist eine verbleibende Node-Ablehnung konkret untersuchbar.

## Prüfumfang

Sechs gezielte ComfyUI-Tests sind bestanden. Der neue Test bestätigt insbesondere die Zuordnung eines internen `nested/wai.safetensors` zu dem exakten von ComfyUI gemeldeten Windows-Namen `nested\wai.safetensors` sowie die Auswahl unterstützter Scheduler. Ein echter WAI-Illustrious-Lauf auf dem ROG Ally bleibt auf dem Zielgerät zu prüfen.
