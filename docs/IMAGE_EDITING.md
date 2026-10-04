# Bildstudio und KI-Bildbearbeitung

„Bild erstellen“ enthält Modell, Prompt, offenen Negativ-Prompt und Bildformat. LoRAs, Engineinformationen und weitere Generierungsparameter sind aufklappbar. Die Breite der Einstellungen und Galerie bleibt anpassbar.

Unter „Bild bearbeiten“ stehen zwei Ansichten zur Verfügung:

- **Mit KI bearbeiten:** Ein Bild öffnen, hineinziehen oder aus der Studio-Galerie wählen. Pinsel oder Rechteck markieren die Bereiche, die der Prompt verändern soll. Radierer, Rückgängig, Wiederholen und Auswahl löschen korrigieren die Markierung. „Ganzes Bild“ erlaubt Änderungen überall.
- **Ebenen und Werkzeuge:** Der vorhandene Ebeneneditor bleibt verfügbar.

KI-Bearbeitung verwendet einen vollständigen lokalen SDXL-Safetensors-Checkpoint und die Vulkan-Engine. Qwen Image 2.1 über ComfyUI ist weiterhin für Bildgenerierung verfügbar, aber nicht für diesen Bearbeitungspfad. Die Veränderungsstärke legt fest, wie stark die KI vom Ausgangsbild abweichen darf. „VAE auf der CPU ausführen“ kann Grafikspeicher sparen; sie garantiert keine ausreichende Speicherkapazität.

PNG, JPEG, WebP und BMP werden als eigene PNG-Arbeitskopie vorbereitet. Deren Größe wird auf 256–4096 Pixel pro Seite, ein 64-Pixel-Raster und maximal 4.194.304 Pixel angepasst. Das Original wird niemals überschrieben. Bei einer Bereichsmarkierung bleiben schwarze Maskenpixel in dieser Arbeitskopie exakt erhalten; weiche Maskenkanten werden überblendet. Nach einer Größenanpassung bezieht sich diese Zusage auf die Arbeitskopie, nicht auf die Originalauflösung.

Eingaben und Markierungen bleiben lokal gespeichert. Ergebnisse sind zunächst temporär und werden erst nach „In Galerie speichern“ übernommen. Bildgenerierung und Bearbeitung besitzen getrennte Eingaben und gefilterte Auftragsverläufe; alle Aufträge bleiben in der gemeinsamen Auftragsübersicht erreichbar.
