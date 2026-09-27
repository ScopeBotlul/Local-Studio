# Local Studio 0.36.20

## Galerie-Speichern repariert

- **In Galerie speichern** funktioniert wieder, wenn der zuletzt im Studio geöffnete Galerie-Unterordner inzwischen gelöscht, verschoben oder nicht mehr sicher erreichbar ist.
- Local Studio erkennt ausschließlich die beiden Zielordnerfehler `gallery_missing` und `gallery_path`, setzt das Sitzungsziel auf den Galerie-Hauptordner zurück und wiederholt denselben Speichervorgang genau einmal.
- Schreib-, Bild- und Datenbankfehler werden weiterhin angezeigt. Sie lösen keinen automatischen zweiten Versuch aus und werden nicht als erfolgreicher Speichervorgang dargestellt.
- Die vier zuletzt erzeugten lokalen Bilder wurden als vollständige temporäre PNG-Ergebnisse erkannt; an diesen persönlichen Dateien wurde nichts verändert.

## Grenzen

Der Hotfix ändert nur die Zielordnerbehandlung des vorhandenen Galerie-Speicherwegs. Unveränderte Bildgenerierung, Projekte, Editoren, Modellverwaltung, Updates und Installation wurden nicht erneut vollständig geprüft.
