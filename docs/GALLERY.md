# Galerie – Stand 0.19.0

Neu: **Bild bearbeiten** öffnet für das ausgewählte Bild den [Bildeditor](IMAGE_EDITOR.md). Export erzeugt eine eigenständige PNG-/JPEG-Variante im selben Ordner; Original und Herkunft bleiben erhalten. Die Herkunftskette unterscheidet Dateikopie und Bildbearbeitung.

Die Galerie ist die gemeinsame lokale Medienablage für Bilder, Videos, Audio und Musik. Der erste Dateibrowser liest echte Dateien im Galeriepfad aus den Einstellungen. Er legt keine Kopie aller Medien in einer versteckten Datenbank an.

1. **Galerie** öffnen. Standardmäßig werden Medien aus dem Galerieordner und seinen Unterordnern angezeigt. Ordnerbuttons wechseln in einen echten Unterordner; **Unterordner einbeziehen** bestimmt den Suchumfang.
2. **Dateien kopieren …** öffnet die Dateiauswahl. Importierte Dateien werden in den aktuell geöffneten Ordner kopiert. Die Quelle bleibt erhalten; gleichnamige Zieldateien werden niemals überschrieben. Bis zu 100 Dateien und 8 GiB je Datei; Dateien mit Teilfehlern werden einzeln gemeldet.
3. Alternativ **Ordner im Explorer** öffnen und dort Medien ablegen. Windows meldet Änderungen direkt an die Galerie. Ereignisse werden kurz zusammengefasst; eine Prüfung alle 30 Sekunden ergänzt die Überwachung. Falls der Watcher ausfällt, erfolgt die Aktualisierung alle 3 Sekunden. Kopierte Dateien brauchen keinen Bildauftrag, um zu erscheinen.
4. Nach Dateiname/Pfad, Tags oder zugeordnetem Modell/Prompt suchen oder **Bilder**, **Video**, **Audio / Musik** filtern. Die Liste hat 50 Medien pro Seite. Das Raster lädt kleine Bild- und Videominiaturen; die große Vorschau lädt nur das ausgewählte Medium.
5. Bilder lassen sich zoomen, einpassen, im vergrößerten Ausschnitt scrollen und im Vollbild ansehen. Video und Audio haben Wiedergabesteuerung ohne Autoplay. **Vorheriges/Nächstes Medium** oder Pfeiltasten bei fokussiertem Vorschaufenster wechseln innerhalb der aktuellen Seite.
6. Eigene generierte Bilder bieten **Einstellungen wiederherstellen**, wenn die gespeicherte Datei weiterhin dem lokalen Bildauftrag zugeordnet ist. Normale importierte Medien erhalten keine erfundenen KI-Metadaten.

Erfasste Endungen: PNG, JPG/JPEG, WebP, GIF, BMP, AVIF; MP4/M4V, WebM, MOV, MKV, AVI; MP3, WAV, OGG, Opus, FLAC, M4A. Auflistung ist keine Codec-Zusage: Die Wiedergabe hängt vom tatsächlich enthaltenen Codec und der Windows/WebView-Unterstützung ab. Fehlerhafte Dateien bleiben unverändert. Bildvorschauen sind auf 32 MiB Dateigröße begrenzt; Audio/Video werden in höchstens 4 MiB großen Bereichen gelesen.

Die Suche besucht höchstens 20.000 Verzeichniseinträge bis 32 Ebenen Tiefe. Maximal 200 direkte Ordnerbuttons; ein Hinweis meldet Begrenzungen und übersprungene Einträge. Bei großen Beständen einen Unterordner auswählen. Links/Reparse-Points und Netzwerkpfade werden nicht verfolgt. Keine automatische Inhaltsanalyse oder Übertragung nach außen.

Noch offen: echte Dateisystem-Watcher, Video-/AVIF-Miniaturen und vollständige Virtualisierung, seitenübergreifende Auswahl, Sammelkopieren, Drag-and-drop mit Verschieben/Strg-Kopieren, Papierkorb-Aufbewahrungsregeln, automatischer Vergleich abgeleiteter Versionen, laufwerksübergreifende Herkunftszuordnung und vollständige Versionen und vollständige Projekte. Nach einem abgebrochenen Kopierimport kann eine nicht veröffentlichte `.tmp`-Datei im Ziel bleiben; Originale bleiben erhalten. Der Galeriepfad folgt derzeit dem Datenstamm; eigenständiger Pfadwechsel mit geprüftem Verschieben folgt separat.

Technische Grundlage: Der Medienhandler nutzt den aufrufenden WebView-Namen aus [Tauris UriSchemeContext](https://docs.rs/tauri/latest/tauri/struct.UriSchemeContext.html). Kopierimporte werden mit [MoveFileExW](https://learn.microsoft.com/de-de/windows/win32/api/winbase/nf-winbase-movefileexw) ohne `MOVEFILE_REPLACE_EXISTING` veröffentlicht. Diese Begrenzungen ergänzen die lokalen IPC-Berechtigungen. Prüfnachweise stehen in [TEST_MATRIX.md](../TEST_MATRIX.md).


## Favoriten und Tags

Unter der Medienvorschau **Zu Favoriten hinzufügen** oder **Aus Favoriten entfernen** wählen. Im Feld **Neuer Tag** einen einzelnen Namen eingeben und Enter oder **Tag hinzufügen** drücken. Das × am Tag entfernt ihn. Jede dieser Aktionen speichert sofort; ein lediglich eingetippter, noch nicht hinzugefügter Tag ist ein Entwurf und wird beim Wechsel des Mediums verworfen.

Bis zu 32 Tags pro Datei, je 64 Zeichen. Doppelte Namen werden unabhängig von Groß-/Kleinschreibung zusammengefasst; Leerzeichen werden bereinigt. Kommas und Steuerzeichen sind nicht erlaubt. Die Datei selbst wird nicht geändert.

**Nur Favoriten**, **Tagfilter**, Suche und Medientyp können kombiniert werden. Die Tagauswahl bezieht sich auf tatsächlich gefundene Dateien im geöffneten Ordner einschließlich des gewählten Unterordnerumfangs. Beim Umbenennen innerhalb derselben Galerie auf NTFS bleiben die Markierungen erhalten. Eine neue Kopie erhält eigene Markierungen; die Zuordnung folgt keiner beliebigen Datei, die später denselben Namen bekommt.

Die lokale Datei config/gallery.sqlite3 enthält die Markierungen. Die Zuordnung basiert auf [Windows FILE_ID_INFO](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_id_info), ergänzt um Erstellungszeit und Galerie-Wurzel. Andere PCs, Laufwerke und geänderte Galerie-Wurzelpfade werden noch nicht automatisch neu zugeordnet. Bei Dateisystemen ohne stabile Kennung bleibt der Viewer verfügbar, Markierungen werden deaktiviert. Hardlinks innerhalb derselben Galerie teilen die Identität. Bearbeitung des Inhalts derselben Datei behält die Markierungen.

Bei einem Schreibkonflikt wird nichts still überschrieben: Ansicht aktualisieren lassen und die gewünschte Aktion erneut ausführen. Bei Speicherfehlern bleiben die vorherigen Markierungen erhalten.


## Raster, Liste und Vorschaubilder

Über die beiden Symbole neben **Medien** zwischen **Rasteransicht** und **Listenansicht** wechseln. Die Auswahl bleibt erhalten; die gewählte Ansicht gilt auch beim nächsten Start. Suche, Favoriten, Tags und Seitenwechsel funktionieren in beiden Ansichten.

Das Raster erzeugt kleine Vorschauen für PNG, JPEG, WebP, GIF und BMP. Seitenverhältnis, Transparenz und EXIF-Ausrichtung bleiben in den Miniaturen erhalten. Animierte Bilder zeigen eine statische Vorschau; Video, Audio und AVIF bekommen zunächst Symbolkarten. Anklicken öffnet weiterhin die große Medienvorschau. **Keine Miniatur** mit Erklärung im Tooltip bedeutet nicht, dass die Originaldatei gelöscht wurde.

Nur sichtbare Karten starten eine Vorschauanforderung, höchstens zwei parallel. Miniaturen haben maximal 320 Pixel je Achse. Quelldateien sind auf 64 MiB, 32 Megapixel und 16.384 Pixel je Achse begrenzt. Die große Bildvorschau hat unabhängig davon ihre bisherige 32-MiB-Dateigrenze. Keine zugesicherte ICC-/HDR-Farbtreue der Miniaturen.

Der lokale Cache liegt unter cache/gallery-thumbnails-v1.sqlite3 im aktuellen Datenstamm, getrennt von Favoriten/Tags. Er hält bis zu 2.000 Miniaturen mit zusammen höchstens 64 MiB PNG-Daten; SQLite benötigt zusätzlich Verwaltungs- und Freiseiten. **Vorschaubilder neu aufbauen** leert und komprimiert diese Datenbank und lädt sichtbare Miniaturen neu. Originaldateien und Markierungen bleiben erhalten. Der Cache darf bei geschlossener App entfernt werden; er entsteht bei Bedarf erneut.

Dateiänderungen werden anhand von Identität, Größe und vollständigen Windows-Zeitstempeln erkannt. Bei einem Werkzeug, das sämtliche Kennungen absichtlich erhält, den Cache manuell neu aufbauen. Kann die App den Cache nicht schreiben, wird die Miniatur trotzdem erzeugt; der Tooltip weist auf den fehlenden Cache hin.

Technik: [image 0.25.10](https://docs.rs/image/0.25.10/image/) mit ausschließlich den benötigten Bildformaten. Weil [Codec-Allokationslimits nur best effort sind](https://docs.rs/image/0.25.10/image/struct.Limits.html), prüft Local Studio zusätzlich Bildmaße, Pixelzahl und dekodierte Ausgabelänge vor der Pixeldekodierung. Keine Modell- oder Inferenzabhängigkeiten werden für Miniaturen installiert.


## Mehrere Medien gemeinsam markieren

Die Kästchen an den Medienkarten wählen Dateien für Sammelaktionen aus. Ein Klick auf das Bild bzw. den Dateinamen öffnet weiterhin die große Einzelvorschau. **Alle auf dieser Seite auswählen** umfasst höchstens die 50 Medien der aktuellen Seite. **Auswahl aufheben** entfernt nur die Auswahl, keine Datei oder gespeicherte Markierung.

Bei einer Auswahl erscheinen gemeinsame Aktionen:

- **Auswahl zu Favoriten** oder **Favoriten der Auswahl aufheben**.
- Einen Namen unter **Tag für Auswahl** eingeben und **Tag zur Auswahl hinzufügen** oder **Tag aus Auswahl entfernen** wählen. Hinzufügen ergänzt vorhandene Tags; Entfernen betrifft nur diesen Tag, unabhängig von Groß-/Kleinschreibung.

Die Aktion speichert alle ausgewählten Markierungen gemeinsam. Wenn auch nur eine Datei fehlt, ersetzt/geändert wurde, veraltete Markierungen hat oder das Taglimit überschreiten würde, wird keine Datei der Auswahl geändert. Eine Meldung erklärt den Fehler. Danach erneut auswählen und die gewünschte Aktion wiederholen.

Nach jeder Aktion wird die Auswahl aufgehoben. Ebenso bei Such-, Filter-, Ordner- und Seitenwechsel; Raster-/Listenwechsel erhält sie. Geänderte oder nicht mehr angezeigte Medien fallen aus der Auswahl. Während die neue Liste lädt oder eine Aktion läuft, ist keine weitere Auswahl möglich. Die Auswahl ist eine vorübergehende Bedienhilfe, kein gespeichertes Projekt.

Favoriten-/Tagaktionen ändern die Originaldateien nicht. Die zusätzlichen Dateiaktionen werden im nächsten Abschnitt beschrieben; Sammelkopieren bleibt offen.


## Auswahl, Umbenennen und Verschieben

**Strg + Klick** wählt ein Medium zusätzlich aus oder ab. **Umschalt + Klick** wählt den Bereich ab dem letzten angeklickten Medium auf der sichtbaren Seite; **Strg + Umschalt + Klick** ergänzt die bisherige Auswahl. Ein normaler Klick öffnet die Vorschau und hebt die Sammelauswahl auf. Die Kästchen bleiben als Alternative erhalten.

**Umbenennen**, **Verschieben …** und **In Papierkorb …** gelten für die aktuelle Sammelauswahl, andernfalls für das Medium in der Vorschau. Der Dialog nennt die betroffenen Dateien. Umbenennen ist für eine einzelne Datei möglich; die Dateiendung bleibt unverändert. Verschieben bietet echte Zielordner innerhalb derselben Galerie an. Andere Laufwerke, externe Ordner und das bloße Ändern der Groß-/Kleinschreibung eines Namens sind noch nicht unterstützt.

Identität und Dateiversion werden für alle ausgewählten Dateien geprüft, bevor die erste Datei verändert wird. Vorhandene Ziele werden niemals überschrieben. Ein späterer Betriebssystem- oder Speicherfehler hält den Vorgang an; die App nennt abgeschlossene Schritte und den Fehler. Anders als Sammelmarkierungen sind mehrere Dateisystemänderungen **keine gemeinsame atomare Transaktion**. Ein lokales Vorgangsprotokoll gleicht nach einem Abbruch bereits vollzogene Schritte ab. Es startet keine weiteren Verschiebungen oder Löschungen automatisch. Bei einem ungelösten Recoveryfehler die Dateien unverändert lassen.

Technisch pinnt die App Dateien und Elternordner und verwendet [SetFileInformationByHandle](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-setfileinformationbyhandle) mit [FILE_RENAME_INFO](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_rename_info) ohne Ersetzen vorhandener Ziele. Favoriten/Tags bleiben an der Dateiidentität; bekannte Bildaufträge erhalten den neuen Speicherpfad.

## Lokaler App-Papierkorb

**In Papierkorb …** benötigt eine Bestätigung. Die Medien werden unter `.local-studio-trash` im Galerieordner aufbewahrt und aus normalen Galerieansichten ausgeschlossen. Die Zuordnung mit ursprünglichem Pfad, Löschdatum und Größe liegt in `config/gallery.sqlite3`. Der Papierkorb ist kein Windows-Papierkorb. Medienablage und diese Datenbank beim Update gemeinsam behalten.

**Papierkorb** öffnet die eigene Liste mit Bild-/Video-/Audiovorschau. **Auswahl wiederherstellen** legt die Dateien an den ursprünglichen Ort zurück, wenn der Ordner noch existiert und der Name frei ist. Namenskonflikte überschreiben weder die aufbewahrte noch die neue Datei. Pro Seite sind bis zu 50 Medien auswählbar.

**Endgültig löschen …** und **Papierkorb leeren …** benötigen eine zweite ausdrückliche Bestätigung. Leeren umfasst die beim Öffnen der Bestätigung erfassten Einträge, maximal 20.000; die Ausführung erfolgt in Gruppen von 50 und stoppt bei Fehlern. Später hinzugekommene Einträge werden nicht ungefragt erfasst. Zugeordnete temporäre Originalbilder werden vor dem Löschen entfernt; der gesamte Vorschau-Cache wird geleert. Generierungsmetadaten bleiben erhalten, die Aufträge bieten aber keine erneute Ausgabe/Speicherung der gelöschten Bilder. Unbekannte Dateien, externe Importquellen und vom Nutzer erstellte weitere Kopien werden nicht gelöscht. Kein sicheres Überschreiben freier Datenträgersektoren. Es gibt noch keine automatische Papierkorb-Leerung.

## Modell, Prompt und Herkunft

Die Suche berücksichtigt Modellname, positiven und negativen Prompt zugeordneter Generierungen zusätzlich zu Pfad und Tags, jeweils vor der Seitenauswahl. Im Viewer stehen verfügbare Bildmaße sowie Modellname/-Hash, Runtime, Erstellungszeit, Auftragskennung und Generierungsparameter. Importierte Medien bekommen keine erfundenen Generierungsdaten. Video-/Audiolaufzeit und Auflösung sämtlicher Formate folgen separat.

Neue Speichervorgänge ab 0.13.0 halten die Windows-Dateiidentität und einen Größen-/Zeitstempel fest. Bei älteren Bildaufträgen basiert die einmalige erste Zuordnung auf dem bekannten gespeicherten Pfad; eine bereits vorher extern ausgetauschte Altdatei ist dabei nicht sicher erkennbar. Danach folgen Metadaten der registrierten Identität innerhalb derselben Galerie, während erkannte Inhaltsänderungen die Generierungszuordnung ausblenden. Diese Kennungen sind kein kryptografischer Inhaltsnachweis. App-interne Umbenennungen und Verschiebungen aktualisieren die Bildaufträge. Externe Verschiebungen können zwar die katalogisierte Identität behalten, aktualisieren aber noch nicht zuverlässig alle alten Auftragspfade. Vollständige Herkunftsgraphen, Versionen und Übertragung auf andere PCs/Laufwerke bleiben offen.


## Zwei Bilder vergleichen

Genau zwei Bilder mit den Kästchen oder Strg-Klick auswählen und **Zwei Bilder vergleichen** öffnen. A/B bezeichnet die Reihenfolge der Auswahl und behauptet keine automatische Vorher-/Nachher-Beziehung. Es werden die Originalbilder geladen, keine hochskalierten Miniaturen. Diese Ansicht verändert oder exportiert keine Datei.

- **Nebeneinander** zeigt beide Bilder in gleich großen Ansichtsflächen.
- **Schieberegler** zeigt beide Bilder auf einer gemeinsamen Fläche. Die Trennlinie lässt sich direkt ziehen oder über den Regler verändern. Bei fokussierter Trennlinie funktionieren Links/Rechts und Pos1/Ende; die Endpunkte zeigen ein vollständiges Bild.
- **A/B tauschen** wechselt die Seiten. **Gemeinsamer Zoom** reicht von Einpassen (100 %) bis 800 % relativ zur eingepassten Ansicht, nicht zur nativen Pixelgröße.
- Bei Zoom mit der Maus im Bild ziehen: Beide Ausschnitte bewegen sich gemeinsam. Bei fokussierter Bildfläche verschieben die Pfeiltasten den Ausschnitt, +/− ändern den Zoom und 0 setzt zurück. **Ansicht zurücksetzen** passt beide Bilder wieder ein.
- **Vergleich schließen** oder Esc schließt die Ansicht und erhält die Galerieauswahl.

Unterschiedliche Seitenverhältnisse werden unverzerrt in dieselbe Ansichtsfläche eingepasst. Keine automatische Registrierung, keine zugesicherte pixelgenaue Überlagerung, ICC-/HDR-Farbreferenz oder Animationssynchronisation. Modell/Seed und Bildmaße erscheinen, soweit bekannt; importierte Bilder bekommen keine erfundenen Generierungsdaten.

PNG, JPEG, WebP, GIF und BMP; höchstens zwei Bilder mit jeweils 32 MiB, 32 Megapixeln und 16.384 Pixeln pro Achse. Grenzen werden vor dem Laden geprüft; beschädigte oder während des Ladens ersetzte Dateien erzeugen eine Meldung. Der Medienabruf prüft die ausgewählte Dateiversion erneut. Schon vollständig geladene Pixel bleiben bis zum Schließen als Ansicht im Arbeitsspeicher; es wird keine neue Bilddatei gespeichert. Geänderte Quellen in der Galerie neu auswählen. Vollständige Herkunftsgraphen und automatische Versionsgruppen folgen separat.

## Sortierung und Tastatur

**Sortieren** bietet Name A–Z/Z–A, Änderungsdatum neu/alt und Größe groß/klein. Die Sortierung gilt für alle gefundenen Treffer innerhalb der bestehenden Scan-Grenzen, bevor die 50er-Seiten gebildet werden. Namen werden ohne Beachtung der Groß-/Kleinschreibung lexikografisch verglichen; der vollständige Pfad macht gleiche Werte eindeutig. Ordner bleiben alphabetisch. Die letzte Sortierwahl bleibt über Neustarts erhalten. Ein Wechsel setzt die Seite zurück und hebt die Sammelauswahl auf.

Wenn eine Medienkarte bzw. die Galerie fokussiert ist:

- **Strg+A**: alle Medien auf der aktuellen Seite auswählen.
- **Esc**: Sammelauswahl aufheben.
- **Links/Rechts**: vorherige/nächste Medienkarte samt Vorschau. **Umschalt+Links/Rechts** erweitert den Bereich.
- **F2**: vorhandenen Umbenennen-Dialog für die einzelne Auswahl bzw. Vorschau öffnen.
- **Entf**: vorhandenen Bestätigungsdialog für den Papierkorb öffnen; keine sofortige Löschung.

Texteingaben, Formularfelder, Mediensteuerung und offene Dialoge behalten ihre eigene Tastaturbedienung. Diese Galeriebelegungen sind noch nicht frei konfigurierbar; das übergreifende Shortcut-System bleibt im Plan.

## Direkte Übernahme ab 0.17.0

Dateien aus Explorer auf die Galerie ziehen: Sie werden in den aktuell geöffneten Galerieordner kopiert. Originale bleiben erhalten; ungültige Dateien und Teilfehler werden gemeldet. Während Dialogen oder anderer Dateiaktionen erfolgt kein Import. **Auswahl ins Projekt übernehmen** kopiert die Mehrfachauswahl, ersatzweise das aktuell angezeigte Medium, in das aktive Projekt. Ohne Projekt wird ein unbenanntes angelegt. Dieselben Dateien lassen sich auf die Projektleiste ziehen. Veraltete Galerie-Dateiidentitäten oder Versionen werden abgewiesen.

## Video-Miniaturen

Sichtbare Videokarten decodieren bei Bedarf ein einzelnes lokales Frame über die verfügbaren Windows/WebView-Codecs. Kein Autoplay oder Ton. Höchstens zwei sichtbare Karten werden gleichzeitig verarbeitet; nicht sichtbare Karten warten. Die Vorschau ist höchstens 320 Pixel groß und wird mit der aktuellen Dateiidentität/-version im vorhandenen lokalen Cache gespeichert. Bei Codecfehlern, beschädigten oder zu großen Bildmaßen bleibt eine Symbolkarte mit Hinweis. Abbruch beim Verlassen und ein Zeitlimit geben die Quelle wieder frei. **Vorschaubilder neu aufbauen** entfernt ausschließlich den Cache. Originale bleiben byteidentisch.

## Herkunft und Varianten

Im Viewer unter **Herkunft und Varianten** legt **Kopie als Variante anlegen** eine eigene byteidentische Datei im gleichen Ordner an. Das ist ausdrücklich eine Kopie und noch keine Bildbearbeitung. Neue Namen sind eindeutig; vorhandene Dateien werden nicht überschrieben. Die neue Datei erhält eine Verbindung zum ausgewählten Quellmedium und dessen Versionsgruppe. Eine Gruppe enthält bis zu 100 Metadateneinträge.

Die Kette zeigt Original/Import, bekannte lokale Bildgenerierung oder Variantenkopie und ihre jeweilige Quelle. Vorhandene Generierungsdaten (Modell, Auftrag, Prompt, Parameter, Modellhash) bleiben nachvollziehbar. Normale Importe bekommen keine erfundene KI-Herkunft. Die neueste vorhandene Variante ist standardmäßig Hauptversion; **Als Hauptversion** legt eine andere fest. **Hauptversion öffnen** bzw. **Version öffnen** wechselt zur Datei. Das Raster zeigt weiterhin die einzelnen echten Dateien; es fasst eine Gruppe noch nicht zu einer einzigen Karte zusammen.

Zuordnungen verwenden Datei-ID und Inhaltszeitstempel. Internes oder innerhalb der Galerie externes Umbenennen lässt sich wieder zuordnen; die Suche ist auf die bestehenden 20.000 Einträge/32 Ordnerebenen begrenzt. Veränderte, fehlende oder im Papierkorb liegende Quellen werden nicht als unveränderte Version geöffnet. Nach endgültigem Löschen bleibt nur der Metadateneintrag; es wird keine versteckte Quellenkopie aufbewahrt. Andere Varianten bleiben unabhängig erhalten. Externe Bearbeitungen erzeugen nicht automatisch eine behauptete Bearbeitungskette.

Die Herkunftsdaten liegen lokal in `gallery.sqlite3`, die Dateien bleiben echte Galeriedateien. Projektcontainer transportieren diese zusätzlichen Galeriebeziehungen noch nicht. KI-/Editoroperationen und vollständige Versionsgruppenansichten bleiben weitere Arbeit.
