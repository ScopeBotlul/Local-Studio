# Local Studio

Version 0.4.1 zeigt Modellgrößen in GB: Repository gesamt in Suchkarten/Details, konkrete Dateiauswahl bei Download und lokalen Modellen.

Eine lokale Windows-Kreativanwendung, aufgebaut in den Meilensteinen M0–M12.
Die vollständige Anforderung steht in [SPEC.md](SPEC.md); der aktuelle, getrennt
nach Implementierung und Prüfung geführte Stand in [PROJECT_STATUS.md](PROJECT_STATUS.md).

Der erste Entwicklungsstand umfasst den Desktop-Unterbau: lokale Einstellungen,
Hardwareerkennung, SQLite, Auftragsverwaltung und einen isolierten Prozess für
SHA-256-Dateiprüfungen. Version 0.2.0 ergänzt echte Hugging-Face-Suche, Modell-Details und
einen Token-Anmeldepfad mit Windows-Anmeldespeicher. Ab 0.2.1 ist auch OAuth mit genehmigter, registrierter öffentlicher Client-ID aktiviert. Version 0.4.0 ergänzt bewusste Dateiauswahl, Downloadmanager und eine lokale Dateiliste.
KI-Inferenz und vollständige Runtime-/Modellverwaltung fehlen weiterhin.
Eine Prüfsumme ist kein Nachweis, dass eine Modelldatei sicher oder kompatibel ist.

## Aktuellen Stand 0.4.1 starten

Die lokal gebauten Pakete liegen unter `releases/`: Portable ZIP, EXE und Windows-Installer.
Build-Dateien und lokale Testberichte sind nicht im Quellcode-Repository enthalten.
Zum eigenen Bauen siehe die Entwicklungsschritte unten.
34 native Prüfungen, 35 Rusttests und 10 Frontendtests bestanden; Details siehe
[TEST_MATRIX.md](TEST_MATRIX.md).
Installation/Deinstallation des Installers wurden nicht getestet.

Unter **Modelle** eine Suche starten; die App zeigt echte Hugging-Face-Ergebnisse,
weitere Seiten, Aufgabenfilter, Lizenzen, Dateilisten und exakte Revisionen.
**Dateien auswählen → Download prüfen → Auswahl herunterladen** startet einen
bewussten Download. Unter **Downloads** pausieren/fortsetzen/abbrechen; unter
**Modelle → Lokal gespeichert** Dateien erneut prüfen. Es wird noch kein Modell ausgeführt.
Unter **Hugging Face** liegt die Kontoansicht. Die erweiterte Token-Anmeldung
ist implementiert; ein ungültiger Token wird online abgewiesen. Der Nutzer hat sowohl den früheren App-Login als auch den integrierten Login in 0.3.1 bestätigt. Für den normalen Login
**Mit Hugging Face anmelden** anklicken: Ab 0.3.1 öffnet dieser Button den
integrierten Browser. Nach Abschluss oder Abbruch kehrt die App zur Kontoansicht zurück.
Die App dabei geöffnet lassen; nach drei Minuten gegebenenfalls erneut starten.

Neu in 0.3.0: **Hugging Face → Website im Studio** zeigt die echte Website
in einem eigenen WebView. Die Website-Anmeldung ist vom App-Konto getrennt.
Das Browserprofil bleibt erhalten; ein synthetischer Cookie überstand im nativen
Test den Neustart. Eine echte Website-Anmeldung wurde nicht automatisiert getestet.
**In Local Studio öffnen** übernimmt erkannte Modellseiten in die Modellansicht.
Direkte Website-Dateidownloads bleiben blockiert. Nutze **In Local Studio öffnen**
und anschließend die geprüfte Dateiauswahl. [Downloadumfang und Grenzen](docs/DOWNLOADS.md).

Details, Sicherheitsgrenzen und die erfolgte Registrierung stehen in
[HUGGING_FACE.md](docs/HUGGING_FACE.md). Zugangsdaten niemals in den Chat senden;
eigene Tokens ausschließlich im vorgesehenen Passwortfeld der App eingeben.

Für das portable Update die bisherige App schließen und die drei Dateien aus
dem ZIP im bisherigen Programmordner ersetzen. **Local-Studio-Data behalten.**
Die neue separate Testkopie hat ein eigenes Profil. Zugangsdaten liegen im
Windows-Anmeldespeicher und sind nicht Bestandteil des ZIPs.
Die lokalen Paket-Prüfsummen stehen in `releases/SHA256SUMS-0.4.1.txt`.

## Vorheriger Core 0.1.1


Die geprüfte portable EXE liegt unter
[`releases/Local Studio Core 0.1.1/Local Studio.exe`](releases/Local%20Studio%20Core%200.1.1/Local%20Studio.exe).
Ein Doppelklick startet die Ersteinrichtung. Daneben liegen [das portable ZIP](releases/Local-Studio-0.1.1-core-portable.zip) und
[der gebaute Windows-Installer](releases/Local-Studio-0.1.1-core-setup.exe).
Die Installer-Installation und -Deinstallation sind noch nicht getestet.

Der vorherige 0.1.1-Release-Core hat 15 native Integrationsprüfungen bestanden;
zusätzlich bestehen 12 Rust- und 8 Frontend-Tests. Einzelheiten und offene
Anforderungen stehen in der Testmatrix.

Version 0.1.1 behebt das Überschreiben von Einstellungen bei gleichzeitigem Speichern und Zoomen sowie das Verschwinden älterer aktiver Aufträge aus großen Joblisten. Das Fenster wartet beim Schließen auf eine laufende Speicherung. „Datei prüfen“ heißt jetzt „Prüfsumme berechnen“.

Für das Update einer bestehenden portablen Kopie die Anwendung schließen und die drei Dateien aus dem ZIP im bisherigen Programmordner ersetzen. Den Ordner `Local-Studio-Data` erhalten. Die bisherige laufende Anwendung und ihre Daten wurden bei der Bereitstellung nicht verändert. Paketprüfsummen stehen in [SHA256SUMS-0.1.1.txt](releases/SHA256SUMS-0.1.1.txt).

## Entwicklung unter Windows

Voraussetzungen: Node.js, Microsoft C++ Build Tools mit Windows SDK, WebView2 und
eine Rust-MSVC-Toolchain. Die Skripte verwenden eine vorhandene projektlokale
Toolchain unter `.tools`, sonst die installierte Toolchain.

```powershell
npm ci
# Nur falls Rust fehlt: Installation im Projektordner ohne Änderung des System-PATH.
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bootstrap-rust.ps1
npm run desktop:dev
```

`npm run dev` alleine startet nur die Web-Oberfläche. Sie benötigt die Tauri-App
für Hardware, Dateizugriff und Datenbank; es werden keine Ersatzdaten simuliert.

## Prüfungen und Build

```powershell
npm run build
npm test
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/desktop.ps1 test
npm run desktop:build
```

Ein Windows-Installer wird durch Tauri/NSIS erstellt. Das allein bedeutet nicht,
dass der spätere Packaging-Meilenstein mit signierten Updates bereits erfüllt ist.

`scripts/native-smoke.mjs` testet die echte Windows-WebView und Rust-IPC. Es verwendet standardmäßig den Debug-Build; über `LOCAL_STUDIO_TEST_EXE`
kann die Release-EXE gewählt werden. Beide benötigen eingebettete Frontend-Dateien.
Jeder Lauf kopiert die EXE in einen isolierten portablen Ordner unter `.artifacts`
und verwendet eine eigene Datenbank sowie ein eigenes WebView-Profil. Der
Debug-Port wird ausschließlich für den Testprozess aktiviert. Der Test umfasst
Setup, Einstellungen, SHA-256, Abbruch, Neustart und Crash-Erkennung.
Mit `LOCAL_STUDIO_TEST_HF=1` werden zusätzlich reale öffentliche API-Aufrufe
und ein absichtlich ungültiger Testtoken geprüft; keine OAuth-Registrierung.

## Lokale Daten und portable Entwicklung

Normalerweise liegen die Konfiguration und SQLite-Datenbank im Windows-
Benutzerbereich. Der Datenordner für zukünftige Medien und Modelle ist in den
Einstellungen änderbar. Dabei werden vorhandene Dateien nicht verschoben.

Eine Datei namens `portable.marker` neben der ausführbaren Datei aktiviert den
portablen Datenordner `Local-Studio-Data`. Der vollständige portable Release- und
Updateablauf gehört zu M12 und bleibt gesondert zu testen.

Es werden keine Modelle automatisch installiert, keine Medien hochgeladen und
keine Nutzungsdaten übertragen. Benötigte Softwarepakete werden beim Build aus
den offiziellen Paketquellen bezogen.

## Dokumentation

- [Plan](PLAN.md): vollständige Reihenfolge und Abnahmekriterien.
- [Testmatrix](TEST_MATRIX.md): Anforderungen und tatsächliche Prüfergebnisse.
- [Modellkompatibilität](MODEL_COMPATIBILITY.md): ausschließlich nachgewiesene Modellpfade.
- [Begrenzte Sicherheitsprüfung](docs/SECURITY_REVIEW.md): Befunde, Korrekturen und offene Prüfungen.
- [Architektur](docs/architecture.md): Grenzen zwischen Oberfläche, Rust und Workern.

Technische Originalquellen: [Tauri-Voraussetzungen](https://v2.tauri.app/start/prerequisites/),
[Tauri-Commands](https://v2.tauri.app/develop/calling-rust/),
[native Dialoge](https://v2.tauri.app/plugin/dialog/),
[Playwright mit WebView2](https://playwright.dev/docs/webview2).
