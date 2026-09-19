# Projekte ab 0.21.0

Quellstand nach 0.27.0: Die Startseite bietet eine Projektübersicht mit dem aktuellen Arbeitsprojekt und bis zu zwölf zuletzt geöffneten Projektdateien. Neues Projekt fragt den Namen ab, Durchsuchen öffnet den vorhandenen Dateidialog für .localstudio-Dateien, Ausgewähltes Projekt öffnen öffnet die markierte Zeile beziehungsweise setzt das aktive Projekt fort. Nicht verfügbare Dateien sind gekennzeichnet. Neue Arbeitsprojekte werden weiterhin über Datei → Speichern als Projektdatei abgelegt. Diese Übersicht durchsucht keine Laufwerke automatisch und ist noch nicht im veröffentlichten Build 0.27.0 enthalten.

Die Projektleiste steht über dem Arbeitsbereich. Ohne Projekt bleiben Studio und Galerie verwendbar. Es kann genau ein Projekt aktiv sein.

1. Projektleiste aufklappen, Namen eingeben und **Projekt anlegen** wählen. Die aktuellen Bild-Studio-Eingaben werden übernommen.
2. Mit **Medien hinzufügen** lokale Bilder, Videos oder Audiodateien auswählen. Die App kopiert die Dateien in die lokale Projektarbeitskopie. Alternativ Dateien auf die Projektleiste ziehen, im Studio **Ins Projekt übernehmen** oder in der Galerie **Auswahl ins Projekt übernehmen** verwenden. Ohne aktives Projekt wird dabei ein unbenanntes Projekt angelegt. Eine ausstehende Sitzungswiederherstellung muss zuerst abgeschlossen werden. Originale bleiben unverändert.
3. **Projekt speichern** oder **Strg+S** schreibt eine `.localstudio`-Datei. Die Tastenkombination lässt sich in den Einstellungen ändern. **Speichern unter** benötigt einen freien Dateinamen; beliebige andere Projekte werden nicht überschrieben.
4. **Projekt öffnen** stellt die Eingaben wieder her und zeigt die eingebetteten Medien. Es startet keine Generierung und keinen Download.

Der Container enthält die aktuellen Bildparameter, Prompt und negativen Prompt sowie ausdrücklich hinzugefügte Medien. PNG/JPEG/Video werden ohne erneute Kompression gespeichert; JSON, WAV und BMP können verlustfrei komprimiert werden. Grenzen: 100 Medien, insgesamt 2 GiB Medien und 1 MiB Manifest. Die Vorschau verwendet dieselben begrenzten Bild-/Range-Lesezugriffe und verfügbaren WebView-Codecs wie die Galerie. Ein nicht unterstützter oder beschädigter Medieninhalt kann gespeichert sein, ohne abspielbar zu sein; er wird nicht ausgeführt.

Modellgewichte bleiben außerhalb des Containers. Für das derzeit unterstützte lokale Safetensors-Modell speichert das Projekt Dateiname und SHA-256. Absolute Modellpfade bleiben ausschließlich in der lokalen Arbeitskopie. Nach dem Öffnen eines Containers ist das Modell zunächst nicht zugeordnet. **Originalmodell zuordnen** akzeptiert nur identische Dateiinhalte; alternativ kann im Studio bewusst ein anderes Modell gewählt werden. Die Bereitschaftsprüfung im Studio bleibt erforderlich. Eine Prüfsumme bestätigt Dateigleichheit, keine vertrauenswürdige Herkunft. Hugging-Face-Repository/Revision werden hier noch nicht übernommen.

Den Projektnamen im aufgeklappten Bereich mit **Namen ändern** bearbeiten. Der Dateiname auf der Festplatte bleibt bis zu einer bewussten Speicherung unter anderem Namen bestehen.

**Aus Projekt entfernen** entfernt einen Eintrag aus dem Projekt. Der nächste Speichervorgang enthält dieses Medium nicht mehr. Originale und Galerie bleiben erhalten. **Medien in Galerie kopieren** exportiert Kopien unter ihren ursprünglichen Namen in einen neuen Galerieordner und meldet erfolgreiche Dateien sowie einzelne Fehler. Favoriten, Tags und Generierungsherkunft werden dabei noch nicht übertragen.

Ungespeicherte Studio-Eingaben eines aktiven Projekts werden zusätzlich lokal gesichert. Beim nächsten Start kann dieser Projektarbeitsstand ausdrücklich fortgesetzt werden; das gilt auch nach regulärem Beenden. Die `.localstudio`-Datei ändert sich nur durch Speichern. Der gemeinsame Beenden-Dialog bietet eine zusätzliche Checkbox für die Projektdatei; das Speichern fertiger Bilder in die Galerie bleibt eine eigene Auswahl im selben Dialog. Abbrechen lässt die App geöffnet.

Die Arbeitskopien liegen unter dem eingestellten Recovery-Pfad in `project-sessions`, die kleine Zustandsdatenbank im Konfigurationsordner. Der Projekte-Pfad ist das vorgeschlagene Speicherziel. Ein Wechsel dieser Einstellungen verschiebt keine bestehenden Daten. Import und Save prüfen Grenzen, Formatversion, feste Archivnamen und SHA-256. Überschreiben des eigenen Projekts prüft zusätzlich, ob seine Datei seit dem Öffnen/Speichern verändert wurde. Ein lokales SQLite-Journal sichert die Wiederherstellung zwischen Dateiwechsel und Zustandsübernahme. Ein fremder Zielkonflikt bleibt erhalten und blockiert eine unsichere Wiederherstellung.

**Entfernte Medien zurückholen** stellt aufbewahrte Medien wieder in die aktive Liste. Die Grenze ist unter Einstellungen anpassbar (Standard 100, maximal 1.000 entfernte Einträge); aktive Medien bleiben auf 100 / 2 GiB begrenzt. Entfernte Kopien werden nicht in die `.localstudio`-Datei geschrieben. Diese lokale Wiederherstellungsliste ist kein allgemeines Undo/Redo für Editoren.

**Wiederherstellungsstände** zeigt bis zu 20 lokale Projektstände über alle Projekte. Medien-/Namenswechsel, Schließen und Speichern sichern Vorgänger; Änderungen an Studio-Eingaben werden für die Historie zeitlich zusammengefasst. **Wiederherstellungspunkt erstellen** sichert den aktuellen Stand ausdrücklich. Ein älterer Stand lässt sich auch nach einem Prozessabbruch auswählen; geänderte oder fehlende benötigte Medien blockieren die Wiederherstellung. Die Projektdatei auf der Festplatte ändert sich erst beim nächsten Speichern. Die letzten drei Einträge bleiben vor Bereinigung geschützt, ebenso das aktive Projekt. Details: [Speicherbereinigung](SETTINGS.md).

Direkte Medienübernahme kopiert geprüfte Originalbytes. Sie ersetzt die aktuellen Studio-Eingaben nicht automatisch durch die Parameter des ausgewählten Bildauftrags. Favoriten/Tags und zusätzliche Herkunftsbeziehungen werden noch nicht als Projektmetadaten übertragen.

Noch offen: automatisches Wiederöffnen ohne Dateiauswahl, allgemeines Undo/Redo, weitere Modellreferenzen und vollständige Medienversionsbeziehungen. Ebenen, Masken, RAW-Entwicklung, Timeline, Workflows und weitere Studio-Medientypen folgen mit ihren echten Editoren/Adaptern. Diese Version behauptet keine vollständige M3-Abnahme.

## Zuletzt geöffnet und Windows-Dateiöffnen

Die aufgeklappte Projektleiste enthält **Zuletzt geöffnete Projekte** mit bis zu zwölf erfolgreich geöffneten oder gespeicherten Dateien. Nicht mehr erreichbare Dateien sind gekennzeichnet. **Aus Liste entfernen** löscht nur diesen Listeneintrag; Projektdateien und Medien bleiben erhalten.

Die installierte Ausgabe registriert `.localstudio` für Windows. Doppelklick oder **Öffnen mit → Local Studio** übergibt die Datei an die App. Ist Local Studio bereits geöffnet, wird das Projekt an diese Instanz weitergereicht. Bei ungespeicherten Änderungen bleibt die bekannte Bestätigung erforderlich; Abbrechen erhält das aktuelle Projekt. Archiv- und Medienvalidierung sind dieselben wie beim Öffnen über die Projektleiste. Ungültige Dateien ersetzen den bisherigen Arbeitsstand nicht.

Die portable Ausgabe verändert keine Dateizuordnungen. Sie akzeptiert eine Projektdatei als Argument bzw. über **Öffnen mit**. Portable Installationsordner verwenden getrennte Instanzkennungen, damit mehrere getrennte Datenprofile unabhängig bleiben. Ohne Dateiparameter gelten die bisherigen Start-/Recovery-Regeln. Es wird pro Öffnungsanforderung genau eine `.localstudio`-Datei akzeptiert; andere Argumente werden nicht ausgeführt.

## Bearbeitbare Bilder

Im Bildeditor **Bearbeitung ins Projekt übernehmen**, danach Editor schließen und Projektdatei speichern. Originalbytes und aktive Schritte reisen getrennt im Container Version 2 mit; erneutes Öffnen benötigt keine ursprüngliche Galeriedatei. In der Projektleiste Bild auswählen → **Projektbild bearbeiten**. Änderungen erneut übernehmen, dann Projekt speichern. Die Übersicht und **Medien in Galerie kopieren** verwenden weiterhin Originale; eine korrigierte Ausgabe über den Editor als Variante exportieren. Lokale Projekt-Recovery enthält übernommene Rezepte. Details und Grenzen: [Bildeditor](IMAGE_EDITOR.md). Ältere App-Versionen können Container Version 2 nicht öffnen; Projekte ohne Bearbeitungsrezepte werden weiterhin als Version 1 gespeichert.
