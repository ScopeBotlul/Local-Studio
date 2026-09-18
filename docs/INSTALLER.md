# Windows-Installer ab 0.4.2

Der reguläre Build verwendet Inno Setup 7.1.0 mit `WizardStyle=modern dynamic windows11`.
Setup und Deinstallation lesen beim Start die Windows-Einstellung für den App-Modus.
Nach einem Wechsel in Windows muss das Setup erneut gestartet werden. Hoher Kontrast
hat Vorrang vor benutzerdefinierten Styles. Die Windows-Einstellung wird nicht geändert.

## Bauen

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bootstrap-installer.ps1
npm run desktop:build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package-core.ps1
```

Der Bootstrap lädt den fest versionierten offiziellen Compiler, prüft SHA-256 und
Authenticode und installiert ihn ohne Verknüpfungen unter `.tools/inno-setup`.
Der Compiler trägt seine eigene Deinstallation im Benutzerbereich ein. Eine vorhandene
Installation von Inno Setup wird durch diesen projektlokalen Installationspfad nicht
als portable Toolchain behandelt. Alternativ kann `build-installer.ps1 -CompilerPath`
einen vorhandenen Inno-7.1-Compiler verwenden.

`desktop:build` baut Tauri mit `--no-bundle` und danach das Inno-Setup. Der vorherige
NSIS-Konfigurationsabschnitt bleibt für explizite Tauri-NSIS-Builds erhalten, wird im
regulären Build aber nicht verwendet. Das Ergebnis liegt unter
`src-tauri/target/release/bundle/inno/Local-Studio-0.4.2-hub-setup.exe`.
Das Paketierungsskript kopiert diesen Installer nach `releases/` und erzeugt Prüfsummen.
App- und Paketversion müssen übereinstimmen. Installer und Pakete sind nicht im Git-Repo.

## Verhalten und Wechsel von NSIS

- Installation für den aktuellen Benutzer, normalerweise unter `%LOCALAPPDATA%/Local Studio`.
- Gleiche Programmdatei und gleiche Tauri-App-ID: vorhandene App-Konfiguration bleibt zugeordnet.
- Startmenü-Verknüpfung; Desktop-Verknüpfung und App-Start nach Installation sind optional.
- Laufende Dateien werden nicht erzwungen geschlossen. Vor einem Update Local Studio beenden.
- Ein Zielordner mit `portable.marker` wird vor dem Kopieren abgewiesen.
- Eine registrierte alte NSIS-Installation wird erkannt. Diese zuerst über Windows deinstallieren,
  dabei **App-Daten nicht löschen**, dann das neue Setup starten. Es gibt keine automatische
  NSIS-Migration und keinen Aufruf einer fremden Deinstallationszeichenfolge aus der Registry.
- Deinstallation entfernt nur vom Installer verwaltete Dateien. Keine rekursive Löschung von
  Modellen, App-Daten, Projekten oder nachträglich hinzugefügten Dateien.
- WebView2 wird in den Benutzer-/Maschinen-Registryansichten erkannt. Wenn es fehlt, wird der
  beim Build eingebettete, gültig von Microsoft signierte Evergreen-Bootstrapper ausgeführt.
  Dieser benötigt Internet. Fehlgeschlagene Einrichtung stoppt die App-Installation.

## Prüfung und Grenzen

`python scripts/test-installer.py` baut ausschließlich isolierte Test-Installer mit eigener
App-ID, ohne Verknüpfungen und ohne App-Start. Geprüft werden System/Hell/Dunkel in echten
Installationen, erneute Installation, Deinstallation, Byte-Identität der App, erhaltene zusätzliche
Dateien, ein geschützter portabler Zielordner und simulierte Erkennung eines alten NSIS-Setups.
Es werden weder das Windows-Theme noch bestehende Local-Studio-Installationen verändert.

Der Systemmodus wurde auf diesem Rechner im dunklen Windows-App-Modus geprüft. Hell/Dunkel
wurden zusätzlich durch getrennte Test-Builds erzwungen, nicht durch Ändern der Systemeinstellung.
Hoher Kontrast, fehlende WebView2-Runtime, echter NSIS-Wechsel und alle sichtbaren Dialogseiten
bleiben eigene Abnahmefälle. Computer Use konnte wegen `apply deny-read ACLs` nicht starten;
die Theme-Auswahl ist anhand der Setup-/Uninstall-Protokolle geprüft, nicht anhand von Screenshots.

Der App-Installer besitzt noch kein Windows-Authenticode-Zertifikat. Ab 0.20.0 sind
Ed25519-signierte GitHub-Updates mit bestätigtem Inno-Installationsablauf integriert.
Die portable Ausgabe aktualisiert sich weiterhin manuell. [Bedienung und Signierung](UPDATES.md).

Quellen: [Inno WizardStyle](https://jrsoftware.org/ishelp/topic_setup_wizardstyle.htm),
[AppId und Deinstallationszuordnung](https://jrsoftware.org/ishelp/topic_setup_appid.htm),
[Tauri Windows-Installer](https://v2.tauri.app/distribute/windows-installer/).

## Projektdateien ab 0.18.0

Der Installer registriert im Benutzerbereich `.localstudio` und `LocalStudio.Project` mit einem gequoteten Programm-/Dateipfad. Eine vorhandene andere Standardzuordnung wird nicht überschrieben; Windows kann dann beim ersten Öffnen eine App-Auswahl verlangen. **Öffnen mit → Local Studio** bleibt verfügbar. Die Deinstallation entfernt die eigene Programmzuordnung und den Standardwert nur, wenn dieser weiterhin auf Local Studio zeigt. Daten und Projektdateien bleiben erhalten. Portable Pakete registrieren keine Dateizuordnung.

Die Installerprüfungen verwenden eigene `LocalStudio.Test.…`-ProgIDs und `.localstudio-test-…`-Endungen; sie ändern weder die echte Projektzuordnung noch bestehende Nutzerinstallationen.
