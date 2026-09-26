# Local Studio 0.35.2

## Fehlerbehebung für AMD und ROG Ally

- Der offizielle portable AMD-Build von ComfyUI enthält 3.666 AOTriton-/ROCm-Kerneldateien mit einem Sternchen im 7z-Dateinamen. Windows wandelt dieses für NTFS ungültige Zeichen beim Entpacken in das sichere Vollbreitenzeichen `＊` um. Local Studio hatte diese offiziellen Einträge zuvor als unsichere Archivstruktur abgewiesen.
- Local Studio akzeptiert diese Besonderheit jetzt ausschließlich für `.aks2`-Kerneldateien im offiziellen AOTriton-Ordner des eingebetteten PyTorch-Pakets. Andere Wildcards, mehrere Sternchen und alle bisherigen Traversal- und Windows-Pfadverbote bleiben gesperrt.
- Die Korrektur wurde mit dem vollständigen offiziellen AMD-Paket aus ComfyUI v0.37.0 geprüft: SHA-256, alle 69.154 Archivpfade, vollständige Extraktion, Linkfreiheit und die portable Wurzelstruktur stimmen.

## Aktualisierung

Installiere zuerst Local Studio 0.35.2 und starte danach den AMD-ComfyUI-Download erneut. Ein zuvor abgebrochener Versuch hinterlässt kein teilweise eingebundenes Installationsverzeichnis.

## Bekannte Grenze

Die erfolgreiche Installation bestätigt noch nicht, dass jede AMD-APU von der experimentellen offiziellen Windows-ROCm-Laufzeit unterstützt wird. Local Studio meldet einen späteren Start- oder Treiberfehler getrennt vom nun behobenen Archivfehler.
