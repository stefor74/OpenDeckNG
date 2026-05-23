# OpenDeckNG Roadmap

## Vision
OpenDeckNG ist ein Community-Driven Fork von OpenDeck, spezialisiert auf Linux (CachyOS/Arch) mit Fokus auf:
- Native GIF-Animationen
- Hintergründe/Wallpaper pro Profile
- Erweitertes Starter Pack für Linux-Desktop-Integration
- Bessere Hyprland/Sway/i3-Unterstützung

## Phase 1: Re-Brand & Foundation ✅
- [x] Produktname auf OpenDeckNG umbenennen
- [x] Cargo.toml, package.json, README.md aktualisiert
- [x] Git-Remote auf stefor74/OpenDeckNG (Repo erstellt und gepusht)
- [x] Icons/Branding angepasst (App-Icon, Plugin-Icons, Static-Icons)
- [x] Desktop-Datei, Metainfo, CI/CD auf NG umgestellt

## Phase 2: Native GIF-Animation ✅
- [x] GIF-Frame-Extraktion in `animated_image.rs`
- [x] `AnimatedImage`-Task pro Button/Encoder
- [x] Loop mit konfigurierbarer Framerate
- [x] Memory-Management (alte Tasks stoppen bei `willDisappear`/`setImage`)

## Phase 3: Profile Backgrounds 🟡
- [x] `background_image` + `background_color` in Profile-Store
- [x] Frontend-UI in SvelteKit (BackgroundPicker + Tabs)
- [ ] Hintergrund-Rendering auf Hardware (komplex, elgato-streamdeck hat kein Layering)

## Phase 4: Starter Pack NG (Linux Edition) ✅
- [x] **Media Action** – MPRIS/D-Bus (Play/Pause, Next, Prev, Mute)
- [x] **Audio Volume** – PipeWire/PulseAudio (als Dial-Action)
- [x] **System Monitor** – CPU/RAM/Netzwerk (dynamischer Titel)
- [x] **Hyprland Window** – Workspace-Wechsel, Fenster-Fokus via `hyprctl`
- [x] **Counter/Timer** – Zähler hoch/runter, Stoppuhr
- [ ] **Notification** – D-Bus Notifications senden (optional)

## Phase 5: StreamController-Adapter (optional) ⬜
- [ ] Python-Sidecar für StreamController-Plugins
- [ ] Bridge: Python-Plugin → OpenDeckNG WebSocket

## Phase 6: Release ✅
- [x] CI/CD für Linux Builds
- [x] AUR-Paket (PKGBUILD + PKGBUILD-bin)
- [ ] Flathub (optional)

## Offene Punkte / Backlog
- [ ] Flathub-Release
- [ ] Backend-Background-Rendering auf Stream Deck LCD/Buttons
- [ ] StreamController-Adapter
- [ ] Notification Action (D-Bus)
- [ ] Icons durch professionelles Design ersetzen (aktuell: Script-generiert)
