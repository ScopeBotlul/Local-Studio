# Local Studio 0.32.0

## Änderungen

- Ist **Beim Start automatisch nach Updates suchen** aktiv, prüft Local Studio einmal verzögert nach dem Start. Ein gefundenes Update öffnet automatisch den vorhandenen Update-Dialog mit **Abbrechen** und, bei einer installierten Ausgabe, **Update installieren**. Der Dialog wartet, bis kein anderer modaler Dialog mehr geöffnet ist. Portable Ausgaben bleiben beim sicheren manuellen ZIP-Update.
- Die bisherige lokale 18+-Sperre wurde vollständig aus der Oberfläche und der öffentlichen Desktop-IPC entfernt. Es gibt keine Altersabfrage, PIN, Passwort, Neustartsperre, gesperrten Platzhalter oder ausgeblendete ältere Inhalte mehr.
- Vorhandene Kennzeichnungen in älteren Datenformaten bleiben lesbar und werden ignoriert. 18+-Markierungen von Hugging Face oder Civitai dienen nur noch als sichtbare Metadaten beziehungsweise Suchfilter.

## Aktualisierung

Installierte Ausgaben können das signierte Update im Update-Dialog herunterladen und installieren. Portable Ausgaben ersetzen die Programmdateien und alle vier Runtime-Ordner aus dem ZIP; `Local-Studio-Data` bleibt vollständig erhalten.

## Prüfumfang

Frontend-Build und 57 Frontendtests sind bestanden. Die direkt betroffenen Rust-Module bestanden 25 Tests; die breite Suite bestand 174 Tests, während ein unveränderter Windows-Anmeldespeicher-Test wegen des lokalen `secret_store`-Zugriffs fehlschlug und ein Symlinktest planmäßig ignoriert blieb. Die exakt paketierte EXE bestand den fokussierten nativen Lauf für Version, entfernte Sperr-UI/IPC, gespeicherte Updateeinstellung und echten signierten GitHub-Kanal. Paketbytes, Runtime-Manifeste, Hashes und Update-Signatur stimmen. Unveränderte Inferenz-, Modellqualitäts- und Installer-Lebenszyklus-Suiten wurden nicht allein wegen dieser Version wiederholt.
