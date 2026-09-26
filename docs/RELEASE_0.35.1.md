# Local Studio 0.35.1

## Fehlerbehebung

- Die integrierte ComfyUI-Installation akzeptiert jetzt die langen, verschachtelten Python- und PyTorch-Pfade des offiziellen portablen Archivs. Version 0.35.0 konnte dieses unveränderte offizielle Paket nach erfolgreichem Download und Hashvergleich fälschlich als „unsichere oder unerwartete Struktur“ ablehnen.
- Die Sicherheitsprüfung bleibt aktiv: absolute Pfade, Laufwerkspfade, `..`, leere Pfadbestandteile, ungültige Windows-Zeichen und reservierte Gerätenamen werden weiterhin vor dem Entpacken abgewiesen.
- Die Ausgabe der Windows-Archivfunktion darf sichere Dateinamen in der lokalen Windows-Zeichenkodierung enthalten. Sicherheitsrelevante Trennzeichen und Traversalmarker werden weiterhin als ASCII geprüft.

## Aktualisierung

Installierte Ausgaben können dieses signierte Local-Studio-Update im Update-Center herunterladen und installieren. Danach kann der ComfyUI-Download erneut gestartet werden. Ein fehlgeschlagener Versuch aus 0.35.0 hinterlässt kein teilweise eingebundenes ComfyUI-Verzeichnis.

## Prüfumfang

Der gezielte Rusttest für ComfyUI besteht mit dem realen Längenfall sowie den bestehenden Archiv- und Versionsgrenzen. Die Pfadlänge wurde zusätzlich gegen eine vorhandene offizielle portable ComfyUI-Installation geprüft. Ein erneuter vollständiger Download des mehrere Gigabyte großen Pakets wurde für diese Quellprüfung nicht ausgeführt.
