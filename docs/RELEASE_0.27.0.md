# Local Studio 0.27.0

## Neu

- Gemeinsame Titel- und Menüleiste: Logo links, Projektname und Local Studio mittig, Fenstersteuerung rechts. Menüs und Fensterknöpfe bleiben auch bei geöffneten Dialogen nutzbar.
- Projektverwaltung über **Datei**; Projektmedien und Details über **Ansicht**. Der bisherige große Projektblock entfällt im Studio.
- Überarbeitetes Bildstudio mit Canvas, seitlichen Einstellungen, Ergebnisverlauf und Zoom-/Verschiebefunktionen.
- Eigene SDXL-Auflösungen: 256–2048 Pixel pro Seite in 64er-Schritten, insgesamt höchstens 2.097.152 Pixel.
- Automatische Modellvorprüfung und lesbarere Windows-Pfade.
- Modellbibliothek nach Einsatzzweck, mit getrennten Erweiterungen, technischen Komponenten und nicht eindeutig erkannten Dateien. Encoder, VAEs und LoRAs erscheinen nicht als direkt ausführbare Bildmodelle.
- Automatische lokale 18+-Zuordnung anhand expliziter Modellmetadaten und unterstützter Begleitdateien. Fehlende Angaben bleiben unbekannt; die Funktion ist keine Inhaltsanalyse oder SFW-Garantie.

## Prüfstand und Grenzen

37 Frontend-Tests und 167 Rust-Tests bestanden; ein bereits bestehender Rust-Test ist weiterhin ignoriert. Native Menü-, Editor-/Galerie-, KI-, Windows- und Installerprüfungen bestanden. Installer und portable ZIP enthalten dieselbe geprüfte Programmdatei; Paketprüfsummen und Runtime-Dateien wurden verglichen.

**Offen:** Die erneute tatsächliche SDXL-Generierung mit eigener Auflösung und der darauf aufbauende vollständige native 18+-Sperrtest konnten auf dem Testrechner wegen zu wenig freiem Arbeitsspeicher nicht abgeschlossen werden (`resource_memory`). Der Worker wurde vor dem Laden abgewiesen. Eine vollständige Abnahme dieser beiden Strecken wird für diesen Build nicht behauptet.

Bildgenerierung unterstützt weiterhin vollständige SDXL-Checkpoints. Lose Komponenten werden nicht automatisch zu einer Pipeline zusammengesetzt. Weitere Modellfamilien und der gesamte geplante Funktionsumfang sind noch nicht vollständig unterstützt.

Die Windows-Dateien besitzen noch keine Authenticode-Signatur; SmartScreen kann deshalb warnen. Das Update-Manifest wird unabhängig davon mit dem bestehenden Local-Studio-Updateschlüssel signiert.

## Installation und Aktualisierung

- **Installer:** `Local-Studio-0.27.0-hub-setup.exe`
- **Portable:** `Local-Studio-0.27.0-hub-portable.zip`
- Beim portablen Update Local Studio vollständig schließen, Programmdateien und alle vier Runtime-Ordner ersetzen und **Local-Studio-Data vollständig behalten**. Dort liegen Einstellungen, Projekte, Galerie und weitere lokale Daten.
- Modellgewichte und persönliche Daten sind nicht in den Downloads enthalten.
