# Local Studio 0.36.0

## Fehler direkt aus Local Studio melden

- Unter **Hilfe → Fehler melden …** sammelt Local Studio automatisch App-Konfiguration, Windows- und Hardwaredaten, Komponentenstatus sowie begrenzte lokale Protokolle.
- Der vollständige Bericht ist vor dem Speichern sichtbar und wird als Markdown-Datei im lokalen Berichtordner abgelegt.
- Tokens, Cookies, Passwörter, Prompts, Medien, Projektinhalte, Modellgewichte, Datenbankinhalte und persönliche Pfadbestandteile werden ausgeschlossen.
- **Speichern und GitHub öffnen** öffnet ein vorausgefülltes Issue für `ScopeBotlul/Local-Studio`. GitHub verlangt dort aus Sicherheitsgründen noch die Anmeldung und den abschließenden Klick auf **Submit new issue**. Die lokale Vollversion kann an das Issue angehängt werden.
- Der Dialog ist auf Deutsch und Englisch verfügbar und zeigt transparent, welche Daten aufgenommen und ausgeschlossen werden.

## Prüfumfang

Frontend-Produktionsbuild, Rust-/Tauri-Compilercheck und zwei gezielte Backendtests für Datenredaktion und die begrenzte GitHub-Übergabe sind bestanden. Das tatsächliche Absenden eines öffentlichen Test-Issues wurde nicht automatisiert, damit kein Testbericht im Repository angelegt wird.
