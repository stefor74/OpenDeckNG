# OpenDeckNG – Session Handoff

**Datum:** 2026-05-23  
**Projekt:** `/home/jaffari/ki/projects/opendeck`  
**Git:** Branch `main`, 2 Commits ahead of origin  
**Remote:** `https://github.com/stefor74/OpenDeck.git` (Ziel: `stefor74/OpenDeckNG`)

---

## Was bisher geschah

### Phase 1: Re-Brand ✅
- Produktname auf **OpenDeckNG** geändert (`product_name.txt`)
- `Cargo.toml`, `package.json`, `README.md`, `AGENTS.md` aktualisiert
- Update-Checker auf `stefor74/OpenDeckNG` umgestellt
- Install-Skript `install_opendeckng.sh` erstellt (curl | bash ready)
- Binary-Name bleibt `opendeck` (Kompatibilität)

### Dokumentation im Repo
| Datei | Zweck |
|-------|-------|
| `OPENDECKNG_ROADMAP.md` | Kurz-Übersicht der 6 Phasen |
| `QUALITY_PLAN.md` | Test-Strategie, Pre-flight checks |
| `docs/MASTERPLAN.md` | Vollständiger Implementierungsplan |
| `docs/SESSION_HANDOFF.md` | Diese Datei – Kontext für nächste Session |

---

## Nächste Schritte (Priorisierung)

### Sofort: Phase 4.1 – Media Action (Quick Win)
**Dateien:** Neues Plugin unter `plugins/com.stefor74.opendeckng.linux.sdPlugin/`
**Tech:** `mpris` oder `zbus` Crate für D-Bus Media Keys
**Impact:** Sofort spürbar auf CachyOS

### Danach: Phase 2 – GIF Animation
**Dateien:** `src-tauri/src/animated_image.rs`, `events/inbound/states.rs`
**Tech:** `gif` Crate, Tokio-Tasks, Frame-Loop
**Impact:** Visueller Wow-Effekt

### Dann: Phase 4.2 bis 4.5 – Starter Pack NG
- Audio Volume (PipeWire/PulseAudio)
- System Monitor (CPU/RAM)
- Hyprland Integration
- Counter/Timer

### Später: Phase 3 – Profile Backgrounds
- Größere UI-Arbeit (Backend + Frontend)

---

## Wichtige Dateien

### Backend (Rust)
```
src-tauri/src/
├── main.rs              # Entry point, Tray-Icon
├── elgato.rs            # Hardware-Kommunikation (HIER: GIF-Animation)
├── shared.rs            # Typen, PRODUCT_NAME
├── events/
│   ├── inbound/states.rs    # setImage (HIER: GIF erkennen)
│   ├── outbound/will_appear.rs  # willDisappear (HIER: Animation stoppen)
│   └── frontend/        # Tauri Commands
├── plugins/             # Plugin-Lifecycle
└── store/               # JSON-Persistenz
```

### Frontend (SvelteKit)
```
src/
├── routes/+page.svelte  # Haupt-UI
├── lib/                 # TypeScript-Typen
└── components/          # UI-Komponenten
```

### Plugins
```
plugins/
├── com.amansprojects.starterpack.sdPlugin/  # Original Starter Pack
└── com.stefor74.opendeckng.linux.sdPlugin/  # NEU: Linux Starter Pack
```

---

## Build & Test

```bash
# Pre-flight (MUSS vor jedem Commit)
(cd src-tauri && cargo clippy --all-targets --all-features)
(cd src-tauri && cargo fmt)
(cd src-tauri && cargo test)
deno check
deno lint
deno task check

# Dev-Build
deno task tauri dev

# Release-Build
deno task tauri build
```

---

## Kontakt & Hilfe

- **Projekt-Owner:** stefor74 (GitHub)
- **Ziel-Distro:** CachyOS (Arch-basiert, Hyprland)
- **Original-Projekt:** nekename/OpenDeck
- **AGENTS.md** im Repo enthält vollständige Architektur-Doku

---

## Offene Punkte

1. [ ] GitHub-Repo `stefor74/OpenDeckNG` erstellen
2. [ ] Initial push der Rebrand-Commits
3. [ ] Phase 4.1 (Media Action) implementieren
4. [ ] Phase 2 (GIF Animation) implementieren
5. [ ] Neue Features testen mit physischem Stream Deck
