# Rauc-update-marker

rauc-health — RAUC Slot Marker Service für OpenRC
rauc-health ist ein leichtgewichtiges Rust-CLI-Tool, das den Boot-Health-Status eines RAUC-Systems (Robust Auto-Update Controller) bewertet und anschließend den aktiven Slot als GOOD oder BAD markiert.
Es ist speziell für OpenRC-basierte Linux Embedded Systeme gedacht und automatisiert den Health-Check nach dem Bootprozess.

**Features**
Automatische Integritätsprüfung laufender Services (Runlevel default)
Markierung des RAUC-Slots als GOOD oder BAD
Reboot-Auslösung bei fehlgeschlagener Health-Prüfung
CLI-Unterkommandos für manuelle Operation
Minimaler Dependency-Footprint

**CLI Übersicht**
rauc-health [COMMAND]
Unterkommandos:
Kommando	Beschreibung
- mark-good	Markiert den aktiven Slot als GOOD
- mark-bad	Markiert den aktiven Slot als BAD
- check	Prüft laufende Dienste im Runlevel default und markiert GOOD/BAD

Beispiel:
- rauc-health check


**Installation**

***Voraussetzungen***
- Linux mit RAUC
- OpenRC (Serviceverwaltung)
- Rust stable (>=1.70)
- pkg-config / build-essential
- Kompilieren
- cargo build --release

Das Binary befindet sich anschließend unter:
- target/release/rauc-health

**Deployment**

Binary in /usr/bin/rauc-health installieren<br\>
Init/OpenRC-Service hinzufügen:
- Datei: service/rauc-health.initd
- Ziel: /etc/init.d/rauc-health
- Autostart konfigurieren
- rc-update add rauc-health default

**Funktionsweise (Kurzüberblick)**

- Prüft Systemservices im Runlevel default via OpenRC.
- Ermittelt fehlgeschlagene oder nicht gestartete Services
- Erfolgreich → rauc status mark-good
- Fehlerhaft → rauc status mark-bad und Exit ≠ 0 (→ OpenRC veranlasst Reboot)

**Build Flags / Umgebungen**

|Variable      |	Zweck                  |
|--------------|-------------------------|
|RUST_LOG=debug|	Aktiviert Debug-Ausgabe|
|RUST_BACKTRACE=1 |	Stacktrace im Fehlerfall|


**Testen**
Simulation:
- rc stop irgendein-service
- rauc-health check
- echo $?   # Wird !=0 sein

**Lizenz**

Projekt unter MIT-Lizenz.


**Technische Dokumentation**

Für Entwickler, Maintainer & Integratoren

1. Architektur
```
+---------------------------+
|        rauc-health        |
+-------------+-------------+
              |
              v
+---------------------------+     +----------------+
| OpenRC Service State API  | --> | RAUC CLI       |
+---------------------------+     +----------------+
```
2. Flow:
Lesen von Services über Command::new("rc-status")
Extraktion „default“-Runlevel
Prüfen, ob Services started

Ergebnis:
OK → mark-good
Fehler → mark-bad + Reboot trigger
2. CLI Aufbau (Clap)
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

enum Commands {
    MarkGood,
    MarkBad,
    Check,
}


**Designentscheidung**

<p>Subcommand-basierte API → klare Trennung von Health-Check und manueller Markierung
Fehlerführung via anyhow → konsistentes Error-Handling</p>

**Health-Evaluation**
Algorithmus
rc-status -r default → Liste von Services
Für jedes Element:
Status started?
Falls nein → in Fehlerliste
Markiere:
GOOD wenn Fehlerliste leer
BAD sonst
Beispiel
openvpn     [started]
ssh         [started]
grafana     [stopped]
→ Fehler: grafana → Slot BAD


**RAUC Integration**
Markieren per CLI
rauc status mark-good
rauc status mark-bad
Rückgabecode ≠ 0 signalisiert Failure → OpenRC veranlasst Reboot des Slots.

**Init Script (OpenRC)**
- Die Datei service/rauc-health.initd implementiert:
Boot-Aktionen:
- Health-Check beim Start
- Restart/Retry-Policy
- Hook für Reboot bei BAD

WICHTIG: Init-Script MUSS root besitzen
→ sonst Markierung schlägt fehl


**Fehlerhandling**
- Logging über log::error!
- anyhow::bail! bei kritischem Failure
- Rückgabe-RC !=0 → OS-Reaktion


**Sicherheit**
- Keine persistente Prozesse
- Saubere Isolation
- RAUC hat Systemprivilegien → Tool muss root
- Logging nicht vertraulichkeitskritisch

**Skalierung**
- Zusätzlich: Whitelist kritischer Dienste
- Integration mit D-Bus statt CLI
- Parallelprüfung / Timeout

**Known Limitations**
***OpenRC Pflicht***
- RAUC Standard CLI erwartet
- Nur Runlevel default geprüft
- Keine dynamische Dependency-Graph-Analyse
