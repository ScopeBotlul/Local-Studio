# Local Studio 0.30.0

Dieses Release ordnet die Hauptnavigation neu und gibt den kreativen Werkzeugen mehr Platz.

## Neu

- Schlanke, zentrierte Hauptnavigation mit Chat, Create, Galerie und Modelle.
- Das App-Logo links öffnet die Startseite; Hugging Face, Downloads, Aufträge und Einstellungen bleiben rechts direkt erreichbar.
- Create öffnet eine eigene Werkzeugleiste für Bild erstellen, Bild bearbeiten, Video bearbeiten und GIF erstellen.
- Die frühere Breadcrumb-Leiste und der doppelte Studio-Umschalter im Seiteninhalt wurden entfernt.
- Bei schmaleren Fenstern werden untergeordnete Beschriftungen automatisch reduziert. Die Create-Werkzeuge bleiben horizontal erreichbar.
- Pfeiltasten sowie Pos1 und Ende wechseln weiterhin barrierearm zwischen den Create-Werkzeugen.

## Enthaltene Funktionsbereiche

Die vorhandenen lokalen Bild-, Editor-, Videoschnitt- und GIF-Pfade aus 0.29.0 bleiben enthalten. Das GIF-Studio erstellt lokale Animationen aus bis zu 200 Einzelbildern. Die ComfyUI-, LoRA-, Civitai-, Danbooru-, Galerie-, Projekt-, Assistenten- und Updatefunktionen bleiben unverändert verfügbar.

## Prüfung und Grenzen

Frontend-Build und Frontendtests wurden für die Navigationsänderung ausgeführt. Die paketierte Windows-Anwendung wird in einem isolierten portablen Profil geprüft. Unveränderte KI-, Netzwerk- und Installer-Lebenszyklustests werden nicht allein wegen des neuen Builds vollständig wiederholt. Eine fehlende Prüfung wird in `TEST_MATRIX.md` offengelegt.

## Installation

- Installer: `Local-Studio-0.30.0-hub-setup.exe`
- Portable: `Local-Studio-0.30.0-hub-portable.zip`
- Beim portablen Update die App schließen, Programmdateien und alle vier Runtime-Ordner ersetzen. **Local-Studio-Data vollständig behalten.**
- Die Update-Metadaten werden mit dem bestehenden Local-Studio-Schlüssel signiert. Die Windows-Dateien besitzen weiterhin keine Authenticode-Signatur; SmartScreen kann deshalb warnen.
