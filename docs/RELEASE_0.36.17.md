# Local Studio 0.36.17

## Taskleistenicon und Studio-Navigation

- Der Hauptarbeitsbereich heißt in der oberen Navigation wieder **Studio**. Darunter bleiben Bildgenerierung, Bildeditor, Video und GIF direkt erreichbar.
- Die Akzentfarbe wird jetzt ausdrücklich als großes Windows-Fenstericon (`ICON_BIG`) gesetzt. Windows verwendet genau dieses Icon für die laufende Taskleistenschaltfläche; zusätzlich werden die beiden kleinen Fenstericons aktualisiert.
- Der bisherige Tauri-Aufruf setzte unter Windows lediglich `ICON_SMALL`. Deshalb änderte sich zwar das interne Fenstericon ohne Fehler, die Taskleiste behielt aber das statische Programmsymbol.
- Die App erzeugt weiterhin ausschließlich lokal ein 64 × 64 Pixel großes PNG. Der Rust-Kern prüft Größe, Dekodierbarkeit und Datenmenge, bevor er die drei Windows-Icon-Slots aktualisiert.

## Prüfung und Grenzen

Der neue native IPC-Pfad prüft nach dem Setzen über `WM_GETICON`, dass Windows das neue große Taskleistenicon tatsächlich am Hauptfenster führt. Das installierte Datei-, Desktop- und Startmenüicon bleibt weiterhin statisch, weil diese Symbole aus der Programmbinärdatei beziehungsweise dem Windows-Iconcache stammen. Die Windows-Dateien sind weiterhin nicht Authenticode-signiert.
