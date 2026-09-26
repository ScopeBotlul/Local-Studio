# Local Studio 0.29.0

Dieses Release verbindet das Bildstudio erstmals mit einer lokal betriebenen portablen ComfyUI-Installation und erweitert den täglichen Modell- und Medienworkflow.

## Neu

- Portable ComfyUI erkennen, auswählen, lokal starten und beenden. Beim ersten Start ohne Installation führt ein Einrichtungsdialog zum offiziellen Download; der vorhandene Updateordner lässt sich öffnen.
- SDXL-Bildaufträge können über die lokale ComfyUI-API ausgeführt werden. Bis zu acht LoRAs lassen sich im Studio auswählen, gewichten und vor Ausführung erneut prüfen.
- Rechte Studiogalerie mit gemeinsamem aktuellem Ordner als Speicherziel; Modellnamen ohne `.safetensors`, zufälliger Seed als Standard und Strg+F über Prompt und Negativ-Prompt.
- Lokale Modelle können nach erneuter Warnung sicher vom Datenträger gelöscht werden. Die Bibliothek zeigt außerdem aktuell verwendete Bild-, Sprach- und Chatmodelle.
- Civitai als eingeschränkter interner Browser: Bildvorlage, Prompt, Negativ-Prompt und gemeldete Ressourcen übernehmen. Neue jugendfreie LoRA-Suche mit optionaler Basisfamilie.
- Danbooru-Postseiten im internen Browser öffnen und deren öffentliche Tags formatiert übernehmen. Bekannte Zensur-Tags können über eine Einstellung entfernt werden; manuell eingefügte Listen von weiteren Seiten bleiben möglich.
- Neues GIF-Studio für echte lokale Animationen aus bis zu 200 Bildern.
- Lokaler Programmiermodus im Assistenten ohne Studio-Werkzeuge und ohne automatische Ausführung von Modellcode.
- Navigation auf reine Symbole reduzieren; laufende Downloads direkt über das Downloadsymbol in der Menüleiste prüfen.
- Bereits vollständig geschriebene gültige Bilder bleiben erhalten, wenn der Bildworker erst danach einen Speicherfehler meldet.

## Sicherheit und Grenzen

ComfyUI wird ausschließlich über `127.0.0.1:8188` angesprochen. Local Studio installiert oder startet keine Custom Nodes, Python-Pakete oder modellbereitgestellten Programme. Der Updater wird nicht automatisch ausgeführt. Civitai- und Danbooru-Navigation ist auf feste HTTPS-Domains begrenzt; Downloads werden nicht ungefragt installiert.

Vollständige SDXL-Checkpoints und LoRAs sind der erste ComfyUI-Pfad. Lose UNet-, CLIP-, T5- und VAE-Komponenten benötigen weiterhin explizite familienbezogene Workflows; sie werden nicht automatisch zu einer vermeintlich kompatiblen Pipeline zusammengesetzt. Civitai-Downloads öffnen die offizielle Downloadadresse und werden anschließend über die lokale Modellsuche aufgenommen.

Frontend-Build, Compilerchecks und die automatisierten Frontend-/Rusttests für die geänderten Bereiche sind bestanden. Ein echter ComfyUI-Lauf mit beliebigen Nutzergewichten, Live-Civitai-/Danbooru-Antworten und die native Kind-WebView-Darstellung konnten nicht vollständig automatisiert abgenommen werden. Details stehen in `TEST_MATRIX.md`.

## Installation

- Installer: `Local-Studio-0.29.0-hub-setup.exe`
- Portable: `Local-Studio-0.29.0-hub-portable.zip`
- Beim portablen Update App schließen, Programmdateien und alle vier Runtime-Ordner ersetzen. **Local-Studio-Data vollständig behalten.**
- Die Update-Metadaten werden mit dem bestehenden Local-Studio-Schlüssel signiert. Die Windows-Dateien besitzen weiterhin keine Authenticode-Signatur; SmartScreen kann deshalb warnen.
