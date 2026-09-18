# Local Studio 0.25.0

Großes Entwicklungspaket für Bildgenerierung, Modelle und Windows-Bedienung.

- SDXL-Bild-zu-Bild, maskiertes Inpainting mit Erhalt schwarzer Bereiche und Bildstapel mit wählbarer Seed-Folge.
- Unveränderliche Referenz-/Maskenkopien je Auftrag; Projektformat 4 transportiert beide Dateien. Galerie-PNGs direkt als Referenz übernehmen; Ebenenmasken als PNG exportieren.
- Optionale VAE-Ausführung auf der CPU; tatsächliche Sampling-Schrittzahl statt irreführender Fortschrittsannahmen.
- Gemeinsame Ressourcenwarteschlange für Bild, Assistent, Transkription, Videoexport und Medienvorbereitung; optionale begrenzte Parallelverarbeitung.
- Echte optionale CPU/RAM/GPU-Liveanzeige, Windows-Akzentfarbe und Weiterlaufen im Infobereich.
- Geprüftes Verschieben erfasster Modelldateien mit Kopierprüfung, Abbruch und Kollisionsschutz. Geladene Modelle müssen zuerst entladen werden.
- Abschaltbare Modellupdateprüfung für bekannte HF-Downloads; mehrere Updates gemeinsam prüfen und bewusst herunterladen. Alte Versionen bleiben erhalten.
- Lokale echte Leistungsmessungen und bearbeitbare gelernte Präferenzen; zusätzliche begrenzte Lese-Werkzeuge des Assistenten.
- Korrigierte Maskenvorschau ohne unnötiges Hochskalieren sowie robuster lokaler Hugging-Face-Anmelderückruf bei verzögerten Browseranfragen.

Der gesamte Masterprompt ist noch nicht vollständig umgesetzt. Insbesondere KI-Videoerzeugung, Musikmodelle, Workflows, Vision/Inhaltsindex, 18+-Sperre und weitere fortgeschrittene Editorenfunktionen bleiben offen. Modellverschiebung erfasst registrierte Dateien; bestehende Studio-/Projektmodellpfade müssen danach neu ausgewählt werden. Kein stilles Installieren von Modellabhängigkeiten und keine simulierten KI-Ergebnisse.

Installer: Update unter Hilfe prüfen, herunterladen und bewusst übernehmen. Portable: Programmdateien und alle vier Runtime-Ordner ersetzen, **Local-Studio-Data behalten**. Modellgewichte sind nicht enthalten. Projektformat 4 benötigt 0.25.0 oder neuer.

Bedienung: docs/IMAGE_REFERENCES.md. Implementierung: PROJECT_STATUS.md. Tatsächliche Prüfungen und Grenzen: TEST_MATRIX.md.
