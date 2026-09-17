# LOCAL STUDIO — FINALER MASTERPROMPT FÜR CODEX

## 0. Auftrag

Entwickle im aktuellen Repository eine echte Windows-Desktop-Anwendung namens **Local Studio**.

Local Studio ist eine vollständig lokal arbeitende KI-Kreativsuite zur Verwaltung, Installation und Verwendung lokaler KI-Modelle sowie zur Bearbeitung von Bildern, Videos, Audio und Musik.

Das Projekt soll langfristig folgende Bereiche in einer einheitlichen Anwendung verbinden:

- lokale KI-Modellverwaltung
- Hugging-Face-Integration
- Bildgenerierung
- vollständigen Bildeditor
- Videoerzeugung
- mehrspurigen Videoeditor
- Animation und Motion
- Lipsync
- Audio-/Musikgenerierung und -bearbeitung
- Upscaling und Restaurierung
- Cutout und Inpainting
- Galerie
- Projekte
- Workflows
- lokalen KI-Assistenten
- lokale Medienanalyse
- Hardwareanalyse
- Downloadmanager

Entwickle **funktionierende Software**, keine UI-Demo und keine Sammlung funktionsloser Buttons.

Ein Feature gilt erst als implementiert, wenn der tatsächliche lokale Ausführungspfad funktioniert und getestet wurde.

Arbeite selbstständig durch die unten definierten Entwicklungsphasen. Normale technische Entscheidungen triffst du selbst.

Frage nicht wegen gewöhnlicher Architektur-, Bibliotheks- oder UI-Entscheidungen nach. Recherchiere aktuelle Originaldokumentation, implementiere, teste und dokumentiere deine Entscheidung.

Nur echte Blocker, die eine Nutzerentscheidung zwingend erfordern, dürfen offen bleiben.

---

# 1. Zielplattform

Primäre Zielplattform:

Windows 11 x64.

Erste reale Testplattform ist ein PC mit einer NVIDIA RTX 4080.

Die Hardware von anderen späteren Nutzern ist nicht bekannt.

Deshalb:

- Hardware niemals fest programmieren.
- GPU, VRAM, RAM, CPU und Laufwerke automatisch erkennen.
- NVIDIA zuerst vollständig funktionsfähig machen.
- AMD und Intel erkennen.
- Unterstützung für AMD/Intel nur dort als „unterstützt“ kennzeichnen, wo ein echter funktionierender Backendpfad vorhanden und getestet ist.
- Keine erfundenen Hardware-Kompatibilitätsangaben.

Local Studio ist zunächst für den Entwickler und einen kleinen privaten Freundeskreis gedacht.

Trotzdem soll die Architektur sauber genug sein, dass eine spätere öffentliche Veröffentlichung möglich ist.

---

# 2. Distribution

Erzeuge zwei Varianten:

1. normale installierbare Windows-Version
2. portable Windows-Version

## Installierte Version

- normaler Windows-Installer
- Integration von `.localstudio`-Projektdateien
- automatische App-Updates
- Updates über GitHub Releases
- Repository wird öffentlich sein
- Updatepakete müssen kryptografisch geprüft werden
- keine beliebige heruntergeladene EXE ungeprüft starten

## Portable Version

- kein klassischer Installer erforderlich
- keine automatische Installation neuer App-Versionen
- darf automatisch prüfen, ob eine neue Version verfügbar ist
- dann Hinweis und Downloadmöglichkeit anzeigen
- keine automatische Selbstaktualisierung

---

# 3. Grundprinzip: lokal

Generierung und Medienanalyse erfolgen lokal.

Es gibt keine verpflichtende Cloud-Inferenz.

Keine automatischen Uploads von:

- Bildern
- Videos
- Audio
- Prompts
- Galerie
- Projekten
- Benchmarks

Internet wird nur verwendet, wenn es für Funktionen wie folgende benötigt wird:

- Hugging-Face-Suche
- Modelldownload
- Modellupdates
- App-Updates
- Hugging-Face-Browser

Leistungsdaten und Nutzungsbenchmarks bleiben lokal.

Keine Telemetrie standardmäßig.

---

# 4. Technologie

Bevorzuge für die Hauptanwendung:

- Tauri 2
- React
- TypeScript
- Rust für Desktop-/Systemintegration
- SQLite für lokale Daten
- isolierte lokale Worker für KI-Runtimes
- Python nur dort, wo das KI-Ökosystem es sinnvoll benötigt
- FFmpeg für Video-/Audioverarbeitung
- llama.cpp für geeignete lokale GGUF-Assistenten

Verwende keine monolithische Python-GUI.

Inference darf weder den UI-Thread noch den Haupt-Control-Service blockieren.

Baue eine Adapterarchitektur für Modelle.

Ein Modelladapter beschreibt mindestens:

- Modellfamilie
- unterstützte Aufgaben
- benötigte Dateien
- Runtime
- Eingaben
- Ausgaben
- Parameter
- Ressourcenregeln
- Laden
- Entladen
- Ausführen
- Fortschritt
- Abbruch
- Fehlerbehandlung

Model Card oder Dateiname allein beweisen keine Kompatibilität.

---

# 5. Keine doppelten Implementierungen

Mehrere sichtbare Werkzeuge dürfen dieselbe technische Engine verwenden.

Beispiel:

Image, Edit, Cutout, Upscale und Erase sind unterschiedliche Arbeitsansichten, sollen intern aber gemeinsame Komponenten und dieselbe Medien-/Jobarchitektur verwenden.

Dasselbe gilt für:

- Video
- Animate
- Motion
- Lipsync
- Extend

Baue keine zweite Bildengine nur weil ein Schnellwerkzeug einen eigenen Tab besitzt.

Baue keine zweite Videoengine nur für Animate.

Sichtbare Werkzeuge dürfen task-spezifische Oberflächen besitzen, aber gemeinsame Technik verwenden.

---

# 6. Hauptnavigation

Die dauerhafte linke Hauptnavigation soll ungefähr so aufgebaut sein:

Start

Studio

Modelle

Downloads

Aufträge

Galerie

Hugging Face

Assistent

Einstellungen

Die Navigation soll nicht überladen wirken.

## Studio

Studio enthält:

- Image
- Edit
- Cutout
- Upscale
- Erase
- Video
- Animate
- Lipsync
- Music
- Extend
- Motion
- Workflows

Die Studio-Unterbereiche dürfen als zweite Navigation oder kontextbezogene Werkzeugleiste umgesetzt werden.

---

# 7. UI und Design

Local Studio soll wie eine moderne hochwertige Windows-Kreativanwendung wirken.

Keine überladene Gaming-Oberfläche.

Keine unnötigen Neon-Glow-Effekte.

Keine Dashboard-Kacheln nur um Platz zu füllen.

Verwende:

- klare Hierarchie
- gute Abstände
- ruhige Panels
- große Medienvorschau
- konsistente Icons
- Tooltips
- sichtbare Statusanzeigen
- verständliche Fehlermeldungen

## Themes

Unterstütze:

- Light
- Dark
- Windows-Systemeinstellung

Standard:

Windows-Systemeinstellung.

## Akzentfarbe

Unterstütze:

- frei wählbare Akzentfarbe
- optional Windows-Akzentfarbe übernehmen

## UI-Skalierung

Unterstütze:

- Strg + Mausrad
- Strg + +
- Strg + -
- Strg + 0

UI-Skalierung speichern.

Zusätzlich Einstellung dafür anbieten.

---

# 8. Sprache

Local Studio ist von Anfang an internationalisierbar.

Direkt enthalten:

- Deutsch
- Englisch

Beim ersten Start:

Windows-Sprache erkennen.

Wenn Deutsch unterstützt:

Deutsch.

Wenn Englisch:

Englisch.

Wenn Windows-Sprache nicht unterstützt:

Englisch.

Sprache jederzeit in Einstellungen änderbar.

---

# 9. Tastenkürzel

Alle wesentlichen Tastenkürzel zentral verwalten.

Sie müssen in Einstellungen frei änderbar sein.

Konflikte erkennen.

Mindestens:

- Strg+S = Projekt speichern
- Strg+Z = Undo
- Strg+Y = Redo
- Generierung starten
- Löschen
- UI-Zoom
- Bildansicht zurücksetzen
- Bild einpassen
- 100%-Ansicht

Undo-Limit:

Standard 100 Schritte.

In Einstellungen veränderbar.

---

# 10. Ersteinrichtung

Beim ersten Start einen übersichtlichen Setup-Assistenten zeigen.

Er soll:

1. Datenordner einrichten
2. Hardware erkennen
3. optional Hugging Face verbinden
4. kleines Chatmodell prüfen
5. optional vorhandene Modelle suchen

Keine Bild-, Video- oder Musikmodelle automatisch installieren.

## Kleines Chatmodell

Nur das kleine Assistentenmodell wird beim ersten Start angeboten/benötigt.

Vor Download zuerst prüfen, ob bereits ein kompatibles lokales Exemplar vorhanden ist.

Wenn vorhanden:

verwenden bzw. einbinden.

Wenn nicht:

Download anbieten.

Nicht in Installer integrieren.

Wähle bei Implementierung einen aktuellen geeigneten deutsch- und englischfähigen kleinen lokalen Kandidaten ungefähr in der 1–3B-Klasse.

Anforderungen:

- GGUF oder ähnlich sinnvoll lokal
- llama.cpp-kompatibel
- vernünftige Tool-Calling-Fähigkeit
- brauchbare deutsche Sprache
- Lizenz prüfen
- echte Tests durchführen

Modellwahl nicht fest aus alter Dokumentation übernehmen.

---

# 11. Speicherpfade

Alle wesentlichen Pfade müssen in Einstellungen geändert werden können:

- Modelle
- Assistentenmodelle
- Visionmodelle
- Downloads
- Galerie
- Projekte
- temporäre Dateien
- Recovery
- Cache
- Proxydateien

## Standard

Portable Version:

Local-Studio-Datenordner neben bzw. unterhalb der portablen Anwendung.

Installierte Version:

eigener benutzerbeschreibbarer Local-Studio-Datenordner im Windows-Nutzerbereich.

Nicht versuchen, Nutzerdaten in einen nicht beschreibbaren Program-Files-Ordner zu schreiben.

---

# 12. Hugging Face

Hugging Face ist die primäre Onlinequelle für Modelle.

Nutze offizielle Hugging-Face-APIs.

Unterstütze:

- Suche
- Pagination
- Filter
- Model Cards
- Repo-ID
- Revisionen
- Dateiinformationen
- Lizenzinformationen
- gated models
- OAuth

## Login

Genau ein Hugging-Face-Konto gleichzeitig.

Unterstütze:

- anmelden
- abmelden
- danach anderes Konto anmelden

Standard:

OAuth für eine native öffentliche App.

Manuelle Tokens nur als erweiterte Alternative.

Tokens niemals im Klartext in normalen Config-Dateien speichern.

Windows-geschützten Secret Store verwenden.

## Tokenverwaltung

Ein Button „Tokens auf Hugging Face verwalten“ darf die offizielle Verwaltung öffnen.

Local Studio soll nicht selbst einen unsicheren Ersatz für Hugging-Face-Tokenverwaltung bauen.

---

# 13. Eingebetteter Hugging-Face-Browser

Eigener Hauptbereich:

Hugging Face.

Dieser zeigt die echte Hugging-Face-Website in einem eingebetteten WebView.

Der Browser darf nur Hugging-Face-Domains als normale In-App-Navigation verwenden.

Externe Links öffnen im Windows-Standardbrowser.

Der eingebettete WebView besitzt eine persistente eigene Browser-Sitzung.

Wenn sich der Nutzer dort bei Hugging Face anmeldet, soll die WebView-Sitzung erhalten bleiben.

Der OAuth-App-Login und WebView-Login sind technisch getrennte Sessions.

Nicht versuchen, OAuth-Tokens künstlich in Browser-Cookies umzuwandeln.

## Sicherheit

Der externe Hugging-Face-WebView darf keinen direkten Zugriff auf privilegierte Tauri-/Systemfunktionen bekommen.

Strikte Capability-Trennung.

Eine manipulierte Webseite darf nicht:

- Dateien löschen
- Shellbefehle starten
- beliebige Local-Studio-Tools aufrufen
- Secrets lesen

Wenn möglich, Modellseiten erkennen und eine sichere Local-Studio-Aktion anbieten:

„In Local Studio öffnen“

oder

„Mit Local Studio herunterladen“.

Die Repo-ID muss validiert werden.

---

# 14. Modellbereich

Der zentrale Bereich „Modelle“ ist für vollständige Basismodelle vorgesehen.

Jeder Eintrag zeigt mindestens:

- Modellname
- Repo/Quelle
- Entwickler
- Modellfamilie
- Einsatzzweck
- installierte Variante
- Dateigröße
- Installationspfad
- Status
- Update verfügbar
- Kompatibilität
- lokale Leistungsdaten

Einsatzzwecke können z. B. sein:

- Chat
- Bild
- Video
- Musik
- Audio
- Upscale
- Cutout
- Vision

Mehrere Einsatzzwecke gleichzeitig möglich.

---

# 15. Model Search

Es gibt einen Filter:

„Nur mit Local Studio ausführbare Modelle anzeigen“

Dieser ist ein-/ausschaltbar.

Wenn ausgeschaltet:

auch Modelle anzeigen, die Local Studio nicht ausführen kann.

Solche Modelle dürfen trotzdem heruntergeladen und verwaltet werden.

Deutlich anzeigen:

„Download möglich – Ausführung in Local Studio derzeit nicht unterstützt.“

Keine automatische Vorauswahl einer Modellvariante.

Der Nutzer entscheidet frei:

- Modell
- Variante
- Quantisierung
- Revision

Local Studio darf Empfehlungen anzeigen, aber nichts ungefragt auswählen.

---

# 16. Weitere Modellquellen

Primär Hugging Face.

Zusätzlich unterstützen:

- Import vom PC
- direkte Downloadquelle, soweit sicher und sinnvoll implementierbar

Jede Quelle klar kennzeichnen.

Nicht vertrauenswürdige Dateien niemals automatisch ausführen.

---

# 17. Lokale Modellsuche

Unter Modelle:

„PC nach Modellen durchsuchen“.

Beim ersten Setup danach fragen.

Nicht ungefragt starten.

## Modi

Schnellsuche:

- bekannte Hugging-Face-Caches
- typische AI-Modellordner
- bekannte Local-Studio-Bibliotheken
- typische Speicherorte

Vollständige Suche:

alle verfügbaren lokalen Laufwerke durchsuchen.

Unzugängliche Ordner überspringen und protokollieren.

## Verhalten

Eindeutig erkannte Modelle automatisch in Local Studio importieren/einbinden.

Nicht automatisch verschieben.

Sie bleiben am ursprünglichen Speicherort.

Bei jedem gefundenen Modell Button:

„Verschieben“.

Unsichere Funde nur als Kandidaten anzeigen.

Keine Modellcodes während Scan ausführen.

Neue externe Modelle werden nicht dauerhaft überwacht.

Neue Modelle werden erst beim nächsten manuellen Scan entdeckt.

Bereits importierte Modellpfade dürfen jedoch überwacht werden.

Wenn eine bekannte Datei verschwindet:

sofort bzw. zeitnah „Nicht gefunden“ anzeigen.

Wenn Laufwerk fehlt:

„Laufwerk nicht verfügbar“.

Nicht automatisch den ganzen PC nach dem neuen Pfad durchsuchen.

Neue Zuordnung erfolgt erst beim nächsten bewussten Scan.

---

# 18. Modellinstallation

Ein Modell einzeln:

- installieren
- laden
- entladen
- reparieren
- verschieben
- aktualisieren
- entfernen

Vor Installation anzeigen:

- Dateien
- Variante
- Revision
- Downloadgröße
- zusätzlicher Speicherbedarf
- Komponenten
- Lizenz
- Zugangsvoraussetzungen
- Runtime

Keine versteckte Variante wählen.

---

# 19. Modellverschieben

Modelle dürfen zwischen Laufwerken verschoben werden.

Sicherer Ablauf:

1. Modell entladen/sperren
2. Ziel vorbereiten
3. Daten kopieren
4. Datenintegrität prüfen
5. neue Zuordnung schreiben
6. erst dann alte Daten entfernen

Bei Abbruch oder Crash darf nicht die einzige funktionsfähige Kopie verschwinden.

---

# 20. Modellupdates

Automatische Suche nach Modellupdates:

standardmäßig EIN.

Aber:

Updates niemals automatisch installieren.

Möglichkeiten:

„Aktualisieren“ beim einzelnen Modell.

„Alle aktualisieren“.

„Alle aktualisieren“ startet direkt alle verfügbaren Modellupdates.

Keine zusätzliche Auswahlliste notwendig.

Alte Version soll nach erfolgreichem Update nicht standardmäßig dauerhaft behalten werden.

Während Update alte Version aber erhalten, bis neue Installation geprüft wurde.

---

# 21. Zusatzmodelle und Erweiterungen

LoRA, VAE, ControlNet und ähnliche Komponenten erscheinen NICHT als normale Einträge in der zentralen Modellbibliothek.

Sie erscheinen kontextbezogen beim entsprechenden Studio-Basismodell.

Beispiel Image:

Erweiterungen

LoRA
3 installiert
18 kompatible verfügbar

ControlNet
2 installiert
weitere verfügbar

VAE
Standard aktiv
2 Alternativen verfügbar

Local Studio soll selbst auf Hugging Face nach kompatiblen Erweiterungen suchen.

Nur tatsächliche Kompatibilität behaupten.

Installieren erfolgt auf Nutzeraktion.

Auch NSFW-Erweiterungen anzeigen, wenn 18+-Modus aktiviert ist.

---

# 22. Downloads

Eigener Bereich Downloads.

Jeder Download zeigt:

- Name
- Modell/Datei
- Fortschritt
- übertragene Bytes
- Gesamtgröße
- Geschwindigkeit
- Restmenge
- Restzeit, wenn sinnvoll berechenbar
- Status
- Zielordner

Unterstützen:

- Pause
- Fortsetzen
- Abbrechen
- Retry
- Priorität
- Neustart-Recovery

Pause muss tatsächliche Übertragung stoppen.

Wenn Backend keine bytegenaue Fortsetzung ermöglicht, ehrlich anzeigen.

Keine erfundene Resume-Funktion.

Teilweise vollständige Dateien möglichst wiederverwenden.

Abgebrochene fortsetzbare Downloads nicht automatisch löschen.

---

# 23. Aufträge und Scheduler

Eigener Bereich:

Aufträge.

Standard:

nur ein großer Generierungsauftrag gleichzeitig.

Parallele Generierung ist eine Einstellung und standardmäßig AUS.

Wenn aktiviert:

nur ausführen, wenn Ressourcen ausreichend sind.

Jeder Job speichert beim Start einen unveränderlichen Snapshot aus:

- Modell
- Revision
- Erweiterungen
- Parametern
- Eingabemedien

Wenn während Auftrag A ein anderes Modell ausgewählt wird:

Auftrag A bleibt unverändert.

Neue Auswahl betrifft Auftrag B.

---

# 24. Statusmarkierungen an Studio-Tabs

Wenn ein Werkzeug arbeitet, muss das in der Studio-Navigation sichtbar bleiben.

Beispiel:

Image
1 neues Ergebnis

Video
Läuft · 42 %

Music
2 warten

Wenn echter Prozentfortschritt nicht verfügbar:

keinen erfundenen Wert anzeigen.

Dann z. B.:

„Video wird erzeugt“.

Unterscheide:

- Modell wird geladen
- wartet auf Ressourcen
- generiert
- verarbeitet
- speichert
- abgeschlossen
- Fehler

Außerdem globaler kompakter Jobstatus außerhalb des aktuellen Studio-Tabs.

Ein Klick öffnet den Auftrag.

Nicht ungefragt den aktuellen Tab wechseln.

---

# 25. Modellwechsel innerhalb der Werkzeuge

Jeder Studio-Bereich hat einen gut sichtbaren Modellwahlschalter.

Beispiel:

Video → Modell A auswählen → Video generieren → Modell B auswählen → neues Video generieren.

Kein App-Neustart.

Installiertes Modell beim Wechsel nicht erneut herunterladen.

Trenne:

SELECTED MODEL

und

LOADED MODEL.

Ein Modell wird nicht nur wegen der Auswahl sofort in VRAM geladen.

Laden erst bei tatsächlichem Bedarf oder explizitem Befehl.

Prompt und Eingabedaten beim Modellwechsel soweit sinnvoll erhalten.

Modellspezifische Einstellungen separat speichern.

Wenn Modell A wieder ausgewählt wird:

dessen letzte Einstellungen wiederherstellen.

---

# 26. Hardwaremessung

Local Studio erkennt:

- GPU
- VRAM
- CPU
- RAM
- freie Laufwerkskapazität

Vor großen Jobs:

voraussichtlichen Speicherbedarf prüfen.

Wenn Speicherplatz wahrscheinlich nicht reicht:

warnen.

## Live-Hardwareanzeige

Optional:

- GPU-Auslastung
- VRAM
- RAM
- Temperatur, wenn zuverlässig verfügbar

Standard:

AUS.

In Einstellungen aktivierbar.

---

# 27. Lokale Benchmarks

Nach Generierungen automatisch technische Messwerte speichern:

- Modell
- Modellvariante
- Aufgabe
- Auflösung
- Frames/Dauer
- Hardware
- Laufzeit
- maximaler VRAM, soweit zuverlässig
- RAM, soweit sinnvoll
- relevante Einstellungen

Diese Daten bleiben lokal.

Beim Modell:

Bereich „Leistung“.

Dort vergangene Messungen anzeigen.

Der Assistent darf sie für Empfehlungen verwenden.

Keine Benchmarkdaten hochladen.

---

# 28. Lokaler Assistent

Eigener Bereich:

Assistent.

Der Assistent arbeitet lokal.

Er darf:

- App erklären
- Modelle suchen
- Hardware erklären
- Modelle empfehlen
- Modell auswählen
- Parameter setzen
- Downloads starten
- Jobs vorbereiten
- Jobs starten
- Galerie durchsuchen
- Medien öffnen
- Workflows starten
- mehrere zusammengehörige Schritte ausführen

Nutze Tool Calling.

Der Assistent erhält ausschließlich definierte Local-Studio-Tools.

Kein freier Shellzugriff.

Kein beliebiges Dateisystemtool.

Kein `eval`.

---

# 29. Assistentenaktionen

Wenn der Nutzer einen klaren Auftrag erteilt:

zusammengehörige Schritte ohne wiederholte Bestätigung durchführen.

Beispiel:

„Nimm dieses Bild, entferne die Person und upscale auf 4K.“

Der Assistent darf:

1. passendes Erase-Modell auswählen
2. Erase durchführen
3. Upscale-Modell auswählen
4. Upscale durchführen

Keine unnötigen Bestätigungsdialoge.

## Ausnahme: Löschen

Löschen von:

- Modellen
- Medien
- Projekten
- Nutzerdaten

braucht immer gesonderte Bestätigung.

---

# 30. Assistenten-Modellwahl

Wenn Nutzer explizit ein Modell nennt:

genau dieses verwenden, sofern technisch möglich.

Wenn Nutzer kein Modell nennt:

Assistent darf aus bereits installierten geeigneten Modellen wählen.

Berücksichtigen:

- Aufgabe
- Kompatibilität
- lokale Benchmarks
- Hardware
- gelernte Präferenzen

Fehlt ein notwendiges großes Modell:

nicht ungefragt herunterladen.

Installation anbieten.

18+-Sperre niemals umgehen.

---

# 31. Gelernte Präferenzen

Assistent darf lokal lernen:

- bevorzugte Modelle
- bevorzugte Einstellungen
- bevorzugte Seitenverhältnisse
- häufige Qualitätsparameter
- wiederkehrende Abläufe

Einstellungen:

„Gelernte Präferenzen“.

Nutzer kann:

- ansehen
- ändern
- einzeln löschen
- vollständig zurücksetzen

Explizite aktuelle Nutzerwahl hat immer Vorrang.

---

# 32. Vision und Medienanalyse

Der Assistent darf lokale Galerie-Metadaten durchsuchen.

Zusätzlich soll lokale Inhaltsanalyse möglich sein.

## Bilder

Beispiele:

„Welche Bilder zeigen ein Auto?“

„Finde Bilder mit Bergen.“

## Video

Beispiele:

„Was passiert in diesem Video?“

„Wann erscheint ein Auto?“

## Audio

Beispiele:

- transkribieren
- gesprochene Begriffe suchen
- Tempo analysieren
- Instrumentierung erkennen
- Struktur analysieren

Zusätzliche Analysemodelle NICHT beim ersten Setup automatisch installieren.

Beim ersten tatsächlichen Bedarf:

fragen, ob passendes Modell installiert werden soll.

Danach installiert behalten, bis Nutzer es entfernt.

Medien außerhalb Galerie dürfen per Drag-and-drop in Assistent gezogen werden.

Mehrere Medien gleichzeitig vergleichbar machen.

---

# 33. Galerie-Inhaltsindex

In Einstellungen:

Galerie → Inhaltsanalyse.

Modi:

Bei Bedarf
Standard.

Automatisch im Hintergrund.

Aus.

Bei Bedarf:

erst analysieren, wenn Inhaltsfunktion verwendet wird.

Index lokal speichern.

Nur erneut analysieren, wenn Datei verändert wurde.

---

# 34. Galerie

Galerie ist dauerhafter Hauptbereich.

Speichert:

- Bilder
- Videos
- Audio
- Musik

Die Galerie benutzt echte Ordner auf dem Laufwerk.

Keine rein virtuelle Ordnerstruktur.

Explorer-Kompatibilität erhalten.

---

# 35. Galerie-Pfad ändern

Beim Ändern:

Dialog:

„Bestehende Galerie verschieben“

oder

„Nur neuen Galerieordner verwenden“.

## Verschieben

- kopieren
- prüfen
- neue Zuordnung setzen
- alte Daten erst anschließend entfernen

## Nur neuen Ordner verwenden

alte Galerie bleibt unverändert am alten Ort.

---

# 36. Galerie-Erkennung

Wenn Nutzer über Windows Explorer Dateien in Galerieordner kopiert:

Galerie erkennt sie automatisch.

Drag-and-drop in Galerie unterstützen.

## Drag-and-drop

Normal:

verschieben.

Mit Strg:

kopieren.

Während Drag:

Aktion sichtbar anzeigen.

Auf anderem Laufwerk:

Original erst nach erfolgreichem Kopieren entfernen.

Keine stillen Überschreibungen.

---

# 37. Generierte Ergebnisse speichern

Generierungsergebnisse NICHT automatisch dauerhaft in Galerie speichern.

Sie bleiben zunächst temporäre Ergebnisse.

Speichern möglich über:

- Speichern
- Drag auf leere Galeriefläche
- Drag auf Galerieordner

Drag auf leeren Bereich:

im aktuell geöffneten Galerieordner speichern.

Dateiname automatisch eindeutig vergeben.

Danach in Galerie umbenennbar.

Umbenennen ändert echten Dateinamen.

---

# 38. Ungespeicherte Ergebnisse

Beim Wechsel zwischen Studio-Tabs:

ungespeicherte Ergebnisse behalten.

Beim Beenden:

warnen.

Optionen:

- speichern
- verwerfen
- Abbrechen

Nicht automatisch verwerfen.

---

# 39. Drag zwischen Studio und Galerie

Galerie → Studio:

Medien direkt in kompatibles Werkzeug ziehen.

Beispiele:

Bild → Edit

Bild → Upscale

Bild → Video als Referenz

Audio → Lipsync

Video → Motion

Temporäres Ergebnis → anderes Studio-Werkzeug:

auch ohne vorheriges Speichern möglich.

Bei inkompatiblem Ziel:

verständlich anzeigen.

---

# 40. Galerie-Metadaten

Mit gespeicherten KI-Ergebnissen speichern:

- Modell
- Modellrevision
- Erweiterungen
- Prompt
- Negative Prompt
- Seed
- Generierungseinstellungen
- Herkunftsjob
- Quellmedium
- Datum
- Operation

Info-Button zeigt diese Informationen.

Button:

„Einstellungen wiederherstellen“.

Dieser öffnet das passende Studio-Werkzeug und setzt:

- Modell
- Prompt
- Negative Prompt
- Seed
- kompatible Einstellungen

wieder ein.

---

# 41. Galerieorganisation

Unterstütze:

- echte Ordner
- Favoriten
- frei definierbare Tags
- Suche
- Mehrfachauswahl
- Sammelaktionen

Suche mindestens über:

- Dateiname
- Modell
- Prompt
- Medientyp
- Tags

Ansichten:

Raster.

Liste/Details.

Listenansicht kann zeigen:

- Dateiname
- Auflösung
- Dateigröße
- Modell
- Erstellungsdatum

---

# 42. Thumbnails

Lokale Thumbnails erzeugen für:

- große Bilder
- Videos

Originale nicht verändern.

Thumbnail-Cache neu aufbaubar.

Große Galerie virtualisieren.

Nicht tausende Originalmedien gleichzeitig laden.

---

# 43. Galerie-Viewer

Integrierter Medienviewer.

Bilder:

- Zoom
- Pan
- Vollbild

Video:

- Wiedergabe
- Timeline

Audio:

- Wiedergabe
- Waveform soweit sinnvoll

Pfeiltasten:

vorheriges/nächstes Medium.

---

# 44. Versionen und Herkunft

Bearbeitung immer zerstörungsfrei.

Original nicht überschreiben.

Neue Bearbeitung = neue Datei.

Speichere Herkunft.

Beispiel:

Original → Erase → Edit → Upscale.

Galerie zeigt grafische Entstehungskette.

Vorhandene Version anklickbar.

Endgültig gelöschte Quelle:

nur Metadateneintrag behalten.

Keine versteckte Kopie einer endgültig gelöschten Datei.

---

# 45. Versionsgruppen

Zusammengehörige Versionen automatisch gruppieren.

Standard-Hauptvorschau:

zuletzt erstellte Version.

Nutzer kann manuell eine andere Hauptversion festlegen.

Löschen betrifft nur ausdrücklich ausgewählte Dateien.

Original löschen darf andere Versionen nicht automatisch löschen.

---

# 46. Vorher/Nachher

Für abgeleitete Bilder:

- Nebeneinanderansicht
- interaktiver Split-/Wipe-Regler

---

# 47. App-Papierkorb

Eigener Local-Studio-Papierkorb.

Galerie-Löschen verschiebt nach Bestätigung zuerst dorthin.

Papierkorb zeigt:

- Vorschau
- ursprünglichen Ordner
- Löschdatum
- Größe

Unterstütze:

- wiederherstellen
- endgültig löschen
- Papierkorb leeren

Endgültiges Löschen:

erneute Bestätigung.

Aufbewahrungsfrist einstellbar.

Standard:

nicht automatisch löschen.

Wenn automatische Löschung aktiviert:

entsprechende Frist anwenden.

---

# 48. Projekte

Local Studio kann vollständig ohne Projekt benutzt werden.

Zusätzlich speicherbare Projekte.

Nur ein Projekt gleichzeitig geöffnet.

Projektdateiendung:

`.localstudio`.

Doppelklick soll später Projekt mit Local Studio öffnen.

Speichern:

manuell.

Strg+S.

Keine normale automatische Speicherung als Projekt.

---

# 49. Projektinhalt

Eine `.localstudio`-Datei ist ein komprimierter Container.

Einbetten:

- Bilder
- Videos
- Audio
- Masken
- Arbeitsstände
- RAW-Quellen
- Ebenen
- Timelineinformationen
- Projektmetadaten

Nicht einbetten:

- Basismodelle
- LoRAs
- VAEs
- ControlNets
- andere KI-Gewichte

Speichere stattdessen exakte Referenzen:

- Quelle
- Repo
- Revision
- Variante

Beim Öffnen:

prüfen, welche Abhängigkeiten fehlen.

Dialog z. B.:

3 Komponenten fehlen.

[Alle installieren]
[Ohne fortfahren]

Ohne fehlende Modelle öffnen können.

Betroffene Funktionen deaktivieren.

---

# 50. Projektkompression

Verlustfrei komprimieren, wo sinnvoll.

Bereits stark komprimierte Medien wie MP4/JPEG nicht unnötig erneut komprimieren.

Nicht mehr verwendete eingebettete Medien:

während laufender Sitzung für Undo temporär behalten.

Bei Strg+S:

aus dem Projektcontainer bereinigen, wenn nicht mehr benötigt.

Galeriedateien dabei niemals löschen.

---

# 51. Projektlos arbeiten

Wenn Local Studio ohne Projekt benutzt wird:

vollständige Studiofunktionen verfügbar.

Beim Beenden mit relevantem Arbeitsstand:

fragen:

„Aktuellen Stand als Projekt speichern?“

Diese Abfrage mit anderen Exit-Warnungen sinnvoll in EINEM konsolidierten Dialog kombinieren.

Keine Dialogkette aus fünf einzelnen Fenstern.

---

# 52. Startverhalten

Einstellbar:

- leere Sitzung
- letzten Arbeitsstand wiederherstellen

Standard:

leere Sitzung.

Wenn Wiederherstellung aktiv:

auch temporäre ungespeicherte Ergebnisse wiederherstellen.

---

# 53. Crash Recovery

Unabhängig von Startoption.

Temporären Sitzungszustand regelmäßig sichern.

Bei Crash oder Windows-Abbruch:

beim nächsten Start Recovery anbieten.

Recovery darf enthalten:

- Eingaben
- Prompts
- Einstellungen
- ungespeicherte Ergebnisse
- Masken
- Timelinezustand
- verfügbare Generierungs-Zwischenstände

Echte Generierungsfortsetzung nur anbieten, wenn konkrete Pipeline Checkpoint-Resume unterstützt.

Sonst:

Zwischenstand erhalten, Job aber als unterbrochen markieren.

---

# 54. Temporärdaten

Automatische Bereinigung standardmäßig EIN.

Standard:

nicht mehr benötigte temporäre Generierungsdaten nach 7 Tagen löschen.

Letzte 3 Recovery-Punkte schützen.

Fortsetzbare Downloadreste nicht automatisch löschen.

Einstellungen:

- aktivieren/deaktivieren
- Frist ändern
- Jetzt bereinigen

Vor manueller Bereinigung anzeigen:

- was gelöscht wird
- wie viel Speicher frei wird

---

# 55. 18+-Modus

NSFW/18+ ist primär eine Inhaltskennzeichnung.

Ein technisch kompatibles Modell darf nicht allein wegen des NSFW-Tags künstlich langsamer, eingeschränkter oder nicht ausführbar gemacht werden.

## Standard

18+-Modus AUS.

Beim erstmaligen Aktivieren:

- bestätigen, dass Nutzer mindestens 18 Jahre alt ist
- lokale Sperre einrichten

Wahl:

- PIN
- Passwort

Kein Klartextpasswort speichern.

## Auto-Lock

Einstellung:

„18+-Bereich nach App-Neustart erneut sperren“.

Standard:

EIN.

---

# 56. 18+-Modelle in Studio

Installierte 18+-Modelle bleiben sichtbar und auswählbar.

Wenn 18+-Bereich gesperrt:

Arbeits-/Eingabebereich geblurt.

Button:

„Entsperren“.

Erst danach:

- Prompt bearbeiten
- Eingaben anzeigen
- neue Generierung starten

Laufende Generierung wird durch erneutes Sperren NICHT abgebrochen.

Nur Anzeige wird geschützt.

---

# 57. 18+-Galerie

Wurde ein Medium mit einem als NSFW/18+ gekennzeichneten Basismodell oder einer solchen Erweiterung erzeugt:

bei gesperrtem 18+-Bereich immer bluren.

Unabhängig davon, ob das konkrete Bild explizit aussieht.

Keine zusätzliche Inhaltsklassifikation erforderlich.

Klick auf geblurtes Medium:

PIN-/Passworteingabe.

Info-Metadaten ebenfalls sperren.

Manuell importierte Medien können NICHT manuell mit diesem 18+-Flag versehen werden.

---

# 58. 18+-Analyse

Bei gesperrtem 18+-Bereich:

keine neue Inhaltsanalyse geschützter Medien starten.

Geschützte Inhalte nicht in semantischen Suchergebnissen anzeigen.

Bereits vorhandener lokaler Index bleibt gespeichert, aber gesperrt.

---

# 59. IMAGE – Generierung

Image ist die erste KI-Hauptfunktion, die vollständig funktionsfähig werden muss.

Unterstütze abhängig vom jeweiligen Modell:

- Text-to-Image
- Image-to-Image
- Referenzbilder
- Seed
- Auflösung
- Seitenverhältnis
- Steps
- Guidance
- Scheduler/Sampler
- Batch
- LoRA
- ControlNet
- VAE
- weitere echte Modellparameter

Keine wirkungslosen Regler anzeigen.

Modellabhängige UI aus Capabilities erzeugen.

---

# 60. Bildeditor

Local Studio soll einen echten zerstörungsfreien Bildeditor besitzen.

## Ebenen

Unterstütze:

- Rasterebenen
- Text
- Vektorformen
- Masken
- Einstellungsebenen
- Gruppen

Layer-Funktionen:

- Sichtbarkeit
- Sperren
- Umbenennen
- Reihenfolge
- Deckkraft
- Duplizieren
- Gruppieren
- Zusammenführen
- gängige Blend Modes

Mindestens:

- Normal
- Multiply
- Screen
- Overlay

Weitere sinnvolle Standard-Blending-Modi ergänzen.

---

# 61. Bildeditor – Transformation

Unterstütze:

- Verschieben
- Skalieren
- Drehen
- perspektivische Transformation
- Verzerren

---

# 62. Bildeditor – Auswahl

Unterstütze:

- Rechteckauswahl
- Lasso
- Zauberstab/Farbauswahl
- KI-Objektauswahl

KI-Objektauswahl:

- Objekt anklicken
- Objekt per Text beschreiben

Mehrere Objekte:

- hinzufügen
- entfernen

Standard z. B.:

Shift hinzufügen.

Alt entfernen.

Shortcuts änderbar.

---

# 63. Maskeneditor

Unterstütze:

- Malen
- Radieren
- Pinselgröße
- Härte
- weiche Kante
- Deckkraft
- invertieren
- Undo/Redo
- Zoom

Automatische KI-Auswahl muss anschließend manuell korrigierbar sein.

Wenn Segmentierungs-/Grounding-Modell fehlt:

beim ersten Bedarf Installationsdialog.

---

# 64. Bildeditor – klassische Werkzeuge

Unterstütze:

- Pinsel
- Radierer
- Pipette
- Füllen
- Klonstempel
- Zuschneiden
- Drehen
- Spiegeln
- Resize

Farb-/Bildkorrektur:

- Helligkeit
- Kontrast
- Sättigung
- Farbtemperatur
- Schärfe

Diese soweit sinnvoll als zerstörungsfreie Einstellungsebenen.

---

# 65. Text und Vektor

Text-Ebenen:

- Windows-Schriften
- optionaler Local-Studio-Schriftordner
- Größe
- Farbe
- Ausrichtung
- Kontur

Vektorformen:

- Rechteck
- Kreis
- Linie
- Pfeil
- Füllung
- Kontur

---

# 66. Bildeditor-Navigation

Im Bild-/Maskeneditor:

Mausrad:

Bild zoomen.

Mittlere Maustaste:

Pan/Verschieben.

Strg+Mausrad:

globale UI-Skalierung.

Ansichtsaktionen:

- Fit
- 100 %
- Ansicht zurücksetzen

---

# 67. Bildformate

Mindestens:

- PNG
- JPEG
- WebP
- TIFF

Alpha bei geeigneten Formaten erhalten.

Zusätzlich Kamera-RAW:

- CR2
- CR3
- NEF
- ARW
- DNG
- weitere sinnvoll unterstützte RAW-Formate

---

# 68. RAW-Workflow

Eigener RAW-Entwicklungsdialog.

Mindestens:

- Belichtung
- Weißabgleich
- Highlights
- Schatten
- Schwarzpunkt
- Weißpunkt
- Objektivkorrektur
- Rauschreduzierung

RAW-Entwicklung zerstörungsfrei.

RAW-Quelle und Entwicklungseinstellungen im Projekt erhalten.

---

# 69. Farbmanagement

Unterstütze ICC-Profile.

Mindestens sinnvoll behandeln:

- sRGB
- Display P3
- Adobe RGB

Keine stillen Farbraumkonvertierungen ohne nachvollziehbares Verhalten.

---

# 70. HDR

Unterstütze geeignete:

- 16-Bit
- 32-Bit
- HDR-Workflows

Wenn Windows HDR und Monitor geeignet:

echte HDR-Vorschau verwenden, soweit technisch korrekt implementierbar.

Auf SDR:

Tone-Mapping-Fallback.

---

# 71. KI-Bildverbesserung

Lokale Funktionen:

- Denoise
- Schärfen
- Gesichtsrestaurierung
- alte Fotos restaurieren
- Schwarz-Weiß-Fotos kolorieren
- Upscale
- Cutout
- Inpainting/Erase
- Outpainting/Extend

Benötigte Modelle kontextbezogen installieren.

---

# 72. Bild-Batch

Mehrere Bilder gleichzeitig verarbeiten.

Beispiele:

- Upscale
- Cutout
- Konvertieren
- Denoise
- Restore

Jede Datei als eigener Jobstatus.

Fehler bei einer Datei darf Batch nicht zerstören.

Retry pro Datei.

## Zielordner

Vor Start anzeigen.

Automatisch sinnvollen Unterordner vorschlagen, z. B.:

`Upscaled`

Aber frei änderbar.

Keine destruktive „Originale automatisch ersetzen“-Voreinstellung.

---

# 73. Batch-Dateinamen

Namensvorlagen unterstützen.

Standard:

`{original}_{operation}`

Variablen z. B.:

- original
- operation
- model
- date
- time
- resolution
- counter

Vorschau vor Start.

Namenskollisionen vermeiden.

Vorlagen als Preset speicherbar.

---

# 74. Bildexport

Formabhängige Optionen.

JPEG:

- Qualität
- Chroma-Subsampling
- Farbraum
- Metadaten

PNG:

- Kompression
- Bit-Tiefe
- Alpha
- Farbprofil
- Metadaten

WebP:

- lossless/lossy
- Qualität
- Alpha
- Metadaten

TIFF:

- Kompression
- Bit-Tiefe
- Farbprofil
- Metadaten

Live-Dateigrößenschätzung, wenn sinnvoll.

Presets:

- Web
- Hohe Qualität
- Archiv
- Benutzerdefiniert

---

# 75. Export-Metadaten

Separat wählbar:

- EXIF
- GPS
- Farbprofil
- Local-Studio-Generierungsinformationen

GPS sichtbar kennzeichnen.

---

# 76. Export-Resize

Direkt im Export möglich.

Optionen:

- Original
- Prozent
- feste Breite
- feste Höhe
- längste Kante
- 1080p
- 1440p
- 4K
- Benutzerdefiniert

Klar unterscheiden von KI-Upscale.

---

# 77. Wasserzeichen

Optional.

Standard AUS.

Unterstütze:

- Text
- PNG/Logo
- Position
- Größe
- Deckkraft
- Randabstand

Nur im Export rendern.

Original nicht verändern.

Presets erlauben.

---

# 78. Mehrfache Exportvarianten

Ein Medium kann in einem Export mehrere Presets erzeugen.

Beispiel:

- 4K PNG Archiv
- 1920 WebP
- JPEG zum Teilen

Bei mehreren Medien über normale Jobqueue.

---

# 79. VIDEO – Grundsystem

Video-Bereich enthält sowohl:

- KI-Videoerzeugung
- echten Timeline-Editor

Nicht nur Generator.

Mehrere Video- und Audiospuren.

---

# 80. Video-KI

Abhängig vom Modell:

- Text-to-Video
- Image-to-Video
- Video-to-Video
- Animate
- Motion
- Extend
- Video-Inpainting
- Stiltransfer

Mindestens zwei reale Modelle für dieselbe Video-Unteraufgabe integrieren und tatsächlich testen, damit Modellwechsel praktisch nachgewiesen wird.

---

# 81. Timeline

Mehrspurige Timeline.

Unterstütze:

- mehrere Videospuren
- mehrere Audiospuren
- Bildclips
- Text
- Untertitel
- Effekte
- Übergänge

Clips:

- schneiden
- trimmen
- verschieben
- teilen
- duplizieren

---

# 82. Keyframes

Unterstütze Keyframes für:

- Position
- Skalierung
- Rotation
- Deckkraft
- Lautstärke
- Effektparameter

Sinnvolle Interpolation anbieten.

---

# 83. Klassische Videoeffekte

Mindestens:

- Crossfade
- Dip to Black
- Blur
- Farbkorrektur
- Geschwindigkeit
- Slow Motion
- Stabilisierung
- Chroma Key/Greenscreen

---

# 84. Proxyworkflow

Unterstütze Proxy-Dateien.

Bei großen/aufwendigen Medien:

Proxy-Erstellung anbieten.

Manuell aktivierbar.

Proxy-Auflösung:

- 1/2
- 1/4
- automatisch
- benutzerdefiniert

Timeline zeigt:

Original oder Proxy.

Finaler Export verwendet Originalmaterial.

Proxydateien im Cache.

Neu erzeugbar.

---

# 85. Untertitel

Unterstütze:

- lokale automatische Transkription
- manuelle Bearbeitung
- SRT Import
- SRT Export
- weitere sinnvolle Formate

Speaker Diarization:

- Sprecher 1
- Sprecher 2
- usw.

Erkannte Sprecher im Projekt benennbar.

Name im Projekt weiterverwenden.

---

# 86. Eingebrannte Untertitel

Optional ins Video rendern.

Gestaltung:

- Schrift
- Größe
- Position
- Farbe
- Hintergrund
- Wort-für-Wort-Hervorhebung

---

# 87. Szenenerkennung

Automatische Schnitt-/Szenenerkennung.

Auf Wunsch Clips an erkannten Grenzen aufteilen.

---

# 88. Stille entfernen

Stille und längere Sprechpausen erkennen.

Auf Wunsch automatisch entfernen.

Vor Anwendung Vorschau bzw. Schnittliste zeigen.

---

# 89. Auto Reframe

Unterstütze z. B.:

16:9 → 9:16.

Motiv/Person automatisch verfolgen.

Nutzer darf Zielmotiv ändern.

---

# 90. Objekttracking

Objekt markieren und verfolgen.

Tracking verwendbar für:

- Text
- Blur
- Masken
- Effekte

---

# 91. Video-Inpainting

Objekt markieren.

Tracking über Frames.

Objekt lokal entfernen.

Hintergrund rekonstruieren.

Zeitliche Konsistenz priorisieren.

---

# 92. Video-Freistellen

KI-Matting für Personen/Objekte.

Hintergrund:

- entfernen
- ersetzen
- transparent, wenn Ausgabeformat geeignet

---

# 93. Frame Interpolation

Unterstütze lokale KI-Interpolation.

Beispiele:

30 → 60 FPS.

30 → 120 FPS.

Slow-Motion.

---

# 94. Video-Restaurierung

Lokale Funktionen:

- Denoise
- Schärfen
- alte/komprimierte Videos verbessern
- geeignete Gesichtsrestaurierung

Zeitliche Konsistenz beachten.

---

# 95. Video-Stiltransfer

Vorhandenes Video mit geeignetem Modell in anderen Stil transformieren.

Zeitliche Konsistenz berücksichtigen.

---

# 96. Video-Audio

Pro Audiospur:

- Lautstärke
- Fade In/Out
- Normalisierung
- Equalizer
- Kompressor
- Rauschunterdrückung

Zusätzlich Stem-Separation vorsehen:

- Voice/Vocals
- Instrumental
- Drums
- Bass
- weitere unterstützte Stems

---

# 97. Videoexport

Unterstütze sinnvolle Container:

- MP4
- MKV
- WebM

Codecs abhängig von Runtime/Hardware.

Einstellbar:

- Auflösung
- FPS
- Bitrate/Qualität
- Audioformat
- Hardware-Encoding
- Farbmanagement

Presets:

- 1080p
- 4K
- Archiv
- Benutzerdefiniert

Keine Codecoption anbieten, wenn System sie nicht unterstützt.

---

# 98. LIPSYNC

Lipsync ist eigener Studio-Schnellbereich, benutzt gemeinsame Video-/Audioengine.

Unterstütze:

- Video + Audio
- Bild + Audio, wenn Modell unterstützt
- Gesichtsauswahl
- mehrere Gesichter
- Tracking
- Zeitversatz
- Längenanpassung

Keine Stimmenklonfunktion als Voraussetzung.

---

# 99. MUSIC und AUDIO

Music bietet lokale Musikgenerierung.

Modell wechselbar wie Image/Video.

Modellspezifische Parameter.

Soweit Modell unterstützt:

- Prompt
- Genre
- Stimmung
- Länge
- Seed
- Instrumentierung
- Varianten

Zusätzlich Audioeditor:

- Waveform
- Schneiden
- Mehrspur
- Lautstärke
- Fade
- EQ
- Kompressor
- Denoise
- Normalisierung
- Pitch
- Tempo
- Stem Separation

---

# 100. EXTEND

Extend ist kontextbezogen.

Unterstütze soweit echte Modelle vorhanden:

- Image Outpainting
- Video Fortsetzung
- Music/Audio Fortsetzung

Eine Fortsetzung muss wirklich neuen Inhalt erzeugen.

Nicht einfach bestehende Frames oder Audio wiederholen und als KI-Extend bezeichnen.

---

# 101. MOTION

Motion soll echte steuerbare Bewegungsfunktionen enthalten, soweit passende Modelle verfügbar sind.

Beispiele:

- Pose
- Referenzbewegung
- Kamera
- Motion Strength
- Motion Transfer

Nur Parameter anzeigen, die Modell wirklich verarbeitet.

---

# 102. Workflows

Studio-Unterbereich:

Workflows.

Kein zusätzlicher Hauptnavigationseintrag nötig.

## Standardansicht

einfache Schrittansicht.

Beispiel:

Image → Erase → Upscale → Export.

## Erweitert

optional Node-Editor.

Beide verwenden dieselbe interne Workflow-Graphstruktur.

Keine zwei getrennten Workflow-Systeme.

---

# 103. Workflow-Vorschläge

Assistent darf wiederkehrende Abläufe erkennen.

Nur Vorschlag:

„Du verwendest diesen Ablauf häufig. Als Workflow speichern?“

Nicht automatisch anlegen.

Nach Ablehnung nicht ständig erneut nerven.

Workflow-Vorschläge in Einstellungen deaktivierbar.

---

# 104. Workflow-Schritte

Ein Schritt kann:

- festes Modell
- automatische geeignete Modellauswahl

verwenden.

Parameter:

- fest
- variabel
- beim Start abfragen

---

# 105. Workflow-Datei

Export/import:

`.lsworkflow`

Enthält:

- Graph
- Werkzeuge
- Einstellungen
- Modellreferenzen
- Erweiterungsreferenzen

Keine Modelle einbetten.

Beim Import:

fehlende Modelle prüfen.

Option:

„Fehlendes installieren“.

Importierte Workflowdateien dürfen keinen beliebigen Code oder Shellbefehl enthalten.

Nur definierte Local-Studio-Nodes.

---

# 106. Startseite

Startseite soll nützlich sein, nicht dekorativ.

Mögliche Inhalte:

- letzte Galerieelemente
- letztes Projekt
- laufende Jobs
- häufige Werkzeuge
- favorisierte Workflows
- Updates
- Hardwarehinweis bei Problemen

Nicht zu viele Kacheln.

---

# 107. App schließen

Einstellung:

„Beim Schließen im Infobereich weiterlaufen“.

Standard:

AUS.

Wenn EIN:

Fenster schließen → Tray/Infobereich.

Jobs und Downloads laufen weiter.

Wenn AUS und aktive:

- Downloads
- Generierungen

dann vor Beenden warnen.

Zusätzlich im selben konsolidierten Exit-Dialog berücksichtigen:

- ungespeicherte Ergebnisse
- ungespeichertes Projekt
- projektlose Sitzung

Nicht mehrere nervige Dialoge nacheinander.

---

# 108. Recovery und ungespeicherte Sitzung

Wenn „letzten Arbeitsstand wiederherstellen“ aktiv:

ungespeicherte Ergebnisse erhalten.

Wenn Option deaktiviert und Nutzer regulär beendet:

nach Warnung verwerfen.

Crash Recovery bleibt trotzdem separat.

---

# 109. Sicherheitsregeln

Model Cards, README-Dateien und importierte Workflows sind nicht vertrauenswürdig.

Kein:

- automatisches `trust_remote_code=True`
- `eval`
- Shellcode aus Modellbeschreibungen
- automatische Installation beliebiger Python-Pakete eines Modells
- Ausführen fremder Skripte
- unkontrollierter Custom-Node-Marktplatz

Bevorzuge sichere Gewichtsformate und bekannte Adapter.

Dateiendung allein beweist Sicherheit nicht.

---

# 110. Dateisicherheit

Validiere:

- Pfade
- Archive
- Projektimporte
- Workflowimporte

Schütze gegen:

- Path Traversal
- Überschreiben fremder Dateien
- beschädigte Container
- untrusted embedded content

---

# 111. Modell- und Komponentenlizenzen

Zeige:

- Modelllizenz
- Code-/Runtime-Lizenz
- unbekannte Lizenz

Zugangsbeschränkungen nicht umgehen.

Gated Models:

offiziellen Hugging-Face-Zugang verwenden.

---

# 112. Ressourcenverwaltung

Große Modelle kontrolliert laden und entladen.

VRAM nach Modellwechsel sauber freigeben.

Schnelles Umschalten darf keine verwaisten Worker erzeugen.

Worker:

- Healthcheck
- Timeout
- kontrollierter Shutdown

Nach OOM:

sauberer Fehler.

UI bleibt bedienbar.

Keine automatische Qualitätsreduzierung ohne Hinweis.

---

# 113. Fehlerdarstellung

Nicht nur Python/CUDA-Traceback anzeigen.

Nutzerfreundliche Meldung.

Zusätzlich:

„Details“.

Wenn möglich konkrete Aktion:

- Modell reparieren
- fehlende Komponente installieren
- kleineren Wert verwenden
- Speicher freigeben
- Download erneut versuchen

---

# 114. Entwicklungsreihenfolge

Der Gesamtumfang dieses Dokuments bleibt vollständig erhalten.

Aber NICHT alles gleichzeitig halb implementieren.

Arbeite in folgender Reihenfolge.

## M0 — Core

- Tauri-App
- Navigation
- Einstellungen
- i18n
- Theme
- Datenbank
- Hardwareerkennung
- Worker-System
- Job-System
- Speicherpfade
- Logging
- Recovery-Grundlage

Abnahme:

echte Windows-App startet und persistiert Einstellungen.

## M1 — Modellmanager

- Hugging-Face-Suche
- OAuth
- Browserbereich
- Downloads
- Installation
- Modellscan
- Import
- Verschieben
- Updates
- Modellstatus

Abnahme:

echtes kleines Modell finden, herunterladen/importieren und sicher verwalten.

## M2 — Image End-to-End

Image ist erste vollständig fertige KI-Funktion.

- echtes Bildmodell
- Modellwechsel
- Text-to-Image
- Ergebnis
- temporäre Ergebnisleiste
- Galerie speichern
- Metadaten
- Benchmark

Abnahme:

echtes lokal erzeugtes Bild.

## M3 — Galerie und Projekte

- echte Ordner
- Viewer
- Tags
- Suche
- Papierkorb
- Versionen
- Herkunft
- Projekte
- Recovery

## M4 — Bildeditor

- Ebenen
- Masken
- Auswahl
- Edit
- Erase
- Cutout
- Upscale
- RAW
- HDR
- Batch
- Export

## M5 — Assistent

- Chatmodell
- Tool Calling
- Hardwareberatung
- Galerie-Metadaten
- Aktionen
- gelernte Präferenzen

## M6 — Vision/Analyse

- Bilder
- Video
- Audio
- optional installierte Analysemodelle
- semantischer Index

## M7 — Video Generation

- echtes Video
- mindestens zwei wechselbare Modelle derselben Unteraufgabe
- Animate
- Motion

## M8 — Timeline Editor

- Mehrspur
- Proxy
- Keyframes
- Effekte
- Untertitel
- Tracking
- Audio

## M9 — Advanced Video

- Inpainting
- Matting
- Reframe
- Interpolation
- Restore
- Style Transfer
- Lipsync
- Extend

## M10 — Music/Audio

- Musikgenerierung
- Audioeditor
- Stems
- Music Extend

## M11 — Workflows

- Schrittansicht
- Node-Editor
- Workflowdateien
- Workflow-Vorschläge

## M12 — Packaging

- Installer
- portable Version
- GitHub-Updater
- File Association
- Crash Tests
- Performance
- Accessibility
- finaler Windows-Test

---

# 115. Verbindliche Testfälle

Mindestens folgende real testen:

### Modellverwaltung

- Hugging Face suchen
- gated model ohne Zugang
- Login
- falscher Token
- Download
- Pause
- Resume
- Abbruch
- Netzwerkausfall
- voller Datenträger
- Update
- Verschieben
- fehlendes externes Modell

### Modellwechsel

A auswählen.

Generieren.

B auswählen.

Generieren.

A erneut auswählen.

A-Einstellungen wiederhergestellt.

Kein erneuter Gewichtsdownload.

### Job Snapshot

A läuft.

B auswählen.

B einreihen.

A bleibt A.

B läuft anschließend mit B.

### Galerie

- Drag-and-drop
- Strg+Drag kopieren
- Explorer-Datei hinzufügen
- Umbenennen
- Papierkorb
- Wiederherstellen
- Versionen
- Herkunft
- 18+-Blur

### Projekt

- Projekt speichern
- auf anderem PC öffnen
- fehlendes Modell erkennen
- Installation ablehnen
- Projekt trotzdem öffnen

### Recovery

- App während Arbeit hart beenden
- Recovery erkennen
- Arbeitsstand wiederherstellen

### Bild

- Text-to-Image
- Edit
- Erase
- Cutout Alpha
- Upscale
- Masken
- Ebenen
- Batch

### Video

- echtes Text-/Image-to-Video
- Modellwechsel
- Timeline
- Proxy
- Export
- Untertitel
- Tracking

### Audio

- Wiedergabe
- Bearbeitung
- Transkription
- Stem Separation
- Musikgenerierung

### Offline

Nach vollständig installierter Runtime Netzwerk blockieren.

Installierte lokale Funktionen müssen weiterlaufen.

---

# 116. Messbare UI-Anforderungen

Die Anwendung darf nicht durch den Funktionsumfang chaotisch werden.

Verwende progressive disclosure:

Standardansicht zeigt nur wichtige Optionen.

Erweiterte Parameter in:

„Erweitert“.

Seltener benötigte Features in kontextbezogenen Panels.

Modellbezogene Erweiterungen nicht in zentrale Modellliste werfen.

Image-Schnellwerkzeuge verwenden gemeinsame Bildengine.

Video-Schnellwerkzeuge verwenden gemeinsame Videoengine.

Galerie dauerhaft separat.

Downloads und Jobs separat.

Assistent separat.

Dadurch bleibt die App trotz großem Umfang übersichtlich.

---

# 117. Performance

Große Galerie:

virtualisierte Darstellung.

Thumbnails statt Originale.

Timeline:

Proxy-Support.

Große Modellbibliothek:

lazy metadata loading.

Modelle:

nicht unnötig gleichzeitig laden.

UI muss auch während Generierungen flüssig bleiben.

---

# 118. Dokumentation im Repository

Pflege:

`AGENTS.md`

kurz und eindeutig.

`SPEC.md`

dieses Produktziel.

`PLAN.md`

Meilensteine.

`PROJECT_STATUS.md`

aktueller Stand.

`MODEL_COMPATIBILITY.md`

getestete Modelle/Adapter.

`TEST_MATRIX.md`

Anforderung → Implementierung → Teststatus.

AGENTS.md nicht mit kompletter Spezifikation überladen.

Dort auf SPEC.md verweisen.

---

# 119. Statusregeln

Für jedes Feature getrennt festhalten:

- geplant
- implementiert
- getestet
- blockiert

„Implementiert“ und „getestet“ sind nicht dasselbe.

Keine Behauptung:

„fertig“

nur weil UI vorhanden ist.

Keine Behauptung:

„unterstützt“

nur weil eine Modellkarte einen passenden Tag besitzt.

---

# 120. Technische Entscheidungen, die Codex selbst treffen soll

Folgende Punkte sind bewusst NICHT auf ein konkretes Produkt/Modell festgenagelt:

- exaktes Starter-Chatmodell
- Vision-Modell
- Segmentierungsmodell
- Transkriptionsmodell
- Speaker-Diarization-Modell
- konkrete Bildmodelle
- konkrete Videomodelle
- konkrete Music-Modelle
- konkrete Upscaler
- konkrete Restaurierungsmodelle
- konkrete Backendbibliotheken pro Modellfamilie
- Codecimplementierung
- konkrete RAW-Bibliothek

Für jeden Kandidaten:

1. aktuelle Originalquelle prüfen
2. Lizenz prüfen
3. Windows-Kompatibilität prüfen
4. Hardwareanforderungen prüfen
5. sicheren Ladeweg prüfen
6. echten Test durchführen
7. Ergebnis dokumentieren

Keine veraltete Modellliste blind übernehmen.

---

# 121. Keine Scope-Reduktion

Der große Gesamtumfang dieses Dokuments ist beabsichtigt.

Wenn eine spätere Funktion noch nicht implementiert ist:

im Plan offen lassen.

Nicht kommentarlos streichen.

Aber:

keine 150 funktionslosen Buttons vorbauen.

Immer vertikale funktionierende Pfade fertigstellen.

---

# 122. Definition of Done

Local Studio ist erst vollständig fertig, wenn:

- Installer funktioniert
- portable Version funktioniert
- Modellverwaltung funktioniert
- Hugging-Face-Integration funktioniert
- Image vollständig funktioniert
- Bildeditor funktioniert
- Galerie funktioniert
- Projekte funktionieren
- Assistent funktioniert
- Video funktioniert
- Timeline funktioniert
- Audio/Music funktioniert
- Workflows funktionieren
- Recovery funktioniert
- Updates funktionieren
- 18+-Sperre funktioniert
- reale Modelle getestet wurden
- Tests dokumentiert wurden

Nicht erfüllte Bereiche bleiben sichtbar als offen.

---

# 123. Startauftrag

Beginne jetzt.

1. Prüfe das vorhandene Repository vollständig.
2. Lies AGENTS.md und bestehende Spezifikationen.
3. Erhalte bereits funktionierenden Code.
4. Überführe diese Spezifikation in `SPEC.md`, falls sie dort noch nicht vollständig enthalten ist.
5. Erzeuge bzw. aktualisiere `PLAN.md`, `PROJECT_STATUS.md`, `MODEL_COMPATIBILITY.md` und `TEST_MATRIX.md`.
6. Entferne widersprüchliche oder doppelte alte Anforderungen.
7. Baue keinen parallelen zweiten Stack für bereits vorhandene funktionierende Komponenten.
8. Beginne mit dem nächsten noch offenen Meilenstein.
9. Implementiere echte Funktionen und führe Tests aus.
10. Aktualisiere nach jedem Arbeitspaket den Projektstatus.

Arbeite selbstständig weiter, ohne nach jeder normalen Teilentscheidung erneut um Erlaubnis zu fragen.

Wenn eine technische Entscheidung nicht ausdrücklich vorgeschrieben ist, wähle die Lösung, die:

- am zuverlässigsten
- lokal
- wartbar
- sicher
- performant
- Windows-tauglich
- und für die vorhandene Hardware praktikabel

ist.

Keine Simulationen als fertige Funktionen ausgeben.

Keine erfundenen Testergebnisse.

Keine erfundene Modellkompatibilität.

Baue Local Studio schrittweise als tatsächlich funktionierende Windows-Anwendung.