# Local Studio 0.36.13

## Benachrichtigungen statt störender Einblendungen

- Neben dem Downloadsymbol befindet sich jetzt ein Benachrichtigungssymbol mit Zähler.
- Projekt-, Bildarbeitsstand- und Auftragswiederherstellungen erscheinen gesammelt im Benachrichtigungsfenster.
- Wiederherstellungen können direkt aus der Liste gestartet werden. Der Hinweis auf einen nicht regulär beendeten Start lässt sich dort prüfen oder ausblenden.
- Verbindungsprobleme und allgemeine Bedienfehler werden ebenfalls in der Liste angezeigt, statt den Arbeitsbereich mit einer schwebenden Fehlermeldung zu verdecken.
- Das Fenster unterstützt Tastaturfokus, Escape, Klick außerhalb, deutsche und englische Texte sowie die eingestellte Akzentfarbe.

## Neuer ROG-Ally-Bericht

- GitHub-Issue #4 bestätigt, dass das in 0.36.12 eingeführte AMD-Low-Memory-Profil vollständig aktiv ist: DynamicVRAM, gepinnter Speicher und asynchrones Offloading sind aus; der SDXL-Textencoder läuft auf der CPU.
- WAI Illustrious und der SDXL-Textencoder werden vollständig geladen. Erst beim eigentlichen Sampling beendet sich die offizielle ROCm-Laufzeit auf `gfx1103` mit einer nativen Windows-Zugriffsverletzung.
- Dieser Fehler liegt nach dem erfolgreichen Modell- und Workflowaufbau im nativen ROCm-/PyTorch-Samplingpfad. 0.36.13 ändert deshalb keine weiteren Speicherparameter auf Verdacht.
- Die Bildgenerierung mit diesem Modell auf dem ROG Ally bleibt eine bekannte Einschränkung. Ein automatischer CPU-Fallback wird nicht aktiviert, weil SDXL dort praktisch nicht sinnvoll schnell wäre.

## Prüfumfang

Sieben gezielte Tests für Benachrichtigungs- und Downloadanzeige, der Frontend-Produktionsbuild, der optimierte Rust-/Tauri-Build, Installer, portables Archiv und signierte Update-Metadaten sind bestanden. Die unveränderten Rust-Inferenztests wurden nicht erneut ausgeführt. Der echte ROG-Ally-Lauf bleibt als Zielgeräteprüfung offen.
