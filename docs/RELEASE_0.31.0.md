# Local Studio 0.31.0

Dieses Release integriert die Civitai-Modellsuche direkt in die Modellbibliothek und behebt fehlende Desktop-Berechtigungen für die in 0.29.0 eingeführten ComfyUI-, GIF- und Civitai-Funktionen.

## Neu

- Die Modellbibliothek wechselt zwischen Hugging Face, Civitai und lokal gespeicherten Modellen.
- Civitai-Suche mit Suchtext, Modelltyp, Basisfamilie, Sortierung, Zeitraum und optionalen 18+-Treffern.
- Detailansicht mit Versionen, Triggerwörtern, Dateigrößen, SHA-256 sowie Virus- und Pickle-Scanstatus.
- Direkter Download öffentlicher Checkpoints, LoRAs, VAEs, ControlNets, Upscaler und Textual-Inversion-Dateien über die vorhandene Downloadwarteschlange.
- Downloads landen bei konfiguriertem ComfyUI im passenden Modellordner, sonst im lokalen Modellordner von Local Studio.
- 18+-Treffer bleiben an die lokale Entsperrung gebunden und werden beim Download als geschützt registriert.
- Die fehlenden Tauri-Berechtigungen für ComfyUI, GIF-Studio, Civitai und Danbooru sind im lokalen Hauptfenster freigeschaltet. Fremde Webviews erhalten diese Rechte weiterhin nicht.

## Download-Sicherheit

Local Studio lädt nur die von Civitai als primär gemeldete Safetensors-Datei. Ein SHA-256-Hash und erfolgreiche Virus- sowie Pickle-Scans sind Pflicht. Modellmetadaten werden unmittelbar vor dem Erstellen des Downloadplans erneut von Civitai gelesen. Die fertige Datei wird gegen den erwarteten SHA-256-Hash geprüft, bevor sie veröffentlicht wird. Redirects sind auf Civitai und dessen beobachteten festen Auslieferungshost begrenzt.

Die Suche und öffentliche Downloads funktionieren ohne Civitai-Konto. Modelle, für die Civitai einen API-Schlüssel oder zusätzliche Zustimmung verlangt, werden in dieser Version nicht direkt geladen. Es werden keine Modellabhängigkeiten, Custom Nodes oder fremder Code installiert.

## Prüfung und Grenzen

Frontend, Civitai-Auswahlregeln, Rust-Metadatenverarbeitung und Downloadgrenzen wurden gezielt geprüft. Das öffentliche Civitai-Antwortformat und der erste Download-Redirect wurden live gelesen, ohne ein Modell herunterzuladen. Eine echte mehrgigabytegroße Modelldatei wurde nicht für die Releaseprüfung geladen. Echte ComfyUI-Inferenz und geschützte Civitai-Downloads bleiben außerhalb dieses Prüfstands.

## Installation

- Installer: `Local-Studio-0.31.0-hub-setup.exe`
- Portable: `Local-Studio-0.31.0-hub-portable.zip`
- Beim portablen Update die App schließen, Programmdateien und alle vier Runtime-Ordner ersetzen. **Local-Studio-Data vollständig behalten.**
- Die Update-Metadaten werden mit dem bestehenden Local-Studio-Schlüssel signiert. Die Windows-Dateien besitzen weiterhin keine Authenticode-Signatur; SmartScreen kann deshalb warnen.
