# Local Studio 0.36.23

## Kein stiller Wechsel zur RAM-intensiven Bildengine

- Die lokalen Auftragsdaten zeigten vier erfolgreiche ComfyUI-Bilder, danach zwei Abbrüche in 1–2 ms mit `resource_memory`. Die fehlgeschlagenen Aufträge waren automatisch auf `stable-diffusion.cpp · Vulkan` geroutet worden, obwohl dasselbe Modell zuvor über ComfyUI lief.
- Bei NVIDIA bleibt der automatische Modus jetzt bei ComfyUI, wenn das gewählte Modell als ComfyUI-Checkpoint registriert ist. Eine kurzzeitig nicht erreichbare API führt nun zur ComfyUI-Verfügbarkeitsmeldung statt zum stillen Start des speicherintensiveren Vulkan-Workers.
- AMD-/Intel-Systeme behalten die bestehende automatische Vulkan-Auswahl. Vulkan lässt sich auf NVIDIA weiterhin ausdrücklich im Studio auswählen.
- Die RAM-Meldung nennt jetzt den betroffenen Bildengine-Speicherbedarf und schlägt vor, speicherintensive Programme zu schließen oder ComfyUI zu wählen.

## Gezielte Prüfung und Grenzen

- Routing-Regressionstest: 1/1 bestanden.
- Bildstudio-Fehlermeldungstests: 2/2 bestanden.
- Optimierter Windows-Build und Inno-Installer: erfolgreich erstellt.
- Das Portable ZIP enthält 80 erlaubte Dateien, davon 77 geprüfte Dateien in den vier Runtime-Manifests.
- Ein echter ComfyUI-Generierungslauf nach dem Routing-Fix steht noch aus. Der Regressionstest belegt gezielt, dass Auto bei NVIDIA/ComfyUI-Checkpoint nicht auf Vulkan zurückfällt, wenn die ComfyUI-Prüfung fehlschlägt.
- Veröffentlicht als Latest: https://github.com/ScopeBotlul/Local-Studio/releases/tag/v0.36.23. Alle fünf Release-Dateien wurden öffentlich zurückgeladen und gegen lokale Größen und SHA-256-Werte geprüft.
