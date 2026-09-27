# Local Studio 0.36.25

Das Bildstudio erhält breitere, anpassbare Seitenbereiche:

- Modell, Prompt und Parameter links starten bei 410 px Breite.
- Die Studio-Galerie rechts startet bei 300 px Breite.
- Beide Bereiche lassen sich an den Trennleisten ziehen. Pfeiltasten, Pos1/Ende und Doppelklick bieten dieselben Funktionen per Tastatur.
- Die gewählten Breiten bleiben lokal gespeichert. Auf schmalen Fenstern bleibt die responsive einspaltige Ansicht erhalten.

Geprüft: TypeScript-/Vite-Produktionsbuild und isolierte native Windows-App mit gespeicherten Startwerten, Tastaturverstellung und Rücksetzen. Inferenz, Modell- und Medienpfade sind unverändert.
