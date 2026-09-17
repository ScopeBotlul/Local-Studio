# Begrenzte Sicherheits- und Zuverlässigkeitsprüfung

Stand: 17. September 2026. Geprüft wurden der vorhandene M0-Core und die nachfolgend beschriebene HF-Ergänzung in 0.2.0. Dies ist eine begrenzte Quellcodeprüfung mit den in der [Testmatrix](../TEST_MATRIX.md) dokumentierten Verhaltenstests, kein vollständiges Sicherheitsaudit der geplanten Anwendung.

## Ergänzung 0.4.0: Downloads und lokale Dateiprüfung

Native Pläne validieren Repo, Revision und ausgewählte Dateipfade anhand frischer HF-Metadaten. Die UI übergibt zum Start nur die zuvor erzeugte Plan-ID. Transfers erlauben ausschließlich HTTPS-HF/CDN-Domains; jeder Redirect wird geprüft, Bearer-Tokens werden nur an den exakten API-Host gesendet. Kein freier URL-Download und kein automatisches Ausführen von Modellcode. SQLite enthält keine Secrets oder signierten CDN-Adressen.

Pfad-, Größen-, Hash- und Range-Prüfungen schützen den Downloadpfad. Kopie/erneute Prüfung und abschließender Rename verhindern, dass eine ungeprüfte Auswahl als fertig erscheint. Die lokalen Download-Commands sind für den fremden Website-WebView nicht freigegeben. Die kontrollierten HTTP-Tests decken Pause, Fortsetzung, Ablehnung falscher Integrität, Recovery und Priorität ab. Dies ersetzt keinen Schutz gegen einen bösartigen lokalen Prozess mit denselben Benutzerrechten und keinen vollständigen Abhängigkeitsaudit.

Der Nutzer bestätigt den integrierten App-Login von 0.3.1. Grenzen und Downloadnachweise: [DOWNLOADS.md](DOWNLOADS.md), [Testmatrix](../TEST_MATRIX.md).

## Ergänzung 0.3.1: gewünschter interner App-Login

Der Nutzer fordert ausdrücklich den integrierten Browser auch für den App-OAuth-Button. Dafür wird nur während eines laufenden Loginversuchs dessen exakter Loopback-Endpunkt zugelassen. PKCE, Host-/State-/Methodenprüfung, Credential Manager und die WebView-Capability-Grenze bleiben erhalten. Abbruch und Abschluss entfernen die Ausnahme; veraltete Generationen dürfen neue Versuche nicht überschreiben.

Dies ist eine bewusste Abweichung von RFC 8252 §8.12, das für native OAuth-Apps einen externen User-Agent verlangt. Ein app-eigener WebView besitzt nicht dieselbe Trennung von Anmeldedaten wie der Systembrowser. Die Änderung ist deshalb nicht als RFC-8252-konformer Browserpfad bezeichnet. Es wird weder ein Anbieterverbot umgangen noch eine erfolgreiche echte Anmeldung simuliert. Details und verbleibende Tests stehen in HUGGING_FACE.md.

## Ergänzung 0.3.0: isolierter Website-Browser

Privilegierte Capabilities sind jetzt explizit auf den lokalen WebView `main` begrenzt; ein Fenster-Match würde auch Kind-WebViews einschließen und wird deshalb nicht verwendet. Der echte externe WebView `hf-website` erhielt im nativen Test für vier lokale Commands jeweils `not allowed by ACL`. Browsersteuerung prüft zusätzlich das tatsächliche Caller-Label. OAuth-Tokens werden weder an den Website-WebView geliefert noch in Cookies verwandelt.

Navigation, neue Fenster und Downloads werden kontrolliert. Nur die beiden exakten HTTPS-HF-Hosts bleiben intern; andere zulässige HTTPS-Links gehen über die Windows-URL-Zuordnung. Lokale/IP-Ziele und gefährliche Schemes werden abgewiesen. Bounds/Owner-Validierung schützt gegen ungültige Layouts und verspätete React-Cleanups. 25 Rusttests einschließlich vier Browser-Grenztests bestehen. Cookie-Persistenz wurde mit einem synthetischen Cookie in einem isolierten Profil geprüft, nicht mit einem echten Website-Konto.

Der Nutzer hat den erfolgreichen App-Login samt Konto-Prüfung bestätigt. Das ersetzt keine automatisierte Prüfung von Refresh, Kontowechsel, Website-Login oder gated/private Zugriffen. Kein vollständiges Sicherheitsaudit; bisherige Abhängigkeits- und Installer-Grenzen gelten weiter.

## Historische Ergänzung 0.2.0/0.2.1: Hugging Face

Die neue Netzwerk-/Kontoschicht wurde begrenzt im Quellcode und durch native Tests geprüft. Feste HTTPS-Endpunkte, keine automatischen Weiterleitungen, begrenzte Antwortgrößen/Timeouts, geprüfte Repo-IDs, escaped Model Cards, Windows Credential Manager und keine Secret-Lese-IPC reduzieren die neue Angriffsfläche. Die CSP und lokale Capability-Grenze bleiben erhalten. Offizielle Browserlinks nutzen direkt die Windows-URL-Zuordnung, ohne Shell-Interpreter oder PATH-Programmaufruf.

21 Rusttests und 22 native Release-Prüfungen in 0.2.1 bestehen, darunter Store-/Callback-Tests mit synthetischen Daten, eine real abgewiesene ungültige Anmeldung, Pagination/Modell-Details und Secret-Abwesenheit in SQLite/Logs. `npm audit` meldet weiterhin 0 bekannte JavaScript-Lücken. Das ist kein vollständiger Rust-Abhängigkeits- oder Sicherheitsnachweis.

**Ungeprüft:** positive echte Token-/OAuth-Anmeldung, Refresh, echte Kontowechsel und gated/private Zugriffe. Die zunächst blockierte Registrierung wurde nach ausdrücklicher Nutzerfreigabe einmalig ausgeführt. Version 0.2.1 enthält die öffentliche Client-ID und aktiviert den Browserlogin; Registrierung allein ist kein Nachweis einer erfolgreichen Kontoanmeldung. Die genaue Implementierung und die genehmigte Registrierung stehen in [HUGGING_FACE.md](HUGGING_FACE.md). Der eingebettete externe WebView kam mit 0.3.0 hinzu, siehe oben.

## Historisches Ergebnis und Korrekturen 0.1.1

Im geprüften Core wurde kein kritischer oder hoher Sicherheitsfehler bestätigt. Zwei konkrete Zuverlässigkeitsfehler wurden gefunden und im Quellcode für Version 0.1.1 korrigiert. Der neue Release-Build hat 15/15 native Prüfungen bestanden:

| Befund | Auswirkung | Implementierte Korrektur | Verifikation |
| --- | --- | --- | --- |
| Jobübersicht begrenzt alle Zustände gemeinsam auf die neuesten 250 Einträge | Ältere aktive Aufträge können aus der sichtbaren Liste und aktiven Zählung verschwinden | Alle aktiven Aufträge erhalten; nur abgeschlossene Historie auf 250 begrenzen; stabile gemeinsame Sortierung | Zwei neue Rust-Regressionstests bestanden; insgesamt 12/12 Rusttests |
| Zoom und manuelles Speichern senden konkurrierende vollständige Einstellungskopien | Ein veralteter Stand kann gerade gespeicherte Einstellungen überschreiben | Schreibvorgänge serialisieren; während manuellem Speichern weitere Einstellungen und Zoom sperren; ausstehenden Zoomtimer vor dem Speichern abbrechen und beim Schließen den Speichervorgang abwarten | Am alten 0.1.0-Release nativ reproduziert; im 0.1.1-Release bestanden, einschließlich Schließen während Speicherung/Pfadaktualisierung und Neustartpersistenz |

Die [native Reproduktion an 0.1.0](../.artifacts/native-1789605675623/report.json) zeigt beim überlappenden Speichern und Zoomen `dark` statt des gerade gespeicherten `light`. Der Test verzögert den echten IPC-Transport gezielt; Backend und gespeicherte Daten werden nicht simuliert. Ein vorausgehender Timeout wegen eines Fehlers im Testharness wird nicht als Produktfehler gewertet.

Der [native 0.1.1-Release-Nachweis](../.artifacts/native-1789605849765/report.json) enthält 15 bestandene Prüfungen. Frontend-Build, 8 Frontendtests, 12 Rusttests und Tauri-/NSIS-Build sind erfolgreich.

Diese Befunde sind nicht als bestätigte Exploits eingestuft. Der Button wurde außerdem von „Datei prüfen“ in „Prüfsumme berechnen“ umbenannt: Der Worker berechnet SHA-256; er führt keinen Virenscan und keine Modell-Sicherheitsprüfung durch.

## Vorhandene Grenzen im Core

- Explizite typisierte Tauri-Commands und die Capability `main-local` beschränken privilegierte Funktionen auf das lokale Hauptfenster; es gibt keine Remote-Scope-Freigabe und keinen allgemeinen Shell-Command für die Oberfläche.
- Die Content Security Policy begrenzt Skripte auf die lokale Anwendung und Verbindungen auf IPC. Der externe Website-WebView besitzt ab 0.3.0 keine lokalen Capabilities.
- SQL-Werte werden als Parameter übergeben. Einstellungen werden im Backend validiert. Ein exklusiver Sessionlock verhindert zwei Instanzen mit derselben Konfigurationsdatenbank.
- Hardwareproben verwenden festgelegte absolute Pfade für Systemprogramme. Modellskripte werden nicht ausgeführt und Modellabhängigkeiten nicht automatisch installiert.
- Hashjobs laufen in einem separaten Prozess mit geprüftem JSON-Protokoll, Timeout und Abbruch. Ein stdin-EOF-Wächter beendet den Worker beim Verlust des Parents. Prozessisolation ist kein Nachweis einer Betriebssystem-Sandbox für künftig fremden Modellcode.
- Die gewählte Datei wird zum Lesen geöffnet. Der Worker vergleicht Größe und Änderungszeit vor und nach dem Lesen; Hash und Metadaten bleiben lokal. Im vorhandenen Anwendungspfad gibt es keine Medien-/Prompt-Uploads oder standardmäßige Telemetrie.

## Abhängigkeiten und offene Prüffelder

`npm audit` meldete am 17. September 2026 keine bekannten Schwachstellen für den geprüften JavaScript-Abhängigkeitsstand. Das schließt unbekannte Schwachstellen nicht aus. Ein vollständiger Rust-Abhängigkeitsaudit wurde nicht durchgeführt; dafür liegt kein positiver Sicherheitsnachweis vor.

OAuth-/Token-Code und Windows Secret Store sind ab 0.2.0 mit den oben genannten Grenzen implementiert. Echte positive Kontoanmeldung, Refresh und Kontowechsel bleiben ungeprüft. Modelldownloads, Modellimporte und Inferenz fehlen weiterhin. Vor ihrer Freigabe müssen die Capability-Trennung des externen WebViews, Downloadpfade und Redirects, sichere Gewichtsformate sowie die Ablehnung fremder Modellskripte geprüft werden. Archiv-, Projekt- und Workflowimporte erhalten ihre eigenen Pfad- und Inhaltsprüfungen mit den entsprechenden Meilensteinen.

Die vorhandenen Tests decken weder einen vollständigen Penetrationstest noch alle Dateisystem-Rennen, beschädigten Eingaben, Ressourcenerschöpfungsfälle oder den vollständigen Installer-/Updatepfad ab. Installation/Deinstallation des NSIS-Setups, signierte Updates und die vollständige M0-/M12-Abnahme bleiben offen. Der gesamte zukünftige Umfang aus `SPEC.md` bleibt erhalten.
