# Local Studio 0.36.6

## Alte Auftragsfehler nicht mehr als aktueller Studiofehler angezeigt

- Das Bildstudio hat beim Öffnen automatisch den neuesten gespeicherten Auftrag unter dem Canvas ausgewählt. Dadurch erschien dessen alte Meldung „Nicht genug freier RAM“ bereits ohne ausgewähltes Modell und wirkte wie eine aktuelle Sperre.
- Das Studio startet jetzt mit einem leeren Canvas für die aktuellen Eingaben. Laufende und wartende Aufträge werden weiterhin automatisch angezeigt.
- Abgeschlossene, abgebrochene und fehlgeschlagene Aufträge bleiben vollständig unter „Letzte Aufträge“ erhalten und können bewusst geöffnet werden.
- Eine dort angezeigte Fehlermeldung trägt jetzt ausdrücklich die Kennzeichnung „Fehler dieses gespeicherten Auftrags“.
- Die in 0.36.5 entfernte physische RAM-Vorabgrenze für verwaltete ComfyUI-Aufträge bleibt unverändert aktiv.

## Prüfumfang

Der TypeScript-/Vite-Produktionsbuild prüft die geänderte Studioauswahl und Darstellung. Die Rust- und Ressourcenpfade sind gegenüber 0.36.5 unverändert und wurden für diese reine Oberflächenkorrektur nicht erneut getestet. Eine visuelle Kontrolle auf dem ROG Ally bleibt auf dem Zielgerät zu prüfen.
