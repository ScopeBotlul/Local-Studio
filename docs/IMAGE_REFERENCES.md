# Bildreferenzen, Masken und Stapel ab 0.25.0

## Bild verändern

Im Studio unter **Bildgenerierung** einen unterstützten SDXL-Checkpoint wählen. **Referenzbild wählen** übernimmt eine statische 8-Bit-PNG-Datei mit 512, 768 oder 1024 Pixeln pro Achse. Der Prompt beschreibt das gewünschte Ergebnis. Die Veränderungsstärke reicht von 0,05 bis 1: höhere Werte verändern die Referenz stärker. Ohne Referenz entsteht weiterhin ein Bild allein aus dem Prompt.

In der Galerie kann eine PNG-Datei mit **Als Bildreferenz verwenden** ins Studio übernommen werden. Prompt und explizite Modellauswahl bleiben erhalten; es startet keine Generierung. Nicht passende Abmessungen oder Formate werden erklärt, nicht automatisch herunterskaliert.

**VAE auf CPU ausführen** verlagert die Bildkodierung und -dekodierung auf die CPU. Das spart GPU-Speicher und kann deutlich länger dauern. Diese Einstellung wird bewusst gewählt und mit Auftrag, Projekt und Messung gespeichert. Eine erschöpfte GPU wird nicht durch heimliches Ändern von Auflösung oder Qualität umgangen.

## Maskiert bearbeiten

Mit **Inpainting-Maske wählen** eine undurchsichtige PNG-Maske in derselben Größe auswählen. Alle RGB-Kanäle müssen denselben Grauwert haben. Weiß wird bearbeitet, Schwarz bleibt pixelgenau erhalten, Grau blendet das neue Ergebnis anteilig ein. Eine vollständig schwarze, farbige oder transparente Maske wird abgewiesen. Das tatsächliche SDXL-Ergebnis wird anhand der Maske mit dem Original zusammengesetzt; außerhalb der Maske bleiben auch die ursprünglichen Alphawerte erhalten.

Im Ebenen-Bildeditor lässt sich eine gezeichnete Maske mit **Maske als PNG exportieren** in die Galerie schreiben. Der Export verwendet dieselbe Maskenberechnung wie die Vorschau. Er entspricht der unverzerrten, bereits klassisch bearbeiteten Quellbildgröße der ausgewählten Ebene, nicht der gesamten Komposition. Die Maske bei Bedarf vorher umkehren: im Ebeneneditor bedeutet Schwarz ausblenden, beim Inpainting bedeutet Weiß bearbeiten. Originalbild, Ebene und Maskenrezept bleiben erhalten.

## Stapel

**Bilder im Stapel** erlaubt 1 bis 20 Bilder, begrenzt durch freie Warteschlangenplätze. Entweder denselben Seed für alle verwenden oder den Startseed pro Bild um eins erhöhen. Der höchste Seed ist 4294967295. Jeder Auftrag hat seine eigene unveränderliche Eingabe und sein eigenes Ergebnis. Batchnummer und Seed erscheinen in der Auftragsliste. Bei einem Fehler während des Einreihens werden bereits angelegte Aufträge und die konkrete Teilmenge angezeigt.

Eine große Berechnung läuft standardmäßig gleichzeitig. Die optionale parallele Verarbeitung erlaubt höchstens zwei rechenintensive Vorgänge, davon höchstens einen auf der GPU; RAM und Reserven werden vor dem Start geprüft. Bildstapel bleiben sequentiell. Wartende Vorgänge können abgebrochen werden.

## Dateien, Projekte und Wiederherstellung

Vor dem Einreihen erhält jeder Auftrag eine geprüfte eigene Kopie der Referenz und der Maske. Änderungen am ursprünglichen Bild verändern einen bereits angelegten Auftrag nicht. Die Wiederherstellung von Auftragsparametern verweist auf diese Arbeitskopien. Ungenutzte Arbeitskopien unterliegen der konfigurierten Temporärdatenfrist; aktuell oder pro Modell gemerkte Referenzen bleiben geschützt. Für dauerhafte Wiederverwendung den Arbeitsstand als Projekt speichern.

Projektformat 4 bettet Referenz und Maske mit SHA-256 ein. Nach dem Öffnen werden die eingebetteten Dateien verwendet; externe Ursprungsdateien werden nicht benötigt. Modellgewichte bleiben extern. Projekte der Formate 1–3 sind weiter lesbar; Projekte mit diesen neuen Bildparametern benötigen 0.25.0 oder neuer.

Abbruch und Prozessende hinterlassen nachvollziehbare Auftragszustände. Eine unterbrochene Inferenz wird nicht als vom Sampling-Zwischenstand fortsetzbar ausgegeben.

## Messwerte und Grenzen

Unter **Modelle → Lokal gespeichert → Gemessene Leistung** erscheinen tatsächliche erfolgreiche Läufe mit Modellhash, Parametern, Laufzeit und maximalem zugesichertem Windows-Worker-Speicher. Die Wartezeit auf Ressourcen gehört nicht zur Laufzeit. Eine unbekannte VRAM-Spitze wird als unbekannt angezeigt. Prompts werden nicht in der Benchmark-Datenbank gespeichert.

Unter **Einstellungen → Gelernte Präferenzen** lassen sich pro Modell gelernte Parameter ändern oder löschen. Eigene Änderungen werden beim weiteren Lernen beibehalten. Der lokale Assistent kann Messwerte und Präferenzen über begrenzte Lese-Werkzeuge verwenden. Sie ändern keine aktuelle Nutzerauswahl automatisch.

Geprüft sind konkrete lokale SDXL-Checkpoints auf Windows mit RTX 4080; siehe MODEL_COMPATIBILITY.md und TEST_MATRIX.md. Diese Pfade sind keine allgemeine Modell-, Farbmanagement-, RAW/HDR-, LoRA-, ControlNet- oder Video-KI-Abnahme. Der gesamte weitere Masterprompt bleibt in REQUIREMENTS_STATUS.md erhalten.

## Reproduzierbare Entwicklerprüfungen

Nach dem Desktop-Build in PowerShell `LOCAL_STUDIO_TEST_EXE` auf die absolute Release-EXE setzen. `node scripts/native-25.mjs core|references|models|editors|ai|menu` führt jeweils genau eine Suite in einem neuen isolierten Ordner unter `.artifacts` aus. Die Suite wird ohne `|` ausgewählt, beispielsweise `node scripts/native-25.mjs editors`. Der Starter kopiert die Programmdatei und Runtimes und aktiviert den Test-Debugport nur für diesen Kindprozess.

`references` und `models` benötigen die in den Tests genannten lokalen SDXL-Checkpoints; `models` zusätzlich das gepinnte Qwen-GGUF. `ai` benötigt Qwen und die über `LOCAL_STUDIO_TEST_SPEECH_FIXTURES` angegebenen DE/EN-WAV-Testdateien; das kleine Whisper-Modell wird dabei bewusst über den echten Downloadmanager geladen. Modelle bleiben außerhalb des Repositories. GPU-Suites nicht gleichzeitig in mehreren App-Instanzen starten. Die Detailberichte enthalten tatsächliche Ergebnisse; Dateidialoge werden kontrolliert, Inferenz, Dateien, Installer und Pixel bleiben echt.
