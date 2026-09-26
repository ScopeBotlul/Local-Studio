# Local Studio 0.36.15

## Vulkan-Bildengine für AMD- und Intel-Handhelds

- Das Bildstudio bietet **Automatisch**, **Vulkan** und **ComfyUI** als echte Engine-Auswahl.
- Auf AMD-/Intel-Systemen ohne NVIDIA verwendet **Automatisch** den mitgelieferten, hashgeprüften `stable-diffusion.cpp`-Vulkan-Worker. Damit umgeht Local Studio auf ROG Ally und Xbox Ally den nicht unterstützten ComfyUI-/ROCm-7.2-Pfad für `gfx1103`.
- Vulkan-Geräte werden nicht mehr auf NVIDIA beschränkt. NVIDIA, AMD und Intel werden aus der tatsächlichen `sd-cli --list-devices`-Ausgabe gewählt; ohne Vulkan-Gerät bleibt die Generierung gesperrt.
- Geräte mit gemeinsamem Grafikspeicher verwenden die automatische Speicherverteilung der Runtime. Die bisherige Vorabprüfung auf dedizierten NVIDIA-VRAM wird dort nicht angewandt.
- LoRAs wählen automatisch ComfyUI; Referenzbilder und Inpainting wählen automatisch Vulkan. Die Engine wird zusammen mit jedem Auftrag gespeichert. Alte Projekte bleiben kompatibel und laden mit **Automatisch**.

Gezielt geprüft wurden der Rust-Bildenginepfad mit 28 Tests, die Geräteauswahl für NVIDIA/AMD/Intel, alte Auftragsdaten, der Rust-Compilercheck und der TypeScript-/Vite-Produktionsbuild. Das Entwicklungsgerät erkennt seine reale RTX 4080 und integrierte AMD-GPU über die mitgelieferte Vulkan-Runtime. Ein echter WAI-Illustrious-Generierungslauf auf dem ROG Ally/Xbox Ally muss nach Installation dieses Builds auf dem Zielgerät bestätigt werden.
