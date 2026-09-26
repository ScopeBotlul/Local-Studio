# Local Studio 0.36.16

## Oberfläche, Skalierung und LoRAs auf Windows-Handhelds

- Die eingestellte Oberflächenskalierung verändert jetzt den gesamten Arbeitsbereich wirklich. Der Inhalt behält dabei exakt die verfügbare Fensterfläche; abgeschnittene oder fast leere Studioansichten bei abweichender Skalierung werden vermieden.
- Logo, primäre Schaltflächen und das laufende Windows-Fenster-/Taskleistenicon übernehmen die eingestellte Akzentfarbe. Das Logo wählt automatisch eine lesbare helle oder dunkle Innenfarbe.
- LoRAs lassen sich auf ROG Ally, Xbox Ally und anderen AMD-/Intel-Geräten mit der Vulkan-Engine auswählen und anwenden. Local Studio stellt die bereits geprüften Dateien mit neutralen Namen nur für die Dauer des Auftrags bereit und übergibt sie an den isolierten `stable-diffusion.cpp`-Worker.
- ComfyUI bleibt als alternative Engine erhalten. Bei **Automatisch** bleibt auf AMD-/Intel-Handhelds Vulkan die bevorzugte Route; auf NVIDIA wird weiterhin ComfyUI bevorzugt.

## Prüfung und bekannte Grenzen

Gezielte Frontend- und Rust-Prüfungen decken Akzentkontrast, vollständige UI-Skalierung, LoRA-Parameter, sicheres Bereitstellen und anschließendes Entfernen der temporären LoRA-Dateien ab. Der Produktionsbuild sowie der native Windows-Build wurden erstellt. Ein echter LoRA-Generierungslauf auf dem konkreten ROG Ally bleibt als Zielgerätetest offen; Modell und LoRA müssen dieselbe Basisarchitektur verwenden. Das installierte Datei-/Startmenüicon bleibt das statische Programmlogo, während das Icon des laufenden Fensters und der Taskleistenschaltfläche die Akzentfarbe übernimmt. Die Windows-Dateien sind weiterhin nicht Authenticode-signiert.
