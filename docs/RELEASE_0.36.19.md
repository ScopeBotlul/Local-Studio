# Local Studio 0.36.19

## Rule34 im Tag-Importer

- **Tags importieren** bietet jetzt getrennte Schaltflächen für Danbooru und Rule34.
- Ein einzelner Rule34-Post wird im vorhandenen, isolierten Browser innerhalb von Local Studio geöffnet. Sobald der Browser eine gültige Post-Adresse zeigt, übernimmt **Tags übernehmen** die Tags und gruppiert sie für die Prompt-Vorschau.
- Die Rule34-Seite wird nicht noch einmal im Hintergrund abgerufen. Local Studio liest ausschließlich die Tagliste aus der bereits vom Benutzer geöffneten Seite. Es werden weder Rule34-Zugangsdaten noch API-Schlüssel angefordert oder in Local Studio gespeichert.
- Erlaubt sind nur exakte HTTPS-Adressen von `rule34.xxx` und `www.rule34.xxx`. Unsichere URLs, ähnlich aussehende Fremddomains, mehrere Post-IDs sowie zu große oder unerwartete Antworten werden abgewiesen.
- Fehlermeldungen des Imports sind jetzt auf Deutsch und Englisch verständlich formuliert.

## Grenzen

Der direkte Import setzt voraus, dass die Rule34-Seite im integrierten Browser erreichbar ist und ihre Tagliste weiterhin als Post-Seitenleiste ausliefert. Loginpflichtige, blockierte oder strukturell geänderte Seiten können nicht automatisch importiert werden; das manuelle Einfügen kopierter Taglisten bleibt verfügbar. Unveränderte Inferenz-, Projekt-, Galerie-, Update- und Installerabläufe wurden für dieses kleine Update nicht erneut vollständig geprüft.
