# Local Studio 0.36.24

Diese Version behebt zwei Fehler der Titelleiste:

- Native Menü-Popups werden nicht mehr durch parallele Fensteraktualisierungen entfernt. Datei-, Bearbeiten-, Ansicht- und Hilfe-Menüs bleiben so auch nach wiederholtem Öffnen bedienbar.
- Das Download- und Benachrichtigungsfenster folgt jetzt der eingestellten Oberflächenskalierung. Die Titelleiste bleibt während eines geöffneten Fensters in korrekter Größe und Position.

Geprüft: TypeScript-/Vite-Produktionsbuild, gezielte Downloadübersichtstests, Rust-Build sowie native Windows-Menü- und Skalierungsprüfung. Kein neuer Inferenz- oder Modellgenerierungstest; diese Pfade sind unverändert.
