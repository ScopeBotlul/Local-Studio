# Downloads und lokale Dateien, 0.4.0

Unter **Modelle** ein Repository öffnen, Dateien ausdrücklich auswählen und **Download prüfen** anklicken. Die Vorschau nennt Dateiauswahl, exakten Commit, Lizenz/Zugang, Ziel, Downloadgröße und zusätzlichen Speicherbedarf. **Auswahl herunterladen** bestätigt den Auftrag. Keine Variante oder Datei wird vorausgewählt. Ein heruntergeladenes Set ist noch kein nachgewiesen vollständiges oder ausführbares Modell.

**Downloads** zeigt echte Bytes, Gesamtgröße, verbleibende Bytes, gemessene Geschwindigkeit, berechenbare Restzeit, Status und Ziel. Ein Transfer läuft gleichzeitig. Priorität beeinflusst den nächsten wartenden Auftrag; sie unterbricht keine laufende Übertragung. Pause und Abbruch signalisieren dem asynchronen HTTP-Transfer das Ende und schließen seine Response; erst danach wird der angehaltene Zustand angezeigt. Teil-Dateien bleiben erhalten. Retry/Fortsetzen verwendet die tatsächliche Dateilänge, nicht einen möglicherweise veralteten Datenbankzähler.

Fortsetzung verlangt eine passende HTTP-206-Antwort samt exakt geprüftem Content-Range. Bei HTTP 200 auf eine Range-Anfrage meldet die App fehlende bytegenaue Unterstützung und behält die Teil-Datei. **Neu beginnen** verwirft nach Bestätigung ausschließlich die bekannten Teil-Dateien dieses Auftrags. Nach einem App-Neustart stehen zuvor aktive/wartende Aufträge auf pausiert; es gibt keine unbemerkte automatische Wiederaufnahme.

Die SQLite-Datei `config/downloads.sqlite3` speichert Metadaten, Dateien, Fortschritt und Status. Sie enthält keine Tokens oder signierten CDN-Adressen. Bereits gestartete Aufträge behalten ihre Zielpfade, wenn der Datenstamm später geändert wird. Neue Vorschauen verwenden die dann aktuellen Pfade.

## Übertragung und Integrität

- Repository/Revision/Dateien werden nativ erneut über die HF-API aufgelöst. Der Transfer wird an den aufgelösten Commit gebunden. Unbekannte Größe oder fehlende Quell-Prüfsumme verhindern derzeit einen Start.
- HTTPS-Redirects werden einzeln geprüft; nur exakte HF-Domains oder deren echte Subdomains unter `huggingface.co`/`hf.co` sind erlaubt. Begrenzte Redirectzahl, Verbindungs-/Lese-Timeouts und keine freien URL-Downloads. Bearer-Tokens gehen ausschließlich an `huggingface.co`, niemals an einen CDN-Host.
- Windows-Pfade dürfen keine Traversierung, absoluten Pfade, alternativen Datenströme, reservierten Gerätenamen, problematischen Endzeichen oder kollidierenden Datei-/Ordnernamen enthalten. Reparse Points und Symlinks in geprüften Pfaden werden abgewiesen. Dies ist keine Betriebssystem-Sandbox gegen einen gleichzeitig handelnden Angreifer mit denselben lokalen Benutzerrechten.
- LFS-Dateien werden gegen HF-SHA-256 geprüft; normale Git-Dateien gegen ihren Git-Blob-SHA-1 inklusive Blob-Header. Zusätzlich wird immer SHA-256 gespeichert. Die spätere lokale Prüfung vergleicht auch diesen gespeicherten SHA-256-Wert. Ein Hash bestätigt Byte-Integrität, keine Sicherheit oder Modellkompatibilität.
- Teil-Dateien liegen im Downloadordner unter einer zufälligen Auftrags-ID. Die Auswahl wird in einen separaten Staging-Ordner im Modellordner kopiert und erneut geprüft. Erst dann veröffentlicht ein Rename den finalen Ordner. Vorhandene Zielordner werden nicht überschrieben. Nach erfolgreicher Veröffentlichung werden die bekannten vollständigen Downloadreste entfernt. Bei Fehler/Abbruch bleiben wiederverwendbare Teile erhalten.
- Die Planung kalkuliert konservativ bis zur doppelten Dateigröße plus interner Reserve. Es gibt keine Reservierung des freien Speicherplatzes für beliebige andere Programme. Schreibfehler werden als Fehler gemeldet; keine ungeprüfte Auswahl wird veröffentlicht.

## Lokale Übersicht und Grenzen

**Modelle → Lokal gespeichert** zeigt die heruntergeladenen Dateiauswahlen mit Quelle, Aufgabe, Lizenz, Commit, Dateigröße, Ziel, Status und lokalen Prüfsummen. **Lokale Dateien prüfen** liest und prüft die tatsächlichen Dateien. Beschädigte oder fehlende Dateien werden als ungültig markiert. Es werden keine Modellskripte ausgeführt oder Abhängigkeiten installiert.

Noch offen: lokale PC-Suche/Import, Erkennung vollständiger Runtime-Pakete und Erweiterungen, Reparatur-/Entfernen-/Verschieben-/Update-Abläufe, automatische Updateprüfung, direkte andere Quellen, parallele Transfers und echte Inferenz. Der Downloadmanager vervollständigt M1 nicht. Website-interne direkte Dateidownloads bleiben blockiert; Modellseiten werden über **In Local Studio öffnen** an die geprüfte Dateiauswahl übergeben.

## Nachweise

`download_tests.rs` verwendet einen kontrollierten HTTP-Server mit verzögertem Datenstrom: echtes Pausieren mit stabiler Dateilänge, Range-Fortsetzung nach Wiederöffnung der SQLite-Verwaltung, Cancel mit behaltenen Teilen, nicht unterstützte Fortsetzung und expliziter Neustart, Hashfehler ohne Veröffentlichung, lokale Korruption und Prioritätsreihenfolge. Die lokale HTTP-Ausnahme und freie Test-URL existieren ausschließlich unter `cfg(test)` und sind im ausgelieferten Programm nicht vorhanden.

`scripts/check-downloads.mjs` lädt im isolierten nativen Test `config.json` und `model.safetensors` aus dem öffentlichen HF-Testrepository `hf-internal-testing/tiny-random-gpt2`, überprüft Bytes/Hashes, manipuliert ausschließlich die eigene Testkopie zur Korruptionserkennung und stellt sie wieder her. Dieses interne HF-Testrepository ist nicht im normalen Suchindex gelistet; der Test nutzt deshalb die echte Website-zu-Modell-Übergabe. Es ist kein Assistentenmodell und wird nicht ausgeführt oder mitgeliefert. Aktuelle Laufnachweise stehen in `TEST_MATRIX.md`.

Primärquellen, geprüft am 17. September 2026: [HF-Downloadpfad und CDN-Domains](https://huggingface.co/docs/hub/models-downloading), [Revisionen und Dateimetadaten](https://huggingface.co/docs/huggingface_hub/package_reference/file_download).

## Größenanzeige ab 0.4.1

Die Oberfläche zeigt dezimale GB. Suchkarten/Details summieren alle Dateien der angegebenen Revision, einschließlich Alternativvarianten. Die gewählten Dateien bestimmen die Downloadgröße. Unvollständige Größen werden nicht als genaue Summe dargestellt. Lokale Einträge zeigen die Größe der ursprünglich gespeicherten Auswahl; die vorhandene Integritätsprüfung erkennt nachträglich fehlende oder veränderte Dateien.
