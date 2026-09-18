# Ebenen und Videoschnitt ab 0.22.0

Im **Studio** zwischen **Bildgenerierung**, **Bildeditor** und **Videoschnitt** wechseln. Beide neuen Editoren verwenden denselben lokalen Projektarbeitsstand. **Datei → Projekt speichern** beziehungsweise Strg+S schreibt Medien und aktive Bearbeitungen zusammen in eine `.localstudio`-Datei. Ohne geöffnetes Projekt entsteht beim ersten Import eine unbenannte Arbeitskopie. Originaldateien werden kopiert und bleiben unverändert.

## Bilder

- Neue Arbeitsfläche: Breite/Höhe und transparenter oder farbiger Hintergrund. PNG/JPEG/BMP als Ebenen hinzufügen oder Dateien auf den Editor ziehen.
- Ebene auswählen; Strg+Klick erweitert die Auswahl. Umbenennen, duplizieren, entfernen, Reihenfolge, Sichtbarkeit und Sperre. Gruppen wählen ihre Mitglieder gemeinsam aus; sie sind keine verschachtelten isolierten Mischgruppen.
- Position, Größe, Seitenverhältnis und freie Drehung; Deckkraft und Normal/Multiply/Screen/Overlay. Ausrichten, Zentrieren, Einrasten und Hilfslinien. Verschieben mit Maus, Zoom und mittlere Maustaste zum Verschieben der Ansicht.
- Masken: Pinsel, Rechteck und Ellipse; Größe, Härte und Deckkraft. Ausblenden/Wiederherstellen, Umkehren, Zurücksetzen und separate Maskenvorschau. Masken wirken auf die aktive Ebene. Die Vorschau wird nach dem Loslassen tatsächlich neu berechnet.
- Relative RGB-Korrekturen je Ebene. Rückgängig/Wiederholen gilt gemeinsam für Bild- und Timeline-Änderungen im aktuellen Projekt; die aktive Komposition wird transportiert, die lokale Undo-Historie nicht.
- PNG mit Alpha oder JPEG mit weißem Hintergrund, eigene Exportabmessungen und JPEG-Qualität. Exportvorschau verwendet denselben Renderer. Export erstellt eine neue Galerie-Datei.

Grenzen: 8-Bit-RGB, maximal 32 Ebenen, 8192 Pixel je Kante und 16 Megapixel Ausgabe. Eingebettete ICC-Profile werden im neuen Ebenenpfad ausdrücklich zurückgewiesen; der bestehende Einzelbildeditor bleibt dafür verfügbar. RAW, HDR, vollständiges Farbmanagement, Text-/Vektorebenen und KI-Bearbeitung bleiben weitere Arbeitspakete.

## Video und Ton

- Lokale Video-, Bild- und Audiodateien hinzufügen oder hineinziehen. Importierte Clips werden zunächst hintereinander angeordnet. Für parallelen Ton den Audio-Start auf den gewünschten Zeitpunkt setzen.
- Bis zu acht Video-/Audiospuren, Sperre und Stummschaltung. Clips per Maus verschieben oder Start/Quelle In/Out eingeben; am Abspielkopf teilen, duplizieren, entfernen und auf kompatible Spuren setzen.
- Geschwindigkeit 0,25–4×, Ein-/Ausblenden und Überblenden mit dem vorherigen Clip. Ton einer Videoquelle auf eine Audiospur abtrennen; Originalclip bleibt erhalten und wird stumm.
- Lineare Keyframes für Position, Skalierung, Drehung, Deckkraft und Lautstärke. Helligkeit, Kontrast, Sättigung und Weichzeichnen.
- Manuelle Texte/Untertitel mit Zeit, Größe, Farbe und Position; UTF-8-SRT importieren/exportieren. Texte werden in die Videoausgabe eingebrannt. Eine vorhandene SRT-Datei wird nicht still überschrieben.
- **Vorschau rendern** berechnet die gesamte Timeline mit kleinerer Auflösung; danach Wiedergabe und Suchen im Video. Änderungen starten keinen automatischen Render und kennzeichnen eine alte Vorschau als veraltet. Der Abspielkopf ist die Schnitt-/Keyframeposition; eine Echtzeit-Schnittvorschau ist noch nicht enthalten.
- Export in die Galerie: MP4 (H.264/AAC) oder WebM (VP9/Opus), Auflösung, Bildrate und Bitrate. CPU-Encoding, 8-Bit-SDR. Export verwendet die Originalmedien. Ein echter Renderauftrag läuft gleichzeitig; Fortschritt/Abbruch erscheinen im Editor und unter Aufträge. Ein laufender Auftrag behält seinen eigenen unveränderlichen Timeline-Stand.

Grenzen: 64 Clips, 500 Untertitel, eine Stunde Timeline, maximal 3840×2160 (auch Hochformat bis 1080×1920), 24/25/30/50/60 fps. Der vorhandene Projektcontainer begrenzt die transportierten Medien insgesamt auf 2 GiB. Lange/komplexe Projekte können teuer sein; dies ist die erste klassische Timeline. Proxies, Audio-Wellenformen, automatische Transkription, Tracking, Stabilisierung, KI-Videoerzeugung und weitere M5–M10-Funktionen bleiben offen. Vorhandene HDR-Dateien werden nicht farbmetrisch gemanagt; für diese erste Ausgabe SDR-Material verwenden.

## Speicherung und Schutz

Projektformat 3 enthält Ebenen, Masken, Timeline, Keyframes und Texte sowie die bereits vorhandenen Inhalte. Ältere Programmversionen können diese neuen Projekte nicht öffnen. Alte Projektformate 1/2 bleiben lesbar. Medien, die Ebenen/Clips verwenden, können erst nach Entfernen dieser Verweise aus dem Projekt entfernt werden. Jede tatsächliche Verarbeitung prüft Größe und SHA-256 der Projektquelle.

Arbeitskopien werden lokal gesichert; nach einem Prozessabbruch muss die Wiederaufnahme bestätigt werden. Videoaufträge werden dabei als unterbrochen angezeigt und nicht automatisch erneut ausgeführt. Fertige Exporte liegen in der Galerie. Render-Arbeitsdateien und `render.log` liegen im konfigurierten temporären Verzeichnis unter `video-renders/<Auftrag-ID>`; automatische Aufbewahrungsbereinigung für diese neuen Renderordner ist noch offen.

IPC akzeptiert typisierte passive Daten, keine frei eingebbaren Befehle oder Filterprogramme. FFmpeg läuft als separater, versteckter Prozess mit Windows-Jobobjekt und wird beim Prozessende beendet. Eingangsformate und Protokolle sind begrenzt. Medien/Prompts werden dafür nicht hochgeladen.

## Entwicklungsumgebung

`node scripts/bootstrap-video-runtime.mjs` richtet ausdrücklich die in `src-tauri/video-runtime.json` gepinnte Video-Runtime ein. Download und einzelne Dateien werden anhand SHA-256 geprüft. Desktop-Build, Installer und portable Ausgabe übernehmen nur diese Dateien. `docs/video-runtime` enthält Lizenz-, Build- und Quellhinweise. Die Runtime wird nicht durch Modelle installiert. Kein FFmpeg aus dem PATH wird ausgeführt.

Prüfnachweise stehen getrennt in [TEST_MATRIX.md](../TEST_MATRIX.md).
