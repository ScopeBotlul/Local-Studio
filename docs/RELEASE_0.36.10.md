# Local Studio 0.36.10

## ComfyUI-Abstürze beenden Bildaufträge zuverlässig

- Beendet sich die von Local Studio verwaltete ComfyUI-Engine während einer Generierung, wird der Bildauftrag jetzt sofort mit einer verständlichen Absturzmeldung beendet.
- Antwortet die lokale ComfyUI-API wiederholt nicht mehr, bleibt der Auftrag nicht länger bis zum 15-Minuten-Zeitlimit scheinbar aktiv.
- Bugreports behalten nun sowohl den Anfang des ComfyUI-Logs mit der eigentlichen Fehlermeldung als auch das Ende des nativen Stackdumps.
- Die Diagnoseübersicht zählt jetzt auch Bildaufträge nach Status, ohne Prompts, Modellpfade oder Ausgabedateien offenzulegen.

## Befund aus GitHub-Issue #1

Der Bericht aus Version 0.36.9 zeigt einen nativen Absturz des eingebetteten Python-/ComfyUI-Prozesses auf dem ROG Ally. Der bisherige Bericht enthielt wegen der reinen Ende-Kürzung nur den nativen `python312.dll`-Stackdump; die davor ausgegebene konkrete Fehlerursache fehlte. Deshalb ist noch nicht belastbar entschieden, ob der Absturz durch Speicherknappheit, den AMD-Treiber oder die ROCm-/PyTorch-Laufzeit ausgelöst wurde.

## Prüfumfang

Drei gezielte Bugreport-Tests, sechs ComfyUI-Tests, TypeScript-/Vite-Produktionsbuild, optimierter Rust-/Tauri-Build, Installer, portables Archiv und signierte Update-Metadaten sind bestanden. Der echte AMD-/WAI-Illustrious-Lauf und die vollständige Absturzursache bleiben auf dem ROG Ally zu prüfen.
