# Modellkompatibilität

Ab 0.6.0 ist der erste lokale Bildpfad mit zwei vorhandenen SDXL-Safetensors-Checkpoints tatsächlich ausgeführt. Der Nachweis gilt für die unten genannten Bytes, Parameter und Hardware, nicht pauschal für sämtliche SDXL-Dateien. Frühere `hf-internal-testing/tiny-random-gpt2`-Dateien bleiben ausschließlich Download-/Importfixtures; kein GPT-2-Chatadapter wurde integriert. KI-Video, Musik und Vision bleiben ungeprüft. Lokaler Qwen-Chat und Whisper-Transkription wurden ab 0.24.0 tatsächlich ausgeführt; siehe Ergänzung unten.

| Aufgabenbereich | Geplanter Meilenstein | Adapter/Runtime | Modell/Revision/Variante | Stand |
| --- | --- | --- | --- | --- |
| Kleiner DE/EN-Assistent, ca. 1–3B | Einrichtung M1, Ausführung M5 | llama.cpp b11026 / CPU | Qwen2.5 1.5B Instruct Q4_K_M, gepinnter Hash unten | Echte DE/EN-Inferenz und begrenzte Tools ab 0.24.0 geprüft |
| Bildgenerierung | M2 | stable-diffusion.cpp cc515a0 · Windows Vulkan | Juggernaut XL v9; DreamShaper XL Turbo V2, konkrete lokale Hashes unten | Reale 512×512-Smoke-Läufe bestanden; vollständige Abnahme offen |
| Segmentierung/Grounding, Erase, Cutout, Upscale, Restore | M4 | Noch auszuwählen | Noch auszuwählen | Ungeprüft |
| Bild-/Videoanalyse, Inhaltsindex | M6 | Noch auszuwählen | Noch auszuwählen | Ungeprüft |
| Transkription; Diarization und weitere Audioanalyse offen | M6/M8 | whisper.cpp b5130 / CPU | Whisper Base multilingual, gepinnter Hash unten | Echte DE/EN-Transkription ab 0.24.0 geprüft; keine Diarization-Abnahme |
| Videoerzeugung/Motion, zwei Modelle derselben Unteraufgabe | M7 | Noch auszuwählen | Zwei reale Kandidaten erforderlich | Ungeprüft |
| Video-Inpainting, Matting, Interpolation, Restore, Lipsync, Extend | M9 | Noch auszuwählen | Noch auszuwählen | Ungeprüft |
| Musikgenerierung, Stems, Music-Extend | M10 | Noch auszuwählen | Noch auszuwählen | Ungeprüft |

Für jeden Kandidaten sind vor einer Kompatibilitätsaussage aktuelle Originalquelle, Modell- und Runtime-Lizenz, exakte Revision/Dateien/Hashes, Windows-Unterstützung, sichere Ladeart und Hardwareanforderungen zu dokumentieren. Danach: tatsächliche Ein-/Ausgabe, Parameterwirkung, Laden/Entladen, Abbruch, Speicherfreigabe, Fehlerverhalten und Offline-Ausführung prüfen. Messwerte bleiben lokal.

Ein vollständiger Nachweis enthält Datum, App-/Adapter-/Runtime-Version, Hardware/VRAM/Treiber, Aufgabe, Modelleinstellungen, Ergebnisdatei, Laufzeit, Ressourcenwerte, bekannte Grenzen und Testreferenz. Repo-Tags, Dateiendungen oder eine vorhandene Model Card reichen nicht. AMD/Intel erst dann als unterstützt kennzeichnen, wenn der konkrete Backendpfad getestet ist. Ein reiner Download ohne Adapter bleibt ausdrücklich „verwaltbar, Ausführung nicht unterstützt“.

## Konkreter SDXL-Ausführungsnachweis 0.6.0

17. September 2026, Windows 11 x64, AMD Ryzen 9 9950X3D, NVIDIA RTX 4080 (16376 MiB laut NVIDIA-Erkennung), Treiber 616.56. Runtime: [master-872-cc515a0, Windows Vulkan](https://github.com/leejet/stable-diffusion.cpp/releases/tag/master-872-cc515a0), Archiv-SHA-256 `b8c6538f8948dfaa1adc25c463fb1617098d1ff307032e38f29648d5891ead8d`. Exakte Runtime-Dateihashes in `src-tauri/image-runtime.json`, Runtime-Lizenz MIT, weitere Bibliothekslizenzhinweise mitgeliefert.

| Lokale Datei | Bytes | SHA-256 | Parameter | Gesamtdauer |
| --- | ---: | --- | --- | ---: |
| Juggernaut-XL_v9_RunDiffusionPhoto_v2.safetensors | 7105348188 | `c9e3e68f89b8e38689e1097d4be4573cf308de4e3fd044c64ca697bdb4aa8bca` | 512×512, 10 Schritte, CFG 5, euler/Karras, Seed 42 | 18.05 s |
| DreamShaperXL_Turbo_V2.safetensors | 6939220250 | `6e718af03dcb651250ec8911554c4df8472a6d7027bcb3678a206c1170cc94fc` | 512×512, 4 Schritte, CFG 2, dpm++2m/Karras, Seed 42 | 16.21 s |
| Juggernaut-XL_v9_RunDiffusionPhoto_v2.safetensors | 7105348188 | `c9e3e68f89b8e38689e1097d4be4573cf308de4e3fd044c64ca697bdb4aa8bca` | 512×512, 2 Schritte, CFG 5, euler/Karras, Seed 42 | 21.12 s |

Die Zeiten umfassen Hashprüfung, Modellladen und Generierung bei vorhandenem Dateicache; keine kontrollierten Benchmarks und keine Peak-VRAM-Messung. Drei echte gültige PNGs, Modellwechsel A/B/A, unveränderliche Auftragsdaten, Beenden nach Abschluss, Abbruch während Denoising und Kindprozessende nach App-Abbruch sind nachgewiesen. Modelle wurden nur gelesen, nicht verändert oder erneut heruntergeladen. Bild/Prompt/Metadaten blieben lokal. Vollständig deaktiviertes Netzwerk wurde nicht separat getestet.

[Native Prüfergebnisse](.artifacts/native-1789665002625/report.json), [Auftragsparameter/Hashes/Logs](.artifacts/native-1789665002625/image-results.json), [gespeicherte Galerie nach Neustart](.artifacts/native-1789665002625/image-gallery-restart.png). Die Bilder der ersten erfolgreichen Läufe wurden visuell geprüft: erkennbare Landschaften; der kurze DreamShaper-Test zeigt Detailartefakte und ist kein Qualitätsprofil. Die 2-/4-Schritte-Läufe dienen dem Modellwechseltest.

Die lokalen Dateinamen entsprechen den Kandidaten [RunDiffusion/Juggernaut-XL-v9](https://huggingface.co/RunDiffusion/Juggernaut-XL-v9) und [Lykon/dreamshaper-xl-v2-turbo](https://huggingface.co/Lykon/dreamshaper-xl-v2-turbo). Downloadrevision und Herkunft dieser bereits vorhandenen Dateien wurden nicht verbindlich zugeordnet; ihre SHA-256-Identität ist oben festgehalten. Die App kennzeichnet die Modelllizenz deshalb weiter als **unbekannt**. Keine Modellweitergabe im Paket und keine Lizenzfreigabe für jede Nutzungsart behauptet. Bedingungen der tatsächlichen Bezugsquelle beachten.

Offen bleiben insbesondere Parameter-/Qualitätsabnahme bei 768/1024 Pixeln, getrennte Spitzen-Speichermessung, OOM-/Plattenfehler, weitere Treiber/Grafikkarten, Betrieb mit komplett deaktiviertem Netz, alle weiteren Bildarchitekturen, 18+-Sperre und vollständige M2-Benchmark-/Medien-Recovery-Anforderungen. [Adapterumfang](docs/IMAGE_GENERATION.md).

## Ergänzung 0.8.0: reale sequenzielle Aufträge

[Warteschlangennachweis](.artifacts/native-1789680730133/queue-results.json): Juggernaut XL mit 1024×1024, 60 Schritten und Euler/Karras; danach DreamShaper XL mit 512×512, 4 Schritten und DPM++ 2M/Karras, jeweils NVIDIA RTX 4080/Vulkan. Unterschiedliche Modelldigests und unveränderliche Eingaben bestätigt. Zusätzliche Crash-/Resume-Ausführung des wartenden DreamShaper-Auftrags erfolgreich. Kurze Funktionsprüfung, keine Qualitätsbewertung oder neue allgemeine Kompatibilitätszusage.

## Nachweis 0.24.0: lokaler Chat und Transkription

Qwen2.5 1.5B Instruct Q4_K_M (Apache-2.0) und Whisper Base multilingual (MIT), exakte Revisionen/Hashes in src-tauri/ai-models.json. llama.cpp b11026 und whisper.cpp b5130, offizielle Windows-x64-CPU-Runtimes mit Dateimanifesten. Reale DE/EN-Antworten, Hardware-/Galerie-/Bildvorbereitungs-Toolaufrufe, Laden/Entladen/Abbruch und Prozessende geprüft; keine allgemeine Modell-Qualitätszusage. Echte DE/EN-WAVs transkribiert, Zeiten/Timeline/SRT/Undo/Projekttransport geprüft. [Native Prüfung](.artifacts/native-1789715748322/report.json). GPU-Betrieb dieser Adapter, Diarization und vollständige Netzsperren-Abnahme bleiben offen.


## SDXL-Referenzen und Inpainting 0.25.0

[Tatsächliche Bildprüfung](.artifacts/native-1789720949322/report.json): dieselben oben dokumentierten DreamShaper-XL-Turbo-V2- und Juggernaut-XL-v9-Hashes, stable-diffusion.cpp cc515a0, Windows 11/RTX 4080. Bild-zu-Bild mit 512×512, 6 angeforderten Schritten, Euler/Karras, CFG 5, Stärke 0,8 bzw. 0,65 und bewusst aktivierter CPU-VAE. DreamShaper-Stapel Seeds 42/43 liefert unterschiedliche echte Ausgaben; Juggernaut als zweite Modellvariante ausgeführt. Maskiertes DreamShaper-Inpainting verändert den weißen Bereich, während alle 131.072 Pixel im schwarzen Halbteil exakt erhalten bleiben. Dies ist ein Ausführungs-/Parameter-/Originalschutznachweis, keine pauschale ästhetische oder allgemeine Modellfreigabe.

Ein all-GPU-Referenzlauf scheiterte bei belegtem VRAM. CPU-VAE ist eine explizite Option; keine automatische Verkleinerung oder Beendigung anderer GPU-Anwendungen. Alle Testbilder sind Entwicklungsfixtures. Modelle wurden nicht mit der App gebündelt oder an externe Dienste übertragen.
