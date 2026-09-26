# Local Studio 0.36.18

## Modellbibliothek und Civitai

- Unter **Modelle** liegt jetzt dieselbe schmale Werkzeugleiste wie im Studio. Sie wechselt zwischen **Lokal gespeichert**, **Hugging Face** und **Civitai** und lässt sich vollständig mit der Tastatur bedienen.
- Civitai besitzt eine Kontoansicht mit **Mit Civitai anmelden** und **Civitai-Website öffnen**. Beide verwenden ein eigenes dauerhaftes WebView-Profil innerhalb von Local Studio.
- Die Civitai-Suche bietet direkte Typfilter für Checkpoints, LoRAs, VAEs, ControlNet, Upscaler und Embeddings. Ergebnis- und Detailansichten zeigen die vorhandenen Metadaten und führen weiterhin nur geprüfte Safetensors-Dateien dem Downloadmanager zu.
- Der eingebettete Browser bleibt beim Scrollen innerhalb des sichtbaren Fensters und wird außerhalb der Ansicht verborgen. Offizielle Civitai- und Auth-Subdomains sind zugelassen; Lookalike-Domains und unsichere URLs werden abgewiesen.

## Grenzen

Die Civitai-Websitzung bleibt technisch von der öffentlichen Modell-API und den Downloadaufträgen getrennt. Local Studio liest keine Website-Cookies aus und speichert sie nicht in seiner Datenbank. Eine persönliche echte Kontoanmeldung und kontogebundene Downloads wurden nicht automatisiert geprüft. Modelle, Medien, Prompts und Zugangsdaten sind nicht Bestandteil der Pakete.
