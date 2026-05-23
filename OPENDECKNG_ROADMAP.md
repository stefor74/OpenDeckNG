# OpenDeckNG Roadmap

## Vision
OpenDeckNG ist ein Community-Driven Fork von OpenDeck, spezialisiert auf Linux (CachyOS/Arch) mit Fokus auf:
- Native GIF-Animationen
- Hintergründe/Wallpaper pro Profile
- Erweitertes Starter Pack für Linux-Desktop-Integration
- Bessere Hyprland/Sway/i3-Unterstützung

## Phase 1: Re-Brand & Foundation
- [x] Produktname auf OpenDeckNG umbenennen
- [x] Cargo.toml, package.json, README.md aktualisiert
- [ ] Git-Remote auf stefor74/OpenDeckNG (Repo erstellen)
- [ ] Icons/Branding anpassen

## Phase 2: Native GIF-Animation
- [ ] GIF-Frame-Extraktion in `elgato.rs`
- [ ] `AnimatedImage`-Task pro Button/Encoder
- [ ] Loop mit konfigurierbarer Framerate
- [ ] Memory-Management (alte Tasks stoppen bei `willDisappear`/`setImage`)

## Phase 3: Profile Backgrounds
- [ ] `background_image` + `background_color` in Profile-Store
- [ ] Hintergrund-Rendering unter Buttons (nur für Devices mit großem Display)
- [ ] Webserver-Pfad für Background-Assets
- [ ] Frontend-UI in SvelteKit

## Phase 4: Starter Pack NG (Linux Edition)
- [ ] **Media Action** – MPRIS/D-Bus (Play/Pause, Next, Prev, Mute)
- [ ] **Audio Volume** – PipeWire/PulseAudio (als Dial-Action)
- [ ] **System Monitor** – CPU/RAM/Netzwerk (dynamischer Titel)
- [ ] **Hyprland Window** – Workspace-Wechsel, Fenster-Fokus via `hyprctl`
- [ ] **Notification** – D-Bus Notifications senden
- [ ] **Counter/Timer** – Zähler hoch/runter, Stoppuhr

## Phase 5: StreamController-Adapter (optional)
- [ ] Python-Sidecar für StreamController-Plugins
- [ ] Bridge: Python-Plugin → OpenDeckNG WebSocket

## Phase 6: Release
- [ ] CI/CD für Linux Builds
- [ ] AUR-Paket
- [ ] Flathub (optional)
