# Menüleiste und Updates ab 0.20.0

Die native Windows-Menüleiste folgt Sprache und Theme der Anwendung. **Datei** bietet Neues Projekt (Strg+N), Öffnen (Strg+O), zuletzt geöffnete Projekte, Speichern (Strg+S), Speichern unter (Strg+Umschalt+S), Medien hinzufügen, Projekt fortsetzen, Schließen (Strg+W) und Beenden. Bestehende Speicher-, Konflikt- und Recovery-Dialoge gelten auch hier. Die Projektleiste zeigt weiterhin den Arbeitsstand und die Projektdetails.

**Bearbeiten** enthält Rückgängig/Wiederholen für den aktiven Bildeditor und Einstellungen. **Ansicht** steuert das aktuelle Galeriebild beziehungsweise den Bildeditor und den Oberflächenzoom. Nicht verfügbare Aktionen sind deaktiviert. Kürzel lassen sich unter Einstellungen ändern. **Hilfe** enthält die lokale Anleitung, Versionsinfo und Nach Updates suchen.

## Aktualisieren

Die erste Version mit diesem Updatekanal ist **0.20.0**. Ältere Versionen einmal manuell durch den neuen Installer beziehungsweise die portable ZIP ersetzen. [GitHub-Veröffentlichungen](https://github.com/ScopeBotlul/Local-Studio/releases).

- Installierte Ausgabe: Hilfe → Nach Updates suchen → Update herunterladen → Aktualisieren und neu starten. Ein Paket wird erst nach gültiger Ed25519-Signatur, passender Version, Dateigröße und SHA-256 angeboten. Ungespeicherte Arbeit durchläuft den normalen Beenden-Dialog; Abbrechen installiert nichts. Ein separater Helfer wartet auf das Prozessende, prüft das Paket erneut und startet das Inno-Setup im bisherigen Ordner. Anschließend startet die App erneut.
- Portable Ausgabe: zeigt neue Versionen und öffnet die GitHub-Veröffentlichung. App schließen und Programmdateien einschließlich `image-runtime` aus dem ZIP ersetzen. **Local-Studio-Data behalten.** Kein automatischer Installer oder Registry-Eintrag.
- Automatische Prüfung: standardmäßig einmal nach dem Start, abschaltbar im Updatedialog. Kein automatischer Download oder Installationszwang. Die Verbindung zu GitHub übermittelt keine Medien, Prompts oder Modellpfade. Ohne Verbindung bleibt die Anwendung nutzbar; die manuelle Prüfung zeigt den Fehler.

Heruntergeladene Pakete liegen unter `updates` im Konfigurationsordner. Installationsprotokoll und `result.txt` liegen im jeweiligen Unterordner. Nach einem App-Neustart wird ein vorheriger Download erneut angefordert; unterbrochene Downloads werden nicht fortgesetzt. Alte Updateordner werden bisher nicht automatisch bereinigt. Nach Beenden aller App-/Setup-Prozesse dürfen diese bekannten Updateordner entfernt werden. Ein abgebrochenes oder fehlgeschlagenes Setup wird nicht als erfolgreicher Versionswechsel behauptet; die installierte Versionsnummer ist unter Hilfe → Über Local Studio sichtbar.

## Veröffentlichung für Maintainer

Der feste Kanal ist `ScopeBotlul/Local-Studio`, Release-Tag `v<Version>`. Benötigte Assets: `Local-Studio-<Version>-hub-setup.exe`, `Local-Studio-<Version>-hub-portable.zip`, `SHA256SUMS-<Version>.txt`, `update.json`, `update.sig`. Erst alle Assets hochladen, dann die Veröffentlichung als neuestes Release freigeben. Vorabversionen nicht als Latest markieren.

1. Version in package.json, package-lock.json, Cargo.toml/Cargo.lock und tauri.conf.json angleichen. Frontend-/Rust- und native Prüfungen ausführen.
2. `npm run desktop:build` erzeugt die EXE und den Installer. `powershell -File scripts/package-core.ps1` erzeugt die portable ZIP und Prüfsummen. Danach keine Programmdateien mehr verändern. Nach dem Signieren `node scripts/verify-release.mjs` ausführen.
3. Release-Notizen als UTF-8-Datei schreiben. `node scripts/sign-release.mjs <Notizdatei>` signiert die exakten Paketgrößen, Hashes und URLs. Notizen auf maximal 16.000 UTF-8-Bytes begrenzen.
4. Getesteten Quellstand committen und pushen; GitHub-Release zunächst als Entwurf mit diesen fünf Assets erstellen. Assets prüfen und danach veröffentlichen. `Hilfe → Nach Updates suchen` muss die neue signierte Version erkennen.

Der öffentliche Schlüssel ist `src-tauri/update-public-key.txt`. Der private Schlüssel liegt **ausschließlich lokal**, durch Windows DPAPI für dieses Benutzerkonto geschützt, in `.tools/update-signing/private.dpapi`. Er gehört weder ins Repository noch in Release-Assets. `init` verweigert das Überschreiben vorhandener Schlüssel. Verlust des Windows-Benutzerprofils/DPAPI-Schlüsselmaterials kann die weitere Signierung verhindern: eine vollständige geschützte Sicherung dieses Profils samt Signierschlüssel außerhalb des Repositorys vorhalten. Die verschlüsselte Datei allein ist auf einem anderen PC normalerweise nicht ausreichend. Ein Schlüsselwechsel benötigt einen bewusst geplanten Übergang, kein stilles Neugenerieren.

Die Updatesignatur ist keine Windows-Authenticode-Signatur. Ein kommerzielles Codesigning-Zertifikat und eine vollständige SmartScreen-Reputationsabnahme sind nicht enthalten. Geprüft wird der bereits in der App verankerte Updateschlüssel. Auch eine gültige Signatur erlaubt keine fremden Paket-URLs, Downgrades oder portable Selbstinstallation.
