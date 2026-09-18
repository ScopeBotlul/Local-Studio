# Lokale Modellreferenzen ab 0.5.0

Unter **Modelle → Lokal gespeichert → Modelle vom PC einbinden** einen lokalen
Ordner auswählen oder seinen absoluten Pfad eingeben und **Ordner durchsuchen** starten.
Der manuelle Lauf durchsucht Unterordner. Er verändert, verschiebt und kopiert keine
Modelldateien. Nur Verweise und Prüfergebnisse landen in `config/model-library.sqlite3`.
Es gibt keine Netzwerkanfrage und keine Ausführung von Modellcode.

Ab **0.5.1** startet der separate Button **Vollständige Suche** alle von Windows
gemeldeten lokalen Laufwerksbuchstaben. Die API unterscheidet lokale interne,
Wechsel-, optische und RAM-Laufwerke von gemappten Netzlaufwerken; letztere sind
ausgeschlossen. Unlesbare oder inzwischen fehlende Laufwerke werden übersprungen.
Die UI zeigt Ziellaufwerke, bearbeitete Laufwerke, aktuellen Pfad und echte Zähler.
Kein automatischer Start; **Suche abbrechen** verwirft Teilergebnisse des laufenden
Scans. Ein zweiter Scan kann währenddessen nicht gestartet werden.
Vollständig bezieht sich auf den Suchbereich, nicht auf die Modellvalidierung.

Ab **0.6.0** durchsucht **Schnellsuche** typische vorhandene Modellordner statt ganzer Laufwerke: den eingestellten Modellpfad, Hugging-Face-Caches einschließlich `HF_HUB_CACHE`/`HF_HOME`, LM-Studio-Ablagen, Models/Modelle-Ordner im Profil und auf lokalen Laufwerken sowie typische ComfyUI-/Stable-Diffusion-WebUI-Pfade. Nicht existierende und überlappende Suchwurzeln werden entfernt. Benutzerdefinierte Ablagen werden weiterhin über die Ordnersuche erreicht. Schnellsuche hat dieselben Ressourcenlimits wie die Ordnersuche.

**Im Studio prüfen** übernimmt eine Safetensors-Datei in den ersten SDXL-Adapter. Dessen gesonderte Vorprüfung und reale Ausführung sind unter [Bildgenerierung](IMAGE_GENERATION.md) beschrieben. Der Importstatus allein bleibt unverbindlich.

## Trefferauswahl ab 0.6.1

Standardmäßig erscheinen **Erkannte Modellformate**: strukturell erkannte Safetensors mit üblichen Gewichts-/Bias-/LoRA-Schlüsseln, erkannte GGUF-Köpfe und entsprechend geprüfte Gewichtsindizes. Die Liste bestätigt weder ein vollständiges Basismodell noch Inferenzunterstützung. **Unbestätigte Kandidaten** enthalten unter anderem ONNX-/binäre PyTorch-Kandidaten, unvollständige Indizes, beschädigte neue Funde und reine Tensorcontainer ohne bekannte Gewichtsschlüssel. Unbekannte Gewichts-Namensschemata können dort erscheinen. Kleine Dateien werden nicht aufgrund ihrer Größe ausgeschlossen.

Schnell- und Vollsuche überspringen `$Recycle.Bin`, `System Volume Information`, Windows/Program Files/Program Files (x86) direkt unter einer Laufwerkswurzel sowie `site-packages`, `dist-packages`, `node_modules`, `.git`, `.svn`, `__pycache__` und `.artifacts` als vollständige Pfadkomponenten. Diese bewussten Ausschlüsse zählen als übersprungene Pfade mit konkretem Grund, nicht als erreichte Suchgrenze. Die gezielte **Ordnersuche** kann solche bewusst gewählten Verzeichnisse prüfen. Ein Dateianfang aus reinem UTF-8-Text mit `.pt`/`.pth`/`.ckpt` oder passendem PyTorch-`.bin`-Namen wird nicht als binärer Checkpoint aufgenommen. Dabei wird kein Inhalt ausgeführt.

Alte Bibliotheken werden einmalig und transaktional eingeordnet. **Ausgeblendete alte Treffer** bleiben zur Kontrolle erhalten; es werden keine Referenzen oder Quelldateien gelöscht. Alte Vollsuchen sind an ihrem gespeicherten Laufwerkswurzelpfad erkennbar. Die Anzeige hat höchstens 50 Karten je Seite. Nach einem portablen Update im bisherigen Ordner mit erhaltenem `Local-Studio-Data` wird die vorhandene Liste ohne neue PC-Vollsuche eingeordnet.

## Erkennung und Prüfumfang

| Format | Prüfung | Grenze |
| --- | --- | --- |
| Safetensors | Begrenzter JSON-Header, eindeutige Tensor-Schlüssel, Formen, bekannte Datentypen, zusammenhängende Bytebereiche und passende Dateilänge | Kein Hashvergleich oder Beweis eines vollständigen Basismodells; unbekannte Datentypen bleiben unbestätigt |
| Safetensors-Gewichtsindex | Sichere relative Dateinamen, alle genannten Teil-Dateien vorhanden, Strukturprüfung der verfügbaren Teil-Dateien | Gewichtsindex bestätigt weder weitere Runtime-Dateien noch Tensor-Semantik oder Übereinstimmung aller Index-Tensornamen |
| PyTorch-Gewichtsindex | Sichere Dateinamen und Vorhandensein der Teil-Dateien | Keine Pickle-Deserialisierung; Kandidat bleibt ungeprüft |
| GGUF | Magic und Version 2/3 im 24-Byte-Kopf | Tensor-Metadaten, Inhalt und mehrteilige GGUF-Modelle sind noch nicht vollständig geprüft |
| ONNX, CKPT, PT, PTH, pytorch_model*.bin | Dateiendung/-name und lokale Größe | Ungeprüfte Kandidaten, kein Laden/Deserialisieren |

`config.json` im gleichen Ordner kann einen unverbindlichen Familiennamen liefern.
Erkannte LoRA-Schlüssel/-Metadaten werden als Zusatzkomponente beschriftet; alle
übrigen Funde sind Modellkandidaten, kein automatisch bestätigtes Basismodell.
Ohne integrierten Modelladapter wird kein Fund als ausführbar angezeigt.
Lizenz und Runtime-Vollständigkeit sind unbekannt. Größen beziehen sich auf vorhandene
erfasste Dateien, einschließlich Indexdatei; fehlende Dateien werden nicht mit erfundenen
Größen addiert. Die Darstellung verwendet dezimale GB.

## Lebenszyklus und Grenzen

- Wiederholte Scans aktualisieren dieselbe kanonische Pfadreferenz. Teil-Dateien aus
  einem erkannten Index erscheinen unter dem Index statt als einzelne Modelle.
- Während die lokale Ansicht offen ist, prüft sie ungefähr alle zwei Sekunden bekannte
  Dateipfade, Größe und Änderungszeit. Fehlende/veränderte Dateien werden kenntlich gemacht.
  Neue Dateien werden nur durch einen neuen manuellen Scan entdeckt.
- **Erneut prüfen** liest die begrenzten Metadaten der bekannten Referenz erneut.
  **Aus Liste entfernen** löscht ausschließlich die Datenbankreferenz.
- **Suche abbrechen** übernimmt keine Ergebnisse des abgebrochenen Laufs. Frühere Einträge
  bleiben erhalten. Nach einem Prozessabbruch wird ein laufender Scan als unterbrochen
  angezeigt; es erfolgt kein automatischer Neustart.
- Ein Scan läuft außerhalb des UI-Threads; Dateioperationen bleiben in Rust.
  Der normale App-Abschluss bricht ihn ab und wartet auf sein Ende.
- Ordnersuche: 100.000 besuchte Pfade, 2.000 Funde und 32 Unterordnerebenen.
  Vollsuche: keine feste Pfadzahlbegrenzung, 10.000 Funde, 128 Unterordnerebenen.
  Beide Modi: höchstens 20.000 Dateireferenzen und 50 konkrete Pfadhinweise.
  Weitere übersprungene Pfade zählen im Gesamtzähler mit. Erreichte Ressourcenlimits
  erscheinen ausdrücklich als **Suche unvollständig · Grenze erreicht**.
  Die Traversierung hält nur Iteratoren entlang des aktuellen Verzeichnispfads.
- Höchstens 16 MiB pro Header/Index, 1 MiB für `config.json`, 2.048 Teil-Dateien pro Index,
  insgesamt höchstens 64 MiB Safetensors-Header pro Eintrag. Grenzen werden nicht als
  erfolgreiche Strukturprüfung ausgegeben.
- Nur lokale absolute Laufwerkspfade. Junctions und Reparse-Points werden übersprungen.
  Einzige Dateisymlink-Ausnahme ist ein normaler HF-Snapshot unter `models--…/snapshots/{40-stelliger Hex-Commit}`
  mit Ziel als direktem 40-/64-stelligem Hex-Dateinamen in `blobs` desselben Repositories.
  Verzeichnislinks, andere Repositories und beliebige Linkziele bleiben ausgeschlossen; die Dateiöffnung folgt keinem neu gesetzten Leaf-Link.
  Das ist kein vollständiger Schutz gegen einen lokalen Angreifer, der gleichzeitig
  übergeordnete Verzeichnisse austauscht. Scan-Ausgaben bleiben rein lokal.
- Die Ersteinrichtungsfrage, Volumes ohne
  Laufwerksbuchstaben, Laufwerkstests mit echter Trennung, Verschieben, Modell-/Runtime-Reparatur und
  vollständige Modellklassifikation bleiben offen. Kein M1-Komplettabschluss.

Alle sieben Modellbibliotheks-IPC-Commands sind ausschließlich der lokalen Hauptansicht zugeordnet.
Der externe HF-WebView erhält keine Modellbibliotheks- oder Scanberechtigung.

Prüfnachweise: [Testmatrix](../TEST_MATRIX.md). Technische Quellen:
[Windows-Laufwerksliste](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getlogicaldrives),
[Windows-Laufwerkstypen](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getdrivetypew),
[Safetensors-Format](https://github.com/safetensors/safetensors),
[GGUF-Spezifikation](https://github.com/ggml-org/ggml/blob/master/docs/gguf.md).
