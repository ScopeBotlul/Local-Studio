# Lokale Daten und entfernte 18+-Sperre

Die lokale 18+-Sperre mit Altersbestätigung, PIN oder Passwort wurde in Version 0.32.0 auf ausdrücklichen Nutzerwunsch vollständig aus der Bedienoberfläche und der öffentlichen Desktop-IPC entfernt. Local Studio sperrt, verbirgt oder verwischt Medien, Prompts, Projekte, Modelle und Chatverläufe nicht mehr anhand einer 18+-Kennzeichnung. Vorhandene Sperrdaten aus älteren Versionen werden ignoriert; sie müssen nicht entsperrt oder zurückgesetzt werden.

18+- beziehungsweise NSFW-Angaben von Modellanbietern können weiterhin als reine Information in Suchergebnissen und Modellmetadaten erscheinen. Sie lösen keine Zugangssperre aus. Der Civitai-Filter für 18+-Treffer ist eine normale Suchoption ohne PIN- oder Passwortabfrage.

Die allgemeinen Datenschutzregeln bleiben unverändert: Medien, Prompts, Modellgewichte, Projekte, Benchmarks und Chatverläufe bleiben lokal. Es gibt standardmäßig keine Telemetrie. Netzwerkzugriffe erfolgen nur für bewusst verwendete Dienste wie GitHub-Updates, Hugging Face oder Civitai. Local Studio verschlüsselt lokale Dateien nicht; der Schutz des Windows-Kontos und des Datenträgers liegt weiterhin beim Betriebssystem.

Ältere Projekt-, Galerie- und Auftragsformate behalten ihre bisherigen Kennzeichnungsfelder aus Kompatibilitätsgründen. Diese Felder werden nicht mehr für Zugriffsentscheidungen verwendet, damit ältere Daten ohne Migration lesbar bleiben.
