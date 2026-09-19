# Modellbibliothek – lokaler Build 0.27.0

Im lokalen Build 0.27.0 enthalten. Klassifizierung ist mit Frontend-/Rust-Tests und nativen Bibliotheks-Fixtures geprüft; das ist keine universelle Erkennungs- oder Ausführbarkeitsgarantie. Konkrete Nachweise und der RAM-bedingt offene Generierungstest stehen in TEST_MATRIX.md. In diesem Auftrag wurde kein GitHub-Release veröffentlicht.

Die Standardsicht zeigt Modelle nach Einsatzzweck: Bild, Video, Sprache sowie Audio/Musik. LoRAs und ControlNet stehen unter Erweiterungen; Encoder, VAEs und Vorschau-Autoencoder unter Technische Komponenten. Unklare Dateien bleiben in einem eigenen Bereich erreichbar. Die Dateien werden nicht gelöscht, verschoben oder automatisch heruntergeladen.

## Erkennung und Unterstützung

Safetensors-Tensornamen und bestimmte SDXL-Tensorformen bestimmen die erkennbaren Rollen. Vorhandene Metadaten ergänzen Familie/Zweck und bei passenden LoRA-Angaben die Basisfamilie. Ein bekannter Dateiname oder Modell-Unterordner liefert nur eine vorläufige Zuordnung. Widersprüchliche oder unbekannte Kombinationen sind keine Ausführungsfreigabe.

GGUF wird anhand der begrenzt gelesenen Architekturmetadaten eingeordnet, nicht allein anhand der Dateiendung. Grundlage des Readers ist die [GGUF-Spezifikation](https://github.com/ggml-org/ggml/blob/master/docs/gguf.md). Die Dateiorganisation kann ein einzelner Checkpoint, ein verteilter Gewichtsindex oder ein Modellteil mit weiteren Abhängigkeiten sein; siehe auch [Diffusers-Modellformate](https://huggingface.co/docs/diffusers/main/en/using-diffusers/other-formats).

Der Status „SDXL unterstützt · Vorprüfung im Studio“ setzt die bekannte vollständige SDXL-Struktur voraus. GPU, Runtime und tatsächliche Ausführung werden weiterhin separat geprüft. Die allgemeine Datei-Strukturprüfung bleibt neben der Familien-/Rollenbeschreibung sichtbar. Nur Metadaten oder Namen führen nie zu „bereit“.

## Zuordnung von Bestandteilen

Beim unterstützten SDXL-Checkpoint erkennt die App UNet, CLIP-L, CLIP-G und VAE in derselben Datei. Der vorhandene Worker verwendet diese eingebetteten Bestandteile automatisch. Die Bibliothek zeigt sie einzeln an.

Bei erkannten partiellen SDXL-Dateien werden fehlende eingebettete Teile benannt. „Nicht im Checkpoint enthalten“ bedeutet nicht, dass eine solche Datei nirgends auf dem PC liegt. Lose Komponenten werden erst mit einem dafür implementierten Adapter kombiniert. Es gibt noch keine allgemeine, modellübergreifende Zuordnung externer Encoder/VAEs und keine Ausführung von LoRA/ControlNet/FLUX/Wan/Z-Image.

## Grenzen

- Die Erkennung umfasst bekannte Muster, nicht alle Modellarchitekturen. Unklare Dateien bleiben sichtbar.
- Metadaten sind Herstellerangaben, keine Sicherheits- oder Kompatibilitätsgarantie.
- Die Profilbeschreibung ist auf lokale Metadaten begrenzt und hat ein Lesebudget; große Bibliotheken werden in mehreren Aktualisierungen beschrieben.
- Der Anzeige-Cache ist keine Integritätsprüfung. Modellzulassung, Datenschutzsperre und Worker-Prüfungen bleiben maßgeblich.
- Vollständige Regression, echte UI-Prüfung und Build stehen wegen der ausdrücklichen Nutzeranweisung noch aus; siehe TEST_MATRIX.md.
