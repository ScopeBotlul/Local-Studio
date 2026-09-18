# Speicherorte und Tastenkürzel ab 0.15.0

Unter **Einstellungen → Datenordner → Einzelne Speicherorte anpassen** lassen sich Modelle, Assistentenmodelle, Visionmodelle, Downloads, Galerie, Projekte, temporäre Dateien, Recovery, Cache und Proxies getrennt wählen. Ein leeres Feld verwendet den Standard unter dem Datenordner. **Standard** entfernt die jeweilige Pfadabweichung; **Änderungen speichern** prüft und übernimmt die Auswahl.

Vorhandene Dateien werden nicht verschoben, kopiert oder gelöscht. Für die bisherige Galerie den bisherigen Galeriepfad eintragen. Konfiguration und Auftragsdatenbank bleiben am bisherigen Ort. Ab 0.16.0 verwenden Projekte den Projekte-Pfad als Speichervorgabe und den Recovery-Pfad für lokale Arbeitskopien. Proxies bleiben vorbereitet; die Bild-Arbeitsstand-Recovery bleibt in der Konfigurationsdatenbank.

Aktive Downloads behalten ihre geprüften Quell-/Zielpfade. Neue Bildaufträge speichern temporär unter `<Temporäre Dateien>/image-results/<Auftrags-ID>`. Jeder Auftrag behält diesen Pfad über Einstellungen, Warteschlange und Neustarts hinweg. Ältere Aufträge unter `config/image-results` bleiben lesbar. Galerie-Speichern verwendet den zu diesem Zeitpunkt eingestellten Galerieordner. Die Modellschnellsuche berücksichtigt alle drei Modellordner. Bildminiaturen verwenden den eingestellten Cache.

Ungültige, nicht beschreibbare oder verknüpfte Ordner werden abgewiesen. Kategorien dürfen nicht im selben Ordner oder ineinander liegen; nur Modell-/Assistenten-/Visionordner dürfen verschachtelt sein. Ein fehlgeschlagenes Speichern erhält die vorherigen Einstellungen. Bereits angelegte leere Ordner können nach einer fehlgeschlagenen Schreibprüfung verbleiben.

Unter **Tastenkürzel** auf eine Belegung klicken und die neue Kombination drücken. Escape bricht die Aufnahme ab. Konflikte und reservierte Kombinationen werden sofort angezeigt und beim Speichern nochmals im Backend geprüft. **Entfernen** deaktiviert ein Kürzel, **Standardbelegung wiederherstellen** setzt alle zurück. Änderungen gelten erst nach dem Speichern und bleiben nach Neustart erhalten.

| Aktion | Standard | Gültig in |
| --- | --- | --- |
| Projekt speichern | Strg+S | Hauptfenster, auch im Prompt; ab 0.16.0 |
| Bearbeitung rückgängig / wiederholen | Strg+Z / Strg+Y | Bildeditor außerhalb von Eingabefeldern; ab 0.19.0 |
| Bild generieren/einreihen | Strg+Enter | Studio, auch im Prompt; erst nach Ausführbarkeitsprüfung |
| Galerieauswahl | Strg+A | Galerie außerhalb von Eingabefeldern |
| Umbenennen | F2 | Galerie; vorhandener Dialog |
| Papierkorb | Entf | Galerie; vorhandene Bestätigung |
| Zwei Bilder vergleichen | C | Galerie, genau zwei ausgewählte Bilder |
| Bild einpassen / zurücksetzen | F / 0 | Fokussierter Bildbereich; auch Vergleich |
| Bild in 100 % | 1 | Fokussierte Galerie-Medienvorschau |
| Oberfläche größer / kleiner / 100 % | Strg+Plus / Strg+Minus / Strg+0 | Hauptfenster |

Strg+Mausrad bleibt für UI-Zoom verfügbar. Escape, Tab, Pfeilnavigation und normale Textbearbeitung bleiben kontextabhängige Standardbedienung. Strg+S gehört ab 0.16.0 zum Projekt-Speicherbefehl, Strg+Z/Y ab 0.19.0 zur Bildbearbeitung. Ältere Belegungen erhalten neue Einträge beim Laden. Undo/Redo für weitere Medieneditoren bleibt offen.

Die Galerie-Bildvorschau passt auch Hochformatbilder vollständig ein. **100 %** zeigt ein Bildpixel pro logischem Bildschirmpixel bei der eingestellten UI-Skalierung. Der Zoomregler vergrößert die aktuelle Ansicht; Ziehen verschiebt den Ausschnitt. Einpassen/Rücksetzen setzt den Ausschnitt zurück. Der Zwei-Bild-Vergleich behält seinen gemeinsamen relativen Zoom, keine automatische pixelgenaue Registrierung.

## Speicher und Bereinigung ab 0.17.0

**Speicher prüfen** zeigt freigebbare Dateien/Bytes und geschützte bekannte Arbeitsdateien. Es ist keine vollständige Laufwerksbelegungsanalyse. **Betroffene Dateien ansehen** nennt die konkreten Pfade; **Jetzt bereinigen** verlangt eine Bestätigung. Abbrechen erhält alle Dateien. Die Vorschau gilt fünf Minuten und für die gespeicherte Aufbewahrungsfrist; vor jedem Löschen werden Schutzstatus und Dateiidentität erneut geprüft. Geänderte, inzwischen wieder benötigte oder nicht prüfbare Dateien werden übersprungen. Teilergebnisse und Fehler sind sichtbar. Pro Durchlauf werden höchstens 1.000 Dateien verarbeitet.

**Automatisch bereinigen** ist standardmäßig eingeschaltet, auch beim Laden älterer Einstellungen ohne diesen Wert. Die App prüft nach dem Start mit kurzer Verzögerung und danach höchstens stündlich. Standardfrist sieben Tage, einstellbar von 1 bis 365 Tagen. Änderungen an Einstellungen zuerst speichern. Die gleichen Schutzprüfungen gelten bei manueller und automatischer Bereinigung.

Erfasst werden registrierte Projektarbeitskopien abgeschlossener, alter Sitzungen und bekannte Dateien abgeschlossener/abgebrochener Bildaufträge. Aktives Projekt, die letzten drei Wiederherstellungsstände, jüngere Stände, laufende/pausierte Aufträge und ungespeicherte fertige Bilder bleiben geschützt. Temporäre fertige PNGs werden nur entfernt, wenn eine separat gespeicherte Galerieversion noch ihre geprüfte Identität und identische Bildbytes besitzt. Die Galerieversion bleibt anschließend als Ergebnis verwendbar.

Modelle, Galerieoriginale, Downloads einschließlich `.part`, unbekannte Zusatzdateien und der Galeriepapierkorb werden nicht bereinigt. Alte unregistrierte Projektarbeitskopien aus früheren Versionen werden nicht pauschal durchsucht oder gelöscht. Leere Arbeitsordner können bestehen bleiben. Die Bereinigung ist kein rekursives Löschen beliebiger temporärer Ordner.

**Zurückholbare Projektmedien** legt die maximale lokale Liste entfernter Medien fest (1–1.000, Standard 100); die Grenze greift beim nächsten Entfernen. Die `.localstudio`-Datei enthält weiterhin nur aktive Medien.
