# Local Studio 0.26.0 – lokale 18+-Sperre

- Altersbestätigung, lokale PIN oder Passwort, standardmäßig erneutes Sperren nach Neustart. Zugangsdaten ändern, sofort sperren und Modus ausschalten.
- Lokale Modellkennzeichnung und Übernahme eindeutiger NSFW-Tags bei HF-Downloads. Modelle bleiben sichtbar und auswählbar.
- Geschützte Prompts, Ergebnisse, Galerie-Metadaten, Suchergebnisse und Arbeitsbereiche. Laufende Bildgenerierungen laufen beim Sperren weiter; neue geschützte Aufträge werden im Backend abgewiesen.
- Herkunftsschutz für Kopien, Bearbeitungen und Exporte; geschützte Projekte reisen im neuen Format 5. Chatmodelle und betroffene Verläufe sind einbezogen.

Einrichtung über **18+ · Gesperrt** oben im Hauptfenster. Die Sperre schützt App-Zugriffe und verschlüsselt keine Dateien auf dem Datenträger. Normale importierte Medien erhalten keinen manuellen 18+-Schalter. Gemischte ältere Verlaufs- und Verwaltungsansichten verlangen vorsorglich die Entsperrung. Anleitung: `docs/PRIVACY.md` im Quellcode.

Beim portablen Update die App schließen, Programmdateien und alle vier Runtime-Ordner ersetzen und **Local-Studio-Data behalten**. Geschützte Projekte benötigen Version 0.26.0 oder neuer. Modelle sind nicht beigepackt. Der weitere Masterprompt-Umfang bleibt erhalten; insbesondere Inhaltsanalyse, semantischer Index und Erweiterungsadapter sind damit nicht als fertig ausgewiesen.
