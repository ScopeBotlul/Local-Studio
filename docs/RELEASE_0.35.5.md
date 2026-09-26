# Local Studio 0.35.5

## ComfyUI auf dem ROG Ally

- Ein echtes Startprotokoll zeigte den nativen Absturz: ComfyUI erkennt die AMD-GPU des ROG Ally als `gfx1103`, stürzt aber beim AOTriton-Kompatibilitätstest in `amdhip64_7.dll` ab.
- Local Studio erkennt offizielle AMD-Portable-Installationen jetzt automatisch und startet sie mit ComfyUIs kompatiblem Split-Cross-Attention-Modus. Dadurch wird der betroffene native AOTriton-Test übersprungen.
- NVIDIA-, Intel- und CPU-Installationen behalten ihren bisherigen Startmodus.
- Im lokalen Startprotokoll steht direkt in der ersten Zeile, wenn der AMD-Kompatibilitätsmodus aktiv ist.

## Prüfumfang

Der geänderte Rust-/ComfyUI-Startpfad und seine direkte Paketerkennung wurden gezielt geprüft. Frontend-Produktionsbuild, optimierter Rust-/Tauri-Build, Installer, portables Archiv und signierte Update-Metadaten wurden ebenfalls geprüft. Ein echter Start auf dem ROG Ally muss nach dem Update noch auf dem Gerät bestätigt werden.
