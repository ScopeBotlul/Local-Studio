# Local Studio 0.36.1

## Downloadfortschritt direkt in der Titelleiste

- Unter dem kompakten Download-Symbol erscheint während eines laufenden Downloads eine schmale Fortschrittsleiste in der eingestellten Akzentfarbe.
- Mehrere Downloads werden anhand der tatsächlich übertragenen Bytes zu einem gemeinsamen Fortschritt zusammengefasst.
- Falls ein laufender Schritt noch keine Gesamtgröße kennt, zeigt die Leiste einen laufenden unbestimmten Status.
- Der Status wird auch bei geschlossenem Downloadfenster aktualisiert. Der zugängliche Name des Symbols nennt Anzahl und Fortschritt.

## Prüfumfang

Sechs gezielte Tests für Downloadzeit und zusammengefassten Fortschritt sowie der TypeScript-/Vite-Produktionsbuild sind bestanden. Die Anzeige wurde nicht mit einem echten mehrgigabytegroßen Modelldownload visuell abgenommen.
