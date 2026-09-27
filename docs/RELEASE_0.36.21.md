# Local Studio 0.36.21

## Menüleiste bleibt bedienbar

- Native Browserflächen für Hugging Face, Civitai, Danbooru und Rule34 werden beim Scrollen jetzt am tatsächlichen App-Inhaltsbereich unterhalb der 36-Pixel-Titelleiste abgeschnitten.
- Verlässt eine eingebettete Website den sichtbaren Inhaltsbereich, wird ihr Child-WebView ausgeblendet. Beim Zurückscrollen erscheint er wieder an der korrekten Position.
- Das Rust-Backend lehnt zusätzlich jede Browserposition ab, die in die obere Menü- und Titelleiste hineinragt. Damit bleibt der Schutz auch bei einem späteren Frontendfehler wirksam.
- Ein hängen gebliebener nativer Popup-Aufruf blockiert weitere Menüversuche nicht mehr dauerhaft. Nach zehn Sekunden kann ein neuer Öffnungsversuch gestartet werden.

## Grenzen

Die aktuell bereits laufende ältere App kann ihren nativen Fensterzustand nicht während des Betriebs austauschen. Nach der Installation von 0.36.21 muss Local Studio einmal neu gestartet werden. Die unveränderten Inferenz-, Projekt-, Galerie- und Downloadpfade wurden für diesen gezielten UI-Hotfix nicht vollständig wiederholt.
