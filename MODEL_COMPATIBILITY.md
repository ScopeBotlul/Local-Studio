# Modellkompatibilität

Noch kein Modell wurde mit einem realen lokalen Ausführungspfad integriert oder getestet. In 0.4.0 wurden ausschließlich kleine Testgewichte und eine Konfiguration aus `hf-internal-testing/tiny-random-gpt2` als Download-/Integritätsfixture heruntergeladen; sie wurden nicht geladen oder ausgeführt. Es gibt keine bestätigte Bild-, Video-, Audio-, Musik-, Vision- oder Chat-Kompatibilität. Hardwareerkennung und ein Datei-Hashjob sind keine Modelltests.

| Aufgabenbereich | Geplanter Meilenstein | Adapter/Runtime | Modell/Revision/Variante | Stand |
| --- | --- | --- | --- | --- |
| Kleiner DE/EN-Assistent, ca. 1–3B | Einrichtung M1, Ausführung M5 | llama.cpp/GGUF vorgesehen | Noch auszuwählen | Ungeprüft |
| Bildgenerierung | M2 | Noch auszuwählen | Noch auszuwählen | Ungeprüft |
| Segmentierung/Grounding, Erase, Cutout, Upscale, Restore | M4 | Noch auszuwählen | Noch auszuwählen | Ungeprüft |
| Bild-/Videoanalyse, Inhaltsindex | M6 | Noch auszuwählen | Noch auszuwählen | Ungeprüft |
| Transkription, Diarization, Audioanalyse | M6/M8 | Noch auszuwählen | Noch auszuwählen | Ungeprüft |
| Videoerzeugung/Motion, zwei Modelle derselben Unteraufgabe | M7 | Noch auszuwählen | Zwei reale Kandidaten erforderlich | Ungeprüft |
| Video-Inpainting, Matting, Interpolation, Restore, Lipsync, Extend | M9 | Noch auszuwählen | Noch auszuwählen | Ungeprüft |
| Musikgenerierung, Stems, Music-Extend | M10 | Noch auszuwählen | Noch auszuwählen | Ungeprüft |

Für jeden Kandidaten sind vor einer Kompatibilitätsaussage aktuelle Originalquelle, Modell- und Runtime-Lizenz, exakte Revision/Dateien/Hashes, Windows-Unterstützung, sichere Ladeart und Hardwareanforderungen zu dokumentieren. Danach: tatsächliche Ein-/Ausgabe, Parameterwirkung, Laden/Entladen, Abbruch, Speicherfreigabe, Fehlerverhalten und Offline-Ausführung prüfen. Messwerte bleiben lokal.

Der spätere Nachweis enthält Datum, App-/Adapter-/Runtime-Version, Hardware/VRAM/Treiber, Aufgabe, Modelleinstellungen, Ergebnisdatei, Laufzeit, Ressourcenwerte, bekannte Grenzen und Testreferenz. Repo-Tags, Dateiendungen oder eine vorhandene Model Card reichen nicht. AMD/Intel erst dann als unterstützt kennzeichnen, wenn der konkrete Backendpfad getestet ist. Ein reiner Download ohne Adapter bleibt ausdrücklich „verwaltbar, Ausführung nicht unterstützt“.
