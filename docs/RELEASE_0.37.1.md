# Local Studio 0.37.1

- Übersichtlicheres Bildstudio: Modell, Prompt, Negativ-Prompt und Bildformat stehen im Vordergrund. LoRAs und weitere Einstellungen sind aufklappbar.
- „Bild bearbeiten“ enthält KI-Bearbeitung mit lokalen SDXL-Modellen über Vulkan: Bild öffnen oder hineinziehen, Bereiche mit Pinsel/Rechteck markieren, radieren sowie Rückgängig/Wiederholen. Alternativ das ganze Bild verändern. Der Ebeneneditor bleibt als eigene Ansicht erhalten.
- Referenzbild-Steuerung aus „Bild erstellen“ entfernt. Bearbeitungseingaben und Markierungen bleiben lokal gespeichert; Galerie und Aufträge öffnen den passenden Arbeitsbereich.
- Originalbilder bleiben unverändert. Unmarkierte Bereiche der modellgerecht angepassten Arbeitskopie werden im Ergebnis erhalten. Ergebnisse werden erst nach Bestätigung in die Galerie gespeichert.
- Referenzbilder lassen sich im Video-/GIF-Studio hineinziehen. Generierungen werden nicht mehr nach einer festen Laufzeit abgebrochen; Verbindungs- und Diagnose-Timeouts bleiben bestehen.

## Prüfung und Grenzen

TypeScript-/Frontend-Kompilierung, gezielte Rust-Prüfungen für Arbeitskopien/Masken und der native Windows-Oberflächentest sind bestanden. Kein vollständiger Regressionstest. Ein echter SDXL-Inpainting-Versuch startete, scheiterte jedoch an fehlendem freien Grafikspeicher. Ein erfolgreicher vollständiger GPU-Lauf für die neue Bearbeitungsoberfläche ist daher noch nicht nachgewiesen. Die optionale CPU-VAE kann Grafikspeicher sparen, garantiert aber keinen erfolgreichen Lauf. KI-Bearbeitung unterstützt derzeit SDXL/Vulkan; Qwen/ComfyUI bleibt für Bildgenerierung verfügbar.

Installer und Portable-Paket verwenden den bestehenden signierten Updatekanal. Eine Authenticode-Signatur des Installers ist weiterhin nicht vorhanden; Windows kann SmartScreen anzeigen. Beim manuellen Portable-Update den Datenordner behalten.
