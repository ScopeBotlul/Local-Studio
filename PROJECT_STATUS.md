# Aktueller Stand 0.4.1: Modellgrößen in GB

- Suchkarten laden die Gesamtgröße der Dateien der aufgelösten Revision im Hintergrund nach (drei Abrufe pro aktiver Warteschlange). Beschriftung **Repository gesamt**; umfasst auch alternative Varianten. Keine Schätzung aus Parameterzahlen oder HF-Speicherverbrauch einschließlich Historie.
- Details zeigen dieselbe Gesamtgröße. Dateiauswahl, Downloadvorschau und lokale Dateiliste zeigen die Größe ihrer konkreten Dateien in dezimalen GB (1 GB = 1.000.000.000 Bytes); Dateizeilen behalten zusätzlich die feinere bisherige Einheit.
- Fehlende/unvollständige Größen oder fehlgeschlagene Abrufe erscheinen als unbekannt. Kleine positive Dateien erscheinen als **< 0,01 GB**, niemals irreführend als null. Leere Auswahl bleibt null. Lokale Größe bezeichnet die gespeicherte Dateiauswahl laut Download-Metadaten, keine aktuelle Dateisystem-Belegungsmessung.
- Größenabruf verwendet vorhandene HF-Host-/Token-/Antwortgrößengrenzen. Keine Model Card und keine Gewichtsdatei wird dafür geladen. Queued Requests werden bei Ansicht-/Suchwechsel verworfen; bereits laufende Antworten ändern die neue Ansicht nicht. Neuer Command nur für lokale Hauptansicht freigegeben.

[Portable Update](releases/Local-Studio-0.4.1-hub-portable.zip), [Installer](releases/Local-Studio-0.4.1-hub-setup.exe). Programm schließen, drei Programmdateien ersetzen, **Local-Studio-Data behalten**. M1 und Inferenz bleiben im unten dokumentierten Umfang offen.

Prüfergebnisse separat in TEST_MATRIX.md. Originalprompt/SPEC unverändert.

---

# Aktueller Stand 0.4.0: Downloads und lokale Dateiliste

Stand: 17. September 2026. Der Nutzer bestätigt den integrierten App-Login aus 0.3.1 ausdrücklich mit „okay login klappt“. Diese Bestätigung gilt dem echten App-Login; Refresh, Kontowechsel und gesonderte Website-Sitzungspersistenz bleiben eigene Prüffelder.

## Implementiertes Arbeitspaket

- **Modelle → Details**: ausdrückliche Dateiauswahl ohne Vorauswahl, native Downloadvorschau mit exaktem Commit, Größe, konservativem zusätzlichem Speicherbedarf, Lizenz/Zugang und Ziel. Erst **Auswahl herunterladen** startet die Übertragung.
- **Downloads**: echte Bytes/Fortschritt/Geschwindigkeit/Restmenge/abschätzbare Restzeit, Queue, Pause, unterstützte Range-Fortsetzung, Abbruch mit behaltenen Teil-Dateien, Retry, bestätigter Neustart ab Byte 0, Priorität und pausierte Wiederherstellung nach App-Neustart. Beim App-Schließen wird auf laufende Downloads hingewiesen und sauber pausiert.
- **Modelle → Lokal gespeichert**: heruntergeladene Dateiauswahlen mit Quelle, Lizenz/Aufgabe, Commit, Dateien, Ziel und Prüfsummen. Erneute lokale Prüfung erkennt Korruption oder fehlende Dateien. Die Liste behauptet keine Runtime-Vollständigkeit oder Ausführbarkeit.
- Hashprüfung gegen HF-Quellhashes, separate geprüfte Zielkopie und abschließender Rename. Keine fremden Modellskripte/Abhängigkeiten ausgeführt. Tokens nur für den exakten HF-Host, keine Weitergabe an CDNs; keine Tokens/signierten URLs in SQLite. Teil-Dateien bleiben bei Fehler/Abbruch erhalten.

## Verifikation

- Frontend-/Tauri-/NSIS-Build, **34 Rusttests**, **8 Frontendtests** bestanden.
- [Native Prüfung der paketierten EXE](.artifacts/native-1789610509905/report.json): **34/34 bestanden**. Alle bisherigen Core-/HF-/Browser-/OAuth-Prüfungen plus fünf Downloadprüfungen.
- Echte HF-Dateien aus `hf-internal-testing/tiny-random-gpt2`, Commit `71034c5d8bde858ff824298bdedc65515b97d2b9`: `config.json` (807 Bytes, Git-Blob-Prüfung) und `model.safetensors` (453.864 Bytes, SHA-256 `8111d5afb0715dbf5a31396d31432cb56370ba23f6650a035ea0fc8a20b4e500`). Nur Byte-/Downloadtest; kein Inferenzmodell integriert oder ausgeführt.
- Native UI-Vorschau ohne automatische Auswahl, geprüfte Übernahme, lokale Dateiliste, Erkennung einer gleich großen beschädigten Testkopie, Wiederherstellung/erneute Prüfung, ungültige Pfade/verbrauchte Plan-IDs/Duplikate und Persistenz nach Neustart bestanden. Der Website-WebView kann auch `download_list` nicht aufrufen; ACL-Ablehnung nativ bestätigt.
- Kontrollierte HTTP-Tests: Pause mit stabiler Teil-Datei, echte Range-Fortsetzung nach Wiederöffnung, nicht unterstützte Fortsetzung ohne Teilverlust, bestätigter Neustart, Cancel, Hashfehler ohne Veröffentlichung, Recovery und Queue-Priorität. Diese Tests sind vom echten öffentlichen HF-Download klar getrennt.
- Download-Vorschau und lokale Dateiliste anhand nativer Screenshots visuell geprüft; keine Überlappung in der geprüften Größe. Keine vollständige Accessibility-/Fenstergrößenabnahme.
- Ein früher nativer Lauf fand das öffentliche interne HF-Testrepo nicht über den Suchindex; der Test verwendet jetzt die vorhandene Website-zu-Modell-Übergabe. Der fehlgeschlagene Suchlauf wird nicht als bestandener Downloadtest gewertet.
- ZIP-CRC, genau drei Programmdateien, UTF-8-Anleitung, Identität mit der getesteten EXE und Paketprüfsummen bestätigt. Keine Modelle/Secrets/Testsitzungen mitgeliefert. Installer gebaut; Installation/Deinstallation nicht geprüft. Kein vollständiger Rust-Abhängigkeitsaudit.

## Pakete und nächste Grenze

[Portable ZIP](releases/Local-Studio-0.4.0-hub-portable.zip), [EXE](<releases/Local Studio Hub 0.4.0/Local Studio.exe>), [Installer](releases/Local-Studio-0.4.0-hub-setup.exe), [SHA-256](releases/SHA256SUMS-0.4.0.txt). App zum Update schließen, Programmdateien im bisherigen Ordner ersetzen und **Local-Studio-Data behalten**. Bestehende Nutzermodelle und Nutzeranmeldung wurden nicht verändert.

M1 ist weiter offen: PC-Suche/Import, vollständige Runtime-/Komponenten-Erkennung, Modellreparatur, Entfernen/Verschieben/Updates, weitere Quellen und private/gated Downloadfälle benötigen eigene Umsetzung/Tests. Downloads laufen aktuell einzeln; die Dateiliste ist keine vollständige Modellverwaltung. Inferenz und alle späteren M0–M12-Anforderungen bleiben erhalten. Details: [DOWNLOADS.md](docs/DOWNLOADS.md). SPEC und Originalprompt sind unverändert und bytegleich.

---

## Historischer Stand bis 0.3.1

# Aktueller Stand 0.3.1: App-Login im integrierten Browser

Auf ausdrücklichen Nutzerwunsch öffnet **Mit Hugging Face anmelden** jetzt die echte OAuth-Anmeldeseite im integrierten Browser von Local Studio. Ein noch nicht vorhandener WebView startet direkt mit dem Loginziel; ein vorhandener WebView wird dafür wiederverwendet. Abschluss und Abbruch führen zur Kontoansicht zurück. Während der Anmeldung nennt die Oberfläche ausdrücklich die Verbindung des App-Kontos.

PKCE, zufälliger State, temporärer Loopback-Listener, Windows Credential Manager und die strikte lokale WebView-Capability bleiben erhalten. Nur der exakte aktive Callback-Endpunkt darf während der Anmeldung zusätzlich navigiert werden. Abschluss/Abbruch entfernt diese Ausnahme; Website-Cookies und App-Tokens bleiben technisch getrennt. Keine Anmeldedaten werden aus der Website gelesen.

**Nachweise:** Frontend-/Tauri-/NSIS-Build, 8 Frontendtests, 26 Rusttests und [29 native Prüfungen](.artifacts/native-1789609392616/report.json) bestanden. Drei neue native Prüfungen belegen den Button-Wechsel zur echten HF-Passwortseite mit PKCE und verweigerter privilegierter IPC, die Ablehnung eines gefälschten States sowie eine gültige Ablehnungsantwort über den echten WebView-Callback, erneuten Versuch mit frischem State und Abbruch mit geschlossenem Port. Der native Screenshot der HF-Anmeldeseite wurde visuell geprüft. Keine erfolgreiche Kontoanmeldung simuliert.

Die paketierte EXE ist bytegleich mit der geprüften Kopie; ZIP-CRC/Inhalt, Anleitung und Prüfsummen stimmen. [Portable ZIP](releases/Local-Studio-0.3.1-hub-portable.zip), [EXE](<releases/Local Studio Hub 0.3.1/Local Studio.exe>), [Installer](releases/Local-Studio-0.3.1-hub-setup.exe), [SHA-256](releases/SHA256SUMS-0.3.1.txt). Für das Update App schließen, Programmdateien im bisherigen Ordner ersetzen und **Local-Studio-Data behalten**.

**Grenzen:** Den vollständigen internen Login samt echter Zustimmung/Tokenaustausch muss der Nutzer noch abschließen; seine vorherige Bestätigung galt dem externen Browserpfad. Refresh, Drittanbieter-SSO, echte Kontowechsel und Installer-Lebenszyklus sind weiterhin ungeprüft. Der gewünschte eingebettete User-Agent weicht von RFC 8252 §8.12 ab; Sicherheitsabwägung siehe [HUGGING_FACE.md](docs/HUGGING_FACE.md). Die Original-SPEC bleibt unverändert; die neue Nutzeranweisung ist hier als Verhaltensänderung festgehalten. Downloadmanager, lokale Modellverwaltung und Inferenz bleiben offen.

---

## Historischer Stand bis 0.3.0

# Projektstatus

Stand: 17. September 2026, **0.3.0 Core + Hugging Face mit integriertem Website-Browser**. M0 und M1 bleiben teilweise offen; keine KI-Inferenz und keine Modelldownloads. SPEC.md und Originalprompt bleiben bytegleich.

## Neues Ergebnis und Kontobestätigung

- Der Nutzer hat den erfolgreichen App-Login und die Konto-Prüfung bestätigt. Das ist ein Nutzerbericht, keine automatisierte Abnahme von Refresh, Kontowechsel oder privaten/gated Repositories.
- **Hugging Face → Website im Studio** zeigt die echte Website in einem nativen Kind-WebView. Eigener persistenter Profilordner, Navigation/Zurück/Vorwärts/Neuladen, externe HTTPS-Links im Standardbrowser, Übergabe erkannter Modellseiten an die bestehende Modellansicht.
- App-OAuth und Website-Login sind getrennt. Keine Token-/Cookie-Übertragung. Die Website hat keine privilegierten App-Capabilities. Website-Dateidownloads bleiben bis zum Downloadmanager deaktiviert.

## Verifikation 0.3.0

- Frontend-/Tauri-/NSIS-Build erfolgreich; **25 Rusttests und 8 Frontendtests bestanden**.
- [Native Prüfung der paketierten EXE](.artifacts/native-1789608618032/report.json): **26/26 bestanden**. 15 Core-Prüfungen, sechs reale öffentliche HF-/Kontostatus-Prüfungen, fünf Browser-Prüfungen. Isolierte Profile; keine Nutzermodelle oder Medien verwendet.
- Browser-Nachweise: echte HF-Website gerendert, vier privilegierte IPC-Aufrufe aus der Website tatsächlich durch ACL abgewiesen, externe Adresse in der Adressleiste abgewiesen, echte Modell-Details aus Website-Übergabe, Verbergen/Wiederöffnen und veraltete Owner-Bereinigung geprüft, Bounds bei UI-Zoom passend, synthetischer Cookie nach Neustart vorhanden bei weiterhin getrenntem App-Kontostatus.
- Hauptansicht und Website wurden im vorangegangenen Browserlauf separat visuell geprüft; CDP bildet native Kind-WebViews nicht im Screenshot der Hauptansicht ab. Keine vollständige Accessibility-/Fenstergrößenabnahme.
- Ein früher Gesamtlauf scheiterte an einer Testannahme: Der bereits angemeldete Standardbrowser schloss OAuth schneller ab als die Portprüfung. Die ausschließlich im isolierten Testprofil entstandene Anmeldung wurde wieder gelöscht. Der Systembrowser-Login-Test ist jetzt separat opt-in; der finale öffentliche API-Lauf verwendet ihn nicht. Historische Callback-/Abbruchnachweise aus 0.2.1 bleiben dokumentiert.
- Paket-CRC, genau drei Programmdateien, UTF-8-Anleitung, EXE-Identität und SHA-256 geprüft. Keine Profile/Secrets im ZIP. Installer gebaut, Installation/Deinstallation weiterhin ungeprüft; kein vollständiger Rust-Abhängigkeitsaudit.

## Aktuelle Pakete

[Portable ZIP](releases/Local-Studio-0.3.0-hub-portable.zip), [Direktstart](<releases/Local Studio Hub 0.3.0/Local Studio.exe>), [Windows-Setup](releases/Local-Studio-0.3.0-hub-setup.exe), [Prüfsummen](releases/SHA256SUMS-0.3.0.txt).

Für ein Update App schließen, die drei Programmdateien im bisherigen Ordner ersetzen und **Local-Studio-Data behalten**. Die bestehende Nutzerversion und deren Anmeldung wurden nicht verändert.

## Weiter offen

Echte Website-Anmeldung und deren Neustartpersistenz wurden nicht mit einem Konto getestet; der Cookie-Test belegt nur die Persistenzmechanik. Refresh, echter Kontowechsel, private/gated Zugriffe und erfolgreicher manueller Token-Login bleiben gesonderte Tests. Nächstes M1-Arbeitspaket: Downloadmanager und lokale Modellverwaltung. Alle weiteren Anforderungen aus M0–M12 bleiben in PLAN.md erhalten.

---

## Historischer Stand 0.2.1


Stand: 17. September 2026, **0.2.1 Core + Hugging Face**. M0 bleibt teilweise offen; der erste M1-Pfad für Suche, Modell-Details und Kontoanbindung ist implementiert. M1 ist insgesamt nicht abgenommen. SPEC.md und Originalprompt sind unverändert und bytegleich.

## Aktuelles Ergebnis 0.2.1

- Unter **Modelle**: echte Hugging-Face-Suche, Aufgaben-/Sortierfilter, Pagination, Repo-Metadaten, Lizenz-/Zugangshinweise, exakte Commit-Auflösung, Dateiliste mit bekannten Größen/Hashes und Model Card als sicherer Originaltext.
- Unter **Hugging Face**: Kontoanzeige, erweiterter Token-Anmeldepfad mit echter Online-Validierung, Windows-Anmeldespeicher, lokale Abmeldung und erneute Konto-Prüfung. Der native Testspeicher ist vom Nutzerprofil getrennt.
- OAuth-Codepfad mit Public Client, PKCE, zufälligem State, temporärem Loopback-Callback, Abbruch, Timeout und Refresh ist implementiert. **Browserlogin ist aktiviert.** Nach ausdrücklichem „ja“ wurde Local Studio einmalig bei Hugging Face registriert: Client-ID `322a729f-1375-4456-8b0e-d3dbea8b8d31`, Scope `openid profile read-repos`, Authentifizierung `none`, kein Client-Geheimnis. [Registrierungsdaten](docs/huggingface-oauth-client.json). Die zuvor abgelehnte Registrierung ist damit abgeschlossen.
- Keine Modelle heruntergeladen, installiert oder ausgeführt. Der Ausführbarkeitsfilter zeigt korrekt keine kompatiblen Modelle. Website-Links öffnen den Standardbrowser; der geforderte eingebettete Website-WebView bleibt offen.

## Nachweise 0.2.1

- Frontend-/Tauri-/NSIS-Build erfolgreich; **21 Rusttests**, **8 Frontendtests** bestanden.
- [Native Release-Prüfung](.artifacts/native-1789607515635/report.json): **22/22 bestanden**, an einer isolierten Kopie der tatsächlich paketierten EXE. Alle bisherigen 15 Core-Prüfungen plus sieben HF-Prüfungen.
- Echte HF-API: 40 unterschiedliche Suchtreffer über zwei Seiten, Aufgabenfilter für Spracherkennung, Modell `openai-community/gpt2` auf Commit `607a30d783dfa663caf39e06633721c8d4cfcd7e`, MIT-Lizenz, bekannte Dateigrößen und Model Card. Fehlende Revision und falscher Token abgewiesen. Keine Testtokens in SQLite oder Logs. Kein erfolgreicher Login simuliert.
- Windows Credential Manager: reale Speicherung/Wiederöffnung/Löschung eines synthetischen Credentials in separatem Testprofil. Lokaler HTTP-Callback nimmt passende Antworten an und weist gefälschte States/Hosts, Duplikate und falsche Methoden ab. Test für späten Login nach Abbruch, HTTPS-Redirect-Verweigerung, Antwortgrenzen und geheimnisfreie Fehler bestanden.
- Screenshots 05/06/07 des vorangegangenen 0.2.0-Laufs wurden visuell geprüft; das Layout blieb in 0.2.1 unverändert: Konto-/Such-/Dateiansicht lesbar, keine Überlappung in der geprüften Fenstergröße. Noch keine vollständige Accessibility-/Layoutabnahme.
- In der 0.2.0-Entwicklung scheiterte ein früher Lauf am Testvergleich von Playwright-Fehlern; ein weiterer fand beim Pfadtest eine andere geöffnete Ansicht vor. Testvergleich und explizite Navigation wurden korrigiert; diese Läufe gelten nicht als bestandene Produktprüfungen. Ein beim Bearbeiten entstandener UTF-8-Fehler wurde vor dem finalen Build behoben; der originale Hash-Referenzwert ist wieder bestätigt.
- `npm audit`: keine bekannten JavaScript-Schwachstellen. Vollständiger Rust-Abhängigkeitsaudit und Installer-Installation/-Deinstallation bleiben offen.

## Pakete 0.2.1

- [Portable ZIP](releases/Local-Studio-0.2.1-hub-portable.zip): 5.165.626 Bytes, SHA-256 `3187bc96b281ca4b15790758f245096695cd975715df265298f8e81eb25fe9b9`.
- [Windows-Setup](releases/Local-Studio-0.2.1-hub-setup.exe): 3.908.458 Bytes, SHA-256 `582f371370970ae828d7bece359e3a435fe69a7d579eb37e1bbb6e8a6a0ff1f0`.
- [Direktstart](<releases/Local Studio Hub 0.2.1/Local Studio.exe>) und [Prüfsummendatei](releases/SHA256SUMS-0.2.1.txt). ZIP-CRC, genau drei Programmdateien, UTF-8-Anleitung und EXE-Identität geprüft. Keine Profile oder Credentials im Paket.
- Update: bisherige App schließen, drei Programmdateien im bisherigen Ordner ersetzen, **Local-Studio-Data behalten**. Laufende Nutzerversionen wurden nicht beendet oder überschrieben.

## Nächste Schritte und Grenzen

Die Registrierung und das neue Paket sind abgeschlossen. Als nächsten Kontotest in 0.2.1 unter Hugging Face auf „Mit Hugging Face anmelden“ klicken und den Login/Zustimmung im Standardbrowser selbst abschließen. Der automatisierte Testlogin wurde abgebrochen und verwendet ein separates Profil. Positive Token-/OAuth-Anmeldung, echter Kontowechsel, Token-Erneuerung und gated/private Zugriffe sind noch nicht Ende-zu-Ende bestätigt. Keine Zugangsdaten im Chat anfordern.

Der native Test belegt den erfolgreichen Windows-Browseraufruf, einen während des Logins aktiven Loopback-Port, die Ablehnung eines gefälschten Callbacks und das Schließen des Ports bei Abbruch. Ein [anonymer Provider-Check](.artifacts/oauth-provider-0.2.1.json) belegt HTTP 302 zur HF-Loginseite und dort HTTP 200. Die Browsersteuerung war technisch nicht erreichbar; eine visuelle Prüfung der externen Seite wird daher nicht behauptet.

Danach Downloadmanager und lokale Modellverwaltung gemäß M1. Einzelpfade, Shortcut-Konfiguration und übrige offene M0-Anforderungen bleiben erhalten. Das komplette M0–M12-Ziel bleibt in PLAN.md und SPEC.md bestehen. Details der neuen Grenze: [HUGGING_FACE.md](docs/HUGGING_FACE.md).

---

# Historischer Stand 0.1.1 (nachfolgend unveränderte frühere Nachweise)

Stand: 17. September 2026, Core 0.1.1. Der erste M0-Core-Pfad ist implementiert und auf Windows nativ geprüft. M0 bleibt insgesamt teilweise offen; M1–M12 sind nicht abgenommen. Die vollständige Spezifikation wurde unverändert aus `prompt` nach `SPEC.md` übernommen; SHA-256 beider Dateien: `2e3232ef5a607123326d3732679e28a5758106a5d8db61a7420b2e47b71d4bdd`.

## Ergebnis dieses Arbeitspakets

Eine echte Windows-Anwendung mit Tauri 2, React/TypeScript, Rust und SQLite startet und bietet Einrichtung, Einstellungen, Hardwareinventar und Datei-Hashjobs. Der Hashjob läuft in einem eigenen Prozess, meldet tatsächlichen Fortschritt und unterstützt Abbruch. Einstellungen und Jobs bleiben über Neustarts erhalten; ein harter Abbruch wird als Recovery-Fall erkannt. Es wurden keine KI-Modelle heruntergeladen, integriert oder als kompatibel getestet.

| Bereich | Geplant | Implementierung | Prüfung |
| --- | --- | --- | --- |
| Vollständige SPEC und Meilensteine | Ja | Vorhanden | SPEC bytegleich mit Quelle per SHA-256 geprüft |
| Typisierter IPC-Vertrag | Ja | Rust und `src/types.ts` | Native Einrichtung, Settings und Jobbefehle im E2E geprüft |
| Windows-App und lokale Oberfläche | Ja | Implementierter Core-Pfad | Frontend-/Tauri-/NSIS-Build 0.1.1; 15/15 native Release-Prüfungen bestanden |
| SQLite-Einstellungen, Sprache, Theme, Akzent, Skalierung | Ja | Implementiert | DE/EN/Theme, Zoom/Reset, ungültiges Speichern und Neustartpersistenz nativ bestanden; neue Regression für konkurrierendes Speichern/Zoom und Schließen während Speicherung/Pfadaktualisierung bestanden |
| Datenstamm und abgeleitete Unterordner | Ja | Implementiert; Einzelpfade noch offen | Rusttests für Speicherbaum/Reparatur; native Einrichtung und Stammwechsel bei erhaltenem Originalordner/stabiler DB geprüft |
| CPU/RAM/GPU/VRAM/Laufwerksinventar | Ja | Implementiert | Windows 11, Ryzen 9 9950X3D und RTX 4080 nativ erkannt; unbekannte VRAM-Werte als unbekannt |
| Hintergrund-Hashjob, Status, Abbruch, Fehler | Ja | Eigener Prozess, Queue, Abbruch, Timeout, Fehler; alle aktiven Jobs bleiben sichtbar | Echter SHA-256 gegen Referenz; fehlende Datei abgewiesen; wartender und laufender Job abgebrochen; Ergebnis in UI; zwei neue Rusttests für Listenbegrenzung und Sortierung |
| Sessionmarker, unterbrochene Jobs, Logs | Ja | Implementiert | Sauberer Neustart und Crash-Recovery geprüft; keine vorgetäuschte Fortsetzung |
| Vollständige M0-Anforderungen | Ja | Teilumfang offen | Nicht vollständig abgenommen |
| Modelle, Downloadmanager und Hugging-Face-Login | Ja | Nicht implementiert | Nicht getestet |
| Bild/Video/Audio, Galerie/Projekte, Assistent/Analyse, Workflows | Ja | Nicht implementiert | Nicht getestet |
| 18+-Sperre und App-Updates | Ja | Nicht implementiert | Nicht getestet |
| Release-Installer/portable Ausgabe | Ja | Release-EXE und NSIS-Installer gebaut; portable Core-Verpackung | Release-EXE mit portable Marker nativ geprüft; ZIP-Inhalt/CRC/EXE-Identität geprüft; Installer nicht installiert/deinstalliert |

## Korrekturen und Nachweise für 0.1.1

Version 0.1.1 behebt zwei konkrete Zuverlässigkeitsfehler: Ältere aktive Aufträge bleiben auch bei mehr als 250 Einträgen sichtbar; nur abgeschlossene Historie wird auf 250 begrenzt. Während manuellem Speichern werden weitere Einstellungsänderungen und Zoom gesperrt, damit kein veralteter Stand gerade gespeicherte Einstellungen überschreibt. Das Schließen wartet außerdem auf die Speicherung und die Aktualisierung der Speicherpfade. Der Button heißt nun „Prüfsumme berechnen“; er berechnet SHA-256 und ist kein Virenscan oder Modell-Sicherheitscheck.

- `npm run build` und Tauri-/NSIS-Build 0.1.1: erfolgreich.
- `npm test`: 8/8 Frontendtests bestanden.
- Rusttests über `scripts/desktop.ps1 test` / `cargo test --manifest-path src-tauri/Cargo.toml`: 12/12 bestanden. Zwei neue Datenbanktests prüfen unter anderem 301 aktive plus 300 abgeschlossene Jobs, erhaltene aktive IDs/Abbruchfähigkeit, die Begrenzung der Historie und stabile Sortierung.
- [Release-E2E 0.1.1](.artifacts/native-1789605849765/report.json): 15/15 bestanden. Die paketierte EXE wurde als Kopie mit portable Marker, eigener Datenbank und eigenem WebView-Profil geprüft. Neu sind verzögertes Speichern mit konkurrierendem Zoom sowie Schließen während echter Speicherung und verzögerter Pfadaktualisierung, einschließlich Neustartpersistenz.
- [Reproduktion am alten 0.1.0-Release](.artifacts/native-1789605675623/report.json): Der gezielt verzögerte echte IPC-Transport reproduziert den Einstellungsverlust (`dark` statt gespeichertem `light`). Backenddaten werden nicht simuliert. Ein vorausgehender Testharness-Timeout zählt nicht als Produktfehler.
- Das portable ZIP enthält genau drei Programmdateien mit gültiger CRC, bytegleicher getesteter EXE und UTF-8-Anleitung für 0.1.1. Beide Paketprüfsummen stimmen mit `SHA256SUMS-0.1.1.txt` überein.
- Keine unbehandelten Frontend-Laufzeitfehler im nativen Test. Screenshot 04 des neuen Release-Laufs wurde visuell geprüft: die Beschriftung „Calculate checksum“ ist ohne Überlappung lesbar. Eine vollständige UI-/Accessibility-Abnahme bleibt offen.
- Der NSIS-Installer ist gebaut; Installation und Deinstallation sind weiterhin ungetestet.

Die [begrenzte Sicherheitsprüfung](docs/SECURITY_REVIEW.md) hält Umfang und Grenzen fest. Im geprüften Core wurde kein hoher oder kritischer Sicherheitsfehler bestätigt. `npm audit` meldete am 17. September 2026 keine bekannten JavaScript-Abhängigkeitsschwachstellen. Ein vollständiger Rust-Abhängigkeitsaudit wurde nicht durchgeführt. Hugging-Face-Login und KI-Funktionen sind weiterhin nicht implementiert.

## Historische Nachweise für 0.1.0

- `npm run build`: erfolgreich; Produktions-Frontend gebaut.
- `npm test`: 8 Frontendtests bestanden.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 10 Rusttests bestanden, einschließlich Single-Instance-Sperre und Reparatur eines nicht verfügbaren Datenstamms.
- [Debug-E2E](.artifacts/native-1789604766537/report.json): 11/11 erfolgreich. [Release-E2E](.artifacts/native-1789604938456/report.json): 13/13 erfolgreich am 17. September 2026; zusätzlich Tastaturzoom/Reset und Datenstammwechsel mit erhaltenem Originalverzeichnis und stabiler Datenbank. Die tatsächliche Release-EXE lief als Kopie in einem isolierten Ordner mit `portable.marker`. Beide Berichte enthalten Hardware und den tatsächlich berechneten Referenzhash.
- Tauri-Release-Build und NSIS-Erstellung erfolgreich: `src-tauri/target/release/local-studio.exe` und `src-tauri/target/release/bundle/nsis/Local Studio_0.1.0_x64-setup.exe`. Der Installer wurde nicht installiert oder deinstalliert.
- Einrichtung und Einstellungen wurden anhand nativer Screenshots visuell geprüft; kein Überlauf in den geprüften Ansichten. Im E2E keine unbehandelten Frontend-Laufzeitfehler.

Die statische Backendprüfung führte zu Korrekturen: exklusiver Sessionlock vor DB-Öffnung, feste absolute NVIDIA-Probewege, serialisierter Jobstart/Abbruch, Dateiprüfung außerhalb des Worker-Locks, asynchrone native I/O-Commands, Worker-Bereinigung bei Fehlern und reparierbarer Start trotz fehlendem Datenstamm.

## Core 0.1.1 zum Ausprobieren

- [Portable ZIP](releases/Local-Studio-0.1.1-core-portable.zip), 3.784.957 Bytes: enthält ausschließlich `Local Studio.exe`, `portable.marker` und `LIESMICH.txt`. CRC-Prüfung erfolgreich; eingebettete EXE bytegleich mit der getesteten Release-EXE; UTF-8-Anleitung geprüft.
- [NSIS-Setup](releases/Local-Studio-0.1.1-core-setup.exe), 2.864.161 Bytes: erfolgreich gebaut; Installation und Deinstallation wurden nicht durchgeführt.
- [SHA-256-Prüfsummen](releases/SHA256SUMS-0.1.1.txt) für beide Pakete. Entpackter Direktstart: [Local Studio.exe](<releases/Local Studio Core 0.1.1/Local Studio.exe>).

Die vorhandene laufende 0.1.0-Anwendung und ihre Daten wurden nicht verändert. Zum Aktualisieren einer bestehenden portablen Kopie die Anwendung schließen und die drei Dateien aus dem ZIP im bisherigen Programmordner ersetzen; den Ordner `Local-Studio-Data` erhalten. Der separate Direktstartordner von 0.1.1 verwendet eine eigene portable Konfiguration.

Dies sind frühe Core-Builds ohne KI-Modelle. Die portable Anwendung benötigt Windows x64 mit WebView2 und einen beschreibbaren entpackten Ordner.

## Offene Grenzen und nächster Schritt

Der IPC-Vertrag umfasst noch keine Modelladapter, Inferenz, Downloadjobs, Projektcontainer oder Medieneditoren. Der Datei-Hashjob beweist diese Fähigkeiten nicht. `StoragePaths` beschreibt abgeleitete Verzeichnisse; freie Einzelpfadkonfiguration, frei belegbare Shortcuts mit Konflikterkennung sowie vollständige Einrichtung mit Hugging Face, Modellscan und Assistentenmodell bleiben offen. Undo- und Aufbewahrungswerte sind Einstellungen für spätere Funktionen; tatsächliche Medien-Undo-/Bereinigungsabläufe fehlen noch.

Die Recovery-Grundlage erkennt unterbrochene Jobs. Wiederherstellung von Ebenen, Medien, Prompts oder Timelinezuständen, periodische Recovery-Punkte und deren Schutzregeln sind noch nicht implementiert. Große Modelljobs, Offline-Inferenz, vollständige Ressourcenverwaltung, Accessibility-Abnahme und M12-Distribution sind nicht durch die Core-Tests abgedeckt.

Als Nächstes stehen die noch offenen M0-Punkte und anschließend M1 gemäß `PLAN.md` an. Die Core-Pakete sind erstellt und das portable Archiv ist geprüft. Installer-Installation/-Deinstallation, signierte Updates und vollständige M12-Abnahme bleiben offen. Es liegt derzeit keine dokumentierte Nutzerentscheidung als Blockade vor.
