# Bildeditor ab 0.19.0

In der **Galerie** ein Bild auswählen und **Bild bearbeiten** öffnen. Dieser erste Editorpfad unterstützt statische PNG-, JPEG- und BMP-Dateien mit 8 Bit je Kanal. Animierte PNGs, GIF, WebP, RAW und höher aufgelöste Farbtiefen werden nicht still auf ein Einzelbild oder 8 Bit reduziert.

- Um 90° links/rechts drehen und horizontal/vertikal spiegeln.
- **Ausschnitt im Bild ziehen** aktiviert die Rechteckauswahl. Alternativ X/Y/Breite/Höhe in Bildpixeln eingeben; **Ausschnitt anwenden** übernimmt sie.
- Breite/Höhe ändern, standardmäßig mit festem Seitenverhältnis. **Größe anwenden** verwendet klassische Lanczos-Skalierung mit vormultipliziertem Alpha, keine KI-Vergrößerung.
- Rückgängig/Wiederholen über Schaltflächen oder standardmäßig Strg+Z/Strg+Y. Beide Kürzel sind in Einstellungen änderbar; in Eingabefeldern bleibt die Texteingabe zuständig. Das eingestellte Undo-Limit gilt, ältere notwendige Rezeptschritte bleiben intern erhalten.
- Mausrad zoomt, mittlere Maustaste verschiebt. Strg+Mausrad ändert die Oberfläche. Einpassen berücksichtigt Breite und Höhe; 100 % stellt Bildpixel in entsprechender Bildschirmgröße dar. Die Vorschau ist auf 1.600 Pixel je Kante begrenzt, keine pixelgenaue Vollauflösungslupe für größere Bilder.

## Export und Originalschutz

**Als Variante exportieren** schreibt eine neue PNG- oder JPEG-Datei in denselben Galerieordner. Das Original wird nur gelesen und während der Verarbeitung gegen Schreiben/Austausch gesperrt. Quelle und alle Zwischenmaße werden erneut geprüft. Eine vollständig geschriebene Datei wird unter einem eindeutigen Namen ohne Überschreiben veröffentlicht. Herkunft, Elternversion und angewendete Bearbeitungsschritte stehen lokal in SQLite. Die Galerie zeigt den Eintrag als **Bildbearbeitung**; Hauptversionswahl und bestehende Medien-/Projektübernahme bleiben verfügbar.

PNG erhält Alpha, JPEG hinterlegt transparente Pixel weiß; Qualität ist von 1 bis 100 einstellbar. Vorschau und Export verwenden dieselben Rust-Transformationen. EXIF-Orientierung wird einmal in die Pixel eingerechnet. EXIF/GPS werden beim Export entfernt. Eingebettete RGB-ICC-Profile werden bytegleich in PNG/JPEG erhalten; ohne Profil findet keine zusätzliche Profilzuordnung statt. Graustufen-/CMYK-Profile werden abgewiesen. Das ist noch keine vollständige farbmetrische ICC-/HDR-Abnahme oder frei wählbare Farbraumkonvertierung.

## Entwürfe und Grenzen

Angewendete Schritte und Undo/Redo-Verlauf werden im lokalen WebView-Profil gespeichert, an Galerie, Dateiidentität und genaue Quellversion gebunden. Bei erneutem Öffnen desselben unveränderten Bildes **Entwurf fortsetzen** wählen. Editor schließen bietet Behalten/Verwerfen/Weiterarbeiten. Der gemeinsame App-Beenden-Dialog weist auf noch nicht exportierte Bearbeitung hin und wartet auf eine laufende Verarbeitung. Ein Speicherausfall wird sichtbar gemeldet. Exportierte Medien und deren Herkunft sind unabhängig vom Entwurf.

Die Entwürfe werden noch nicht in `.localstudio` eingebettet und sind kein transportables Projektformat. Bei externem Umbenennen, Ersetzen, Galeriepfadwechsel oder Löschen des WebView-Profils ist automatische Entwurfszuordnung noch offen. Nach einem Prozessabbruch während des Exports kann eine unveröffentlichte `.tmp`-Datei zurückbleiben; sie wird nicht als fertiges Bild angezeigt. Keine automatische Löschung unbekannter Dateien.

Grenzen: Quelle 64 MiB, 32 Megapixel, höchstens 16.384 Pixel pro Kante, 1 MiB ICC, maximal 1.000 Rezeptschritte und insgesamt 512 Millionen verarbeitete Ausgabepixel pro Aufruf. Bei langen Bearbeitungen Zwischenstand exportieren und neu öffnen. Eine Verarbeitung gleichzeitig, gemeinsam mit dem begrenzten Miniaturdecoder; Rust-Hintergrundthread hält die Oberfläche frei. Keine KI-Inferenz in diesem Editorpfad.

M4 bleibt offen: Ebenen, Masken, Auswahlwerkzeuge, Text/Vektor, freie/perspektivische Transformation, Korrekturen, RAW, vollständiges ICC/HDR, KI-Bearbeitung, Batch, weitere Formate/Exportziele/Metadatenoptionen und Projekttransport der editierbaren Rezepte.
