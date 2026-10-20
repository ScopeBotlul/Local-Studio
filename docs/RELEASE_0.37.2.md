# Local Studio 0.37.2

- Im GIF-Studio unter „Mit KI“ lassen sich jetzt bis zu acht LoRAs aus der lokalen Modellbibliothek oder per Safetensors-Datei hinzufügen.
- Stärke jeder LoRA einzeln einstellen, Einträge entfernen und die Auswahl lokal behalten. Dieselbe Auswahl steht auch im Video-Studio zur Verfügung.
- ComfyUI verwendet eine Kette aus LoraLoaderModelOnly-Knoten vor dem Sampling. Externe LoRA-Ordner werden über den bestehenden Mechanismus registriert; fremd gestartete Engines werden nicht automatisch beendet.
- Vulkan verwendet geprüfte, schreibgeschützte Quelldateien und temporäre LoRA-Arbeitskopien mit neutralen Namen. Die Arbeitskopien werden nach dem Auftrag entfernt.

## Prüfung und Grenzen

Gezielte Rust-Tests für LoRA-Verkettung, Eingabegrenzen, Dateisperren, Staging und Bereinigung sowie der isolierte native Oberflächentest sind bestanden. Dieser Oberflächentest prüft die Anfrageübergabe kontrolliert; eine vollständige GPU-Generierung mit einer echten Wan-LoRA wurde nicht durchgeführt. Nur zur jeweiligen Wan-Modellversion passende LoRAs verwenden; SDXL-Bild-LoRAs sind inkompatibel. Kein vollständiger Regressionstest für unveränderte Funktionen. Die bekannte Grafikspeichergrenze des vorherigen SDXL-Inpainting-Tests bleibt bestehen.

Installer und Portable-ZIP werden mit Prüfsummen und signierten Update-Metadaten bereitgestellt. Der Installer besitzt weiterhin keine Authenticode-Signatur. Beim manuellen Portable-Update den Datenordner behalten.
