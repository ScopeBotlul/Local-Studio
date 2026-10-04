# Lokales Video-Studio ab 0.37.0

Unter Studio > Video erstellen stehen **Prompt zu Video** und **Bild zu Video** bereit. Links werden Modell, Engine und Parameter gewählt, in der Mitte wird das Ergebnis als abspielbares MP4 angezeigt, rechts liegt die gemeinsame Galerie.

- Prompt zu Video braucht einen Prompt und ein T2V-/TI2V-Modell.
- Bild zu Video braucht ein Referenzbild und ein I2V-/TI2V-Modell. Ein zusätzlicher Bewegungsprompt ist optional. Bildauswahl übernimmt passende Abmessungen; Bild entfernen wechselt zurück zu Prompt zu Video.
- Breite/Höhe: 128–2048 Pixel in 32er-Schritten. Frames: 5–81 in 4er-Schritten. FPS: 1–50. Die Dauer ist Frames/FPS; höhere FPS erzeugen keine zusätzlichen Frames. Schritte, Guidance, negativer Prompt und fester oder zufälliger Seed sind einstellbar. Die Grenzen garantieren nicht, dass jedes Gerät genug Speicher hat.
- Die Generierung läuft weiter, wenn ein anderer Arbeitsbereich geöffnet wird. Abbrechen beendet den eigenen Worker bzw. den eigenen ComfyUI-Auftrag. Phasen und Laufzeit werden angezeigt; nicht messbare Inferenz wird nicht als Prozentfortschritt dargestellt.
- Ergebnisse bleiben zunächst temporär. Erst In Galerie speichern veröffentlicht ein MP4 unter `Local-Studio-<UUID>.mp4`. Verwerfen entfernt das temporäre Ergebnis. Ungespeicherte Videos werden beim Beenden berücksichtigt.
- Modell, Eingabemodus, Prompts, Abmessungen, Frames, FPS und Seed bleiben als lokale MP4-Metadaten erhalten. Einstellungen können wieder übernommen werden. Im Videoschnitt öffnen übernimmt das gespeicherte Video als Clip in das aktive Projekt.

## Engines und Modelle

ComfyUI verwendet vorhandene Wan-2.1-T2V-/I2V-Modelle sowie Wan-2.2-TI2V-5B. Benötigt werden UMT5 und die zur Modellfamilie passende VAE. Wan 2.1 I2V braucht zusätzlich CLIP Vision H. Local Studio installiert keine Modellabhängigkeiten automatisch. Der ComfyUI-GGUF-Loader muss für GGUF bereits eingerichtet sein.

Bei einer von Local Studio gestarteten ComfyUI-Engine wird auch der Local-Studio-Modellspeicher als Diffusionsmodell-, Textencoder- und VAE-Pfad eingebunden. Nach neuen Downloads gegebenenfalls die Engine neu starten und die Modellliste aktualisieren. Andere externe Dateien müssen für ComfyUI unter dessen diffusion_models liegen. Im Vulkan-Modus können Modell, UMT5 und VAE direkt als lokale Dateien gewählt werden.

Vulkan verwendet die mitgelieferte, hashgeprüfte stable-diffusion.cpp-Runtime in einem isolierten Prozess. Diffusion läuft auf dem Vulkan-Gerät, VAE-Dekodierung auf CPU. Ergebnisse werden mit der geprüften FFmpeg-Runtime als H.264-MP4 ohne Ton kodiert. Die Inferenzpfade werden mit dem GIF-Studio geteilt.

Die Modellliste liest, soweit vorhanden, den begrenzten Modellheader zur Unterscheidung von T2V/I2V. Dateinamen sind nur ein Rückfall. Ein umbenanntes T2V-Modell wird dadurch nicht zu einem I2V-Modell. Wan-2.2-14B mit getrennten High-/Low-Noise-Modellen benötigt einen anderen Workflow und ist hier noch nicht unterstützt. Ebenso sind lange Videos, Tonerzeugung, Interpolation, mehrere Referenzbilder und beliebige andere Videoarchitekturen keine fertiggestellten Funktionen dieses Adapters.

## Nachweis und Grenzen

Zwei wechselbare T2V-Modelle (Wan 2.1 1.3B und Wan 2.2 TI2V 5B) sowie Wan 2.2 mit Referenzbild wurden lokal über den echten ComfyUI-Adapter ausgeführt. Kurze 128×128-Pixel-Läufe mit fünf Frames und zwei Sampling-Schritten prüfen den technischen Pfad, nicht die Qualität normaler Videoaufträge. Vulkan nutzt den vorher geprüften Wan-Framepfad; ein neuer vollständiger Vulkan-MP4-Auftrag oder AMD-Handheld-Lauf wurde für diesen Build nicht wiederholt. Details stehen in TEST_MATRIX.md.

Offizielle Workflowgrundlagen: [Wan 2.1](https://docs.comfy.org/tutorials/video/wan/wan-video), [Wan 2.2](https://docs.comfy.org/tutorials/video/wan/wan2_2).
