# OpenDeckNG – Masterplan

**Datum:** 2026-05-23  
**Agent:** Kimi Code CLI  
**Projekt:** `/home/jaffari/ki/projects/opendeck`  
**Ziel:** OpenDeckNG als Community-Fork für Linux (CachyOS/Arch) mit Next-Gen Features

---

## 0. Einführung

OpenDeckNG ist ein Fork des originalen OpenDeck-Projekts (nekename/OpenDeck). Der Fokus liegt auf Linux-Desktop-Integration (CachyOS/Arch/Hyprland) und visuellen Verbesserungen, die im Original fehlen oder unzureichend sind.

**Kernarchitektur bleibt erhalten:**
- Tauri v2 (Rust-Backend)
- SvelteKit + TypeScript + Tailwind CSS v4 (Frontend)
- Deno (Build-Tool)
- OpenAction/StreamDeck SDK (Plugin-Protokoll)

---

## Phase 1: Re-Brand & Foundation ✅ ERLEDIGT

### 1.1 Umbenennung (erledigt)
- `product_name.txt` → "OpenDeckNG"
- `src-tauri/Cargo.toml` → name: `opendeck-ng`
- `package.json` → name: `opendeck-ng`
- `README.md`, `AGENTS.md` Titel angepasst
- Update-Checker zeigt auf `stefor74/OpenDeckNG`

### 1.2 Offen: GitHub-Repo
- **Aktion:** `stefor74/OpenDeckNG` Repository auf GitHub erstellen
- **Aktion:** Remote umstellen: `git remote set-url origin https://github.com/stefor74/OpenDeckNG.git`
- **Aktion:** Initial push mit Rebrand-Commit

---

## Phase 2: Native GIF-Animation

### Problem
`elgato.rs` Zeile 52: `image::load_from_memory(&bytes)?` lädt bei GIFs nur das **erste Frame**. Plugins müssen selbst einen Timer implementieren und frame-by-frame `setImage` senden. Das ist umständlich und fehleranfällig.

### Lösung
GIF-Frames automatisch extrahieren und in einem verwalteten Task loopen.

### 2.1 Architektur

```
┌─────────────────────────────────────────────┐
│  Plugin sendet: setImage mit GIF (Base64)   │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│  events/inbound/states.rs: set_image()      │
│  → erkennt GIF anhand Magic Bytes           │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│  Neuer Modul: animated_image.rs             │
│  • Extrahiert alle Frames mit `gif` crate   │
│  • Berechnet Frame-Delays (GIF-Delay/10)    │
│  • Speichert Frames als Vec<DynamicImage>   │
│  • Startet Tokio-Task pro Context           │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│  Animation Loop (pro Button/Encoder)        │
│  • Interval basierend auf Frame-Delay       │
│  • Sendet Frame N an elgato::update_image() │
│  • Loop oder einmalig (konfigurierbar)      │
└─────────────────────────────────────────────┘
```

### 2.2 Implementierung

#### 2.2.1 Neue Crate: `gif` ( falls noch nicht in Cargo.toml )
```toml
[dependencies]
gif = "0.13"
```

#### 2.2.2 Neues Modul: `src-tauri/src/animated_image.rs`

```rust
use std::collections::HashMap;
use std::sync::Arc;

use image::DynamicImage;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio::time::{Duration, interval};

/// Eine extrahierte GIF-Animation
struct GifAnimation {
    frames: Vec<DynamicImage>,
    delays: Vec<Duration>,  // pro Frame
    loop_count: Option<u16>, // None = infinite
}

/// Globaler Registry für laufende Animationen
static ACTIVE_ANIMATIONS: LazyLock<RwLock<HashMap<String, JoinHandle<()>>>> = 
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Extrahiert Frames aus GIF-Daten
fn extract_gif_frames(bytes: &[u8]) -> Result<GifAnimation, anyhow::Error> {
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = options.read_info(bytes)?;
    
    let mut frames = Vec::new();
    let mut delays = Vec::new();
    let loop_count = decoder.loop_count(); // None = infinite
    
    while let Some(frame) = decoder.read_next_frame()? {
        let image = image::RgbaImage::from_raw(
            frame.width as u32,
            frame.height as u32,
            frame.buffer.to_vec(),
        ).ok_or_else(|| anyhow::anyhow!("Invalid frame dimensions"))?;
        
        frames.push(DynamicImage::ImageRgba8(image));
        // GIF delay is in 1/100ths of a second
        delays.push(Duration::from_millis((frame.delay as u64) * 10));
    }
    
    Ok(GifAnimation { frames, delays, loop_count })
}

/// Startet oder ersetzt eine Animation für einen Context
pub async fn start_animation(
    context: crate::shared::Context,
    gif_bytes: Vec<u8>,
) -> Result<(), anyhow::Error> {
    // Alte Animation stoppen
    stop_animation(&context).await;
    
    let animation = extract_gif_frames(&gif_bytes)?;
    if animation.frames.is_empty() {
        return Ok(());
    }
    
    let context_key = format!("{}:{}:{}:{}", context.device, context.profile, context.controller, context.position);
    
    let handle = tokio::spawn(async move {
        let mut frame_index = 0usize;
        let mut loop_counter = 0u16;
        
        loop {
            let frame = &animation.frames[frame_index];
            let delay = animation.delays.get(frame_index).copied().unwrap_or(Duration::from_millis(100));
            
            // Frame als Base64 konvertieren und an elgato senden
            let mut buffer = Vec::new();
            if let Ok(_) = frame.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png) {
                let base64 = base64::engine::general_purpose::STANDARD.encode(&buffer);
                let data_url = format!("data:image/png;base64,{}", base64);
                let _ = crate::elgato::update_image(&context, Some(&data_url)).await;
            }
            
            tokio::time::sleep(delay).await;
            
            frame_index += 1;
            if frame_index >= animation.frames.len() {
                frame_index = 0;
                loop_counter += 1;
                
                // Prüfe Loop-Count
                if let Some(max_loops) = animation.loop_count {
                    if loop_counter >= max_loops {
                        break;
                    }
                }
            }
        }
    });
    
    ACTIVE_ANIMATIONS.write().await.insert(context_key, handle);
    Ok(())
}

/// Stoppt eine laufende Animation
pub async fn stop_animation(context: &crate::shared::Context) {
    let context_key = format!("{}:{}:{}:{}", context.device, context.profile, context.controller, context.position);
    if let Some((_, handle)) = ACTIVE_ANIMATIONS.write().await.remove(&context_key) {
        handle.abort();
    }
}
```

#### 2.2.3 Integration in `events/inbound/states.rs`

In `set_image()` muss vor dem Speichern geprüft werden, ob es sich um ein GIF handelt:

```rust
pub async fn set_image(mut event: ContextAndPayloadEvent<SetImagePayload>) -> Result<(), anyhow::Error> {
    // ... bestehende Logik ...
    
    // NEU: GIF-Animation erkennen und starten
    if let Some(image) = &event.payload.image {
        if image.starts_with("data:image/gif") {
            let data = image.split_once(',').unwrap().1;
            let bytes = base64::engine::general_purpose::STANDARD.decode(data)?;
            crate::animated_image::start_animation(event.context.clone(), bytes).await?;
            // GIF wird nicht als statisches State-Bild gespeichert, sondern separat verwaltet
            return Ok(());
        }
    }
    
    // ... restliche bestehende Logik ...
}
```

#### 2.2.4 Cleanup bei willDisappear

In `events/outbound/will_appear.rs` bei `will_disappear()`:
```rust
// Animation stoppen
crate::animated_image::stop_animation(&instance.context).await;
```

### 2.3 Testplan
1. GIF-Datei als Base64 konvertieren
2. `setImage` mit GIF senden
3. Verifizieren, dass Animation läuft (Frames wechseln)
4. `willDisappear` senden → Animation stoppt
5. Neues statisches Bild setzen → Animation stoppt

---

## Phase 3: Profile Backgrounds

### Problem
OpenDeck hat keine Hintergrundebene. StreamController zeigt Hintergrundbilder/Videos unter den Buttons an. Das ist besonders für Devices mit großem Display (Stream Deck +, Neo) visuell ansprechend.

### Lösung
`background_image` und `background_color` pro Profile speichern und rendern.

### 3.1 Datenmodell

#### 3.1.1 Erweiterung `shared.rs` – `Profile` struct
```rust
#[derive(Clone, Deserialize, Serialize)]
pub struct Profile {
    pub keys: Vec<Option<ActionInstance>>,
    pub sliders: Vec<Option<ActionInstance>>,
    // NEU:
    #[serde(default)]
    pub background_image: Option<String>,  // Base64 oder Pfad
    #[serde(default)]
    pub background_color: Option<String>,  // Hex-Farbe
    #[serde(default)]
    pub background_mode: BackgroundMode,   // Stretch, Tile, Center, Cover
}

#[derive(Clone, Deserialize, Serialize, Default)]
pub enum BackgroundMode {
    #[default]
    Stretch,
    Tile,
    Center,
    Cover,
}
```

#### 3.1.2 Store-Migration
In `store/profiles.rs` muss die JSON-Deserialisierung backward-kompatibel bleiben (serde(default) hilft).

### 3.2 Backend-Rendering

Für Devices mit LCD-Strip (Plus, Neo):
- Hintergrund wird als Bild auf den LCD-Strip geschrieben
- Buttons werden darüber gerendert (elgato-streamdeck crate unterstützt Layering nicht nativ)
- Alternative: Hintergrund nur im "leeren" Bereich zwischen Buttons

Für klassische Devices (Original, Mini, XL):
- Kein echter Hintergrund möglich (nur Buttons mit Bildern)
- Workaround: Hintergrund auf leere Buttons kopieren

### 3.3 Frontend-UI

#### 3.3.1 Neue SvelteKit-Komponente: `BackgroundPicker.svelte`
- Bild-Upload (Drag & Drop)
- Farb-Picker für Fallback
- Mode-Auswahl (Stretch, Tile, Center, Cover)
- Preview

#### 3.3.2 Integration in Profile-Editor
- Neuer Tab "Background" neben "Plugins" und "Settings"
- Tauri-Command: `set_background`

### 3.4 Implementierungsschritte

1. **Datenmodell erweitern** (`shared.rs`, `store/profiles.rs`)
2. **Tauri-Commands hinzufügen** (`events/frontend/profiles.rs`):
   - `get_background(device, profile)`
   - `set_background(device, profile, image, color, mode)`
3. **Backend-Rendering** (`elgato.rs` oder neues `background.rs`):
   - Hintergrundbild skalieren und auf LCD-Strip schreiben
   - Bei Profilwechsel Hintergrund neu laden
4. **Frontend-Komponente** (`src/components/BackgroundPicker.svelte`)
5. **Profile-UI erweitern** (`src/routes/+page.svelte`)

---

## Phase 4: Starter Pack NG (Linux Edition)

### Ziel
5 neue Actions speziell für Linux-Desktops (CachyOS/Hyprland/Sway/i3).

### 4.1 Media Action (MPRIS/D-Bus)

**Funktion:** Play/Pause, Next, Previous, Mute  
**Technologie:** D-Bus `org.mpris.MediaPlayer2` Interface  
**Crate:** `mpris` oder raw `zbus`

```rust
// Beispiel-Implementierung
use mpris::PlayerFinder;

async fn media_play_pause() -> Result<(), anyhow::Error> {
    let finder = PlayerFinder::new()?;
    if let Ok(player) = finder.find_active() {
        player.play_pause()?;
    }
    Ok(())
}
```

**Actions:**
- Media Play/Pause
- Media Next
- Media Previous
- Media Mute (System)

**Property Inspector:** Player-Auswahl (Active, Spotify, Firefox, etc.)

### 4.2 Audio Volume (PipeWire/PulseAudio)

**Funktion:** Lautstärke regeln (ideal für Encoder/Dial)  
**Technologie:** PipeWire (modern) oder PulseAudio (legacy)  
**Crate:** `libspa-sys` (low-level) oder Shell-Out zu `pactl`/`wpctl`

```rust
// Einfacher Ansatz: wpctl (PipeWire)
async fn set_volume(delta: i8) -> Result<(), anyhow::Error> {
    let current = get_current_volume().await?;
    let new = (current as i16 + delta as i16).clamp(0, 100) as u8;
    std::process::Command::new("wpctl")
        .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{}%", new)])
        .spawn()?;
    Ok(())
}
```

**Actions:**
- Volume Up/Down (als Dial-Rotate)
- Volume Mute Toggle
- Volume Set ( konkreter Wert )

**Visual Feedback:** Aktuelle Lautstärke als Titel anzeigen (z.B. "Vol: 65%")

### 4.3 System Monitor

**Funktion:** CPU, RAM, Netzwerk-Usage anzeigen  
**Technologie:** `/proc/stat`, `/proc/meminfo`, `/proc/net/dev`  
**Aktualisierung:** Alle 2 Sekunden

```rust
use sysinfo::System;

static SYSTEM: LazyLock<Mutex<System>> = LazyLock::new(|| Mutex::new(System::new_all()));

async fn get_cpu_usage() -> f32 {
    let mut sys = SYSTEM.lock().await;
    sys.refresh_cpu_usage();
    sys.global_cpu_usage()
}
```

**Actions:**
- CPU Usage (zeigt % als Titel)
- RAM Usage (zeigt % oder GB als Titel)
- Network Up/Down (zeigt MB/s als Titel)

**Timer:** Plugin startet internen Tokio-Task, der alle 2 Sekunden `setTitle` sendet.

### 4.4 Hyprland Window/Workspace

**Funktion:** Workspace-Wechsel, Fenster-Fokus, Fenster minimieren/maximieren  
**Technologie:** `hyprctl` (Hyprland IPC)  
**Voraussetzung:** Hyprland compositor läuft

```rust
async fn hyprctl(args: &[&str]) -> Result<String, anyhow::Error> {
    let socket = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
    let output = std::process::Command::new("hyprctl")
        .args(args)
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

async fn switch_workspace(workspace: u8) -> Result<(), anyhow::Error> {
    hyprctl(&["dispatch", "workspace", &workspace.to_string()]).await?;
    Ok(())
}
```

**Actions:**
- Switch Workspace (1-10, oder +/-)
- Focus Window (by class/name)
- Toggle Floating
- Kill Active Window
- Toggle Fullscreen

**Property Inspector:** Workspace-Nummer oder +/- Modus

### 4.5 Counter / Timer

**Funktion:** Zähler hoch/runter, Stoppuhr  
**State:** Im Plugin-Prozess gehalten (nicht persistent)

```rust
struct CounterState {
    value: i32,
    step: i32,
}

async fn increment_counter(instance: &Instance, state: &mut CounterState) {
    state.value += state.step;
    instance.set_title(Some(state.value.to_string()), None).await?;
}
```

**Actions:**
- Counter (Up/Down mit konfigurierbarem Step)
- Timer (Countdown mit Alarm-Notification)
- Stopwatch (Elapsed time)

### 4.6 Plugin-Struktur

Neues Plugin: `com.stefor74.opendeckng.linux.sdPlugin`

```
plugins/com.stefor74.opendeckng.linux.sdPlugin/
├── Cargo.toml
├── assets/
│   ├── manifest.json
│   ├── icons/
│   └── propertyInspector/
├── src/
│   ├── main.rs
│   ├── media.rs
│   ├── volume.rs
│   ├── system_monitor.rs
│   ├── hyprland.rs
│   └── counter.rs
└── build.ts
```

### 4.7 Property Inspectors

Für jede Action ein simples HTML-Formular:
- Media: Dropdown (PlayPause, Next, Prev, Mute)
- Volume: Checkbox (Show current volume as title)
- System Monitor: Dropdown (CPU, RAM, Net Up, Net Down)
- Hyprland: Textfeld (Workspace-Nummer oder +/-)
- Counter: Number-Feld (Step-Wert)

---

## Phase 5: StreamController-Adapter (optional)

### Problem
StreamController-Plugins sind Python-basiert und nutzen ein GTK4-Plugin-System. Direkte Kompatibilität ist nicht möglich.

### Lösung
Adapter-Layer, der StreamController-Plugins als externe Prozesse lädt und deren Output an OpenDeckNG's WebSocket weiterleitet.

### 5.1 Architektur

```
┌─────────────────────────────────────────────┐
│  StreamController Plugin (Python)           │
│  • Läuft als eigener Prozess                │
│  • Kommuniziert über stdout/stderr          │
└──────────────┬──────────────────────────────┘
               │ JSON-RPC über stdin/stdout
               ▼
┌─────────────────────────────────────────────┐
│  Adapter-Service (Python oder Rust)         │
│  • Startet SC-Plugin als Subprozess         │
│  • Übersetzt SC-API → OpenAction Events     │
│  • WebSocket-Client zu OpenDeckNG           │
└──────────────┬──────────────────────────────┘
               │ WebSocket
               ▼
┌─────────────────────────────────────────────┐
│  OpenDeckNG (WebSocket Server)              │
│  • Empfängt als normales Plugin             │
└─────────────────────────────────────────────┘
```

### 5.2 Implementierung

#### Option A: Python-Bridge (empfohlen)
- Python-Script `streamcontroller_bridge.py`
- Lädt StreamController-Plugin via `importlib`
- Übersetzt Methoden-Aufrufe

#### Option B: Rust-Wrapper
- Rust-Binary als Plugin
- Startet Python-Prozess
- Kommuniziert über stdin/stdout

### 5.3 Komplexität
**Hoch.** StreamController's Plugin-API ist undokumentiert und ändert sich häufig.  
**Empfehlung:** Nur wenn konkrete Plugins gewünscht sind (z.B. sehr beliebtes Plugin existiert nur für StreamController).

### 5.4 Alternative
Statt Adapter: Die Funktionalität der beliebtesten StreamController-Plugins direkt als native OpenDeckNG-Plugins nachbauen.

---

## Phase 6: Release & Distribution

### 6.1 Build-System
- CI/CD mit GitHub Actions
- Targets: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu
- .deb, .rpm, .tar.gz Releases

### 6.2 AUR-Paket
- `opendeckng` (build from source)
- `opendeckng-bin` (binary release)

### 6.3 Flathub
- Flatpak-Manifest
- udev-Rules Hinweis beibehalten

### 6.4 Dokumentation
- Installationsanleitung für CachyOS/Arch
- Hyprland-Integration Guide
- Plugin-Entwickler-Doku

---

## Anhang: Datei-Referenz

### Kern-Dateien (Backend)
| Datei | Zweck |
|-------|-------|
| `src-tauri/src/main.rs` | Entry point, Tauri-Setup |
| `src-tauri/src/elgato.rs` | Hardware-Kommunikation |
| `src-tauri/src/shared.rs` | Typen, Konstanten |
| `src-tauri/src/events/inbound/states.rs` | setImage, setTitle, setState |
| `src-tauri/src/events/outbound/will_appear.rs` | willAppear, willDisappear |
| `src-tauri/src/store/` | JSON-Persistenz |
| `src-tauri/src/plugins/` | Plugin-Lifecycle |

### Frontend
| Datei | Zweck |
|-------|-------|
| `src/routes/+page.svelte` | Haupt-UI |
| `src/lib/` | TypeScript-Typen |

### Plugin (Starter Pack)
| Datei | Zweck |
|-------|-------|
| `plugins/com.amansprojects.starterpack.sdPlugin/src/main.rs` | Plugin-Entry |
| `plugins/.../src/run_command.rs` | Shell-Befehle |
| `plugins/.../assets/manifest.json` | Plugin-Metadaten |

---

## Priorisierung (Empfehlung)

| Rang | Phase | Aufwand | Impact |
|------|-------|---------|--------|
| 1 | 4.1 Media Action | Klein | Sofort spürbar |
| 2 | 4.2 Audio Volume | Klein | Sofort spürbar |
| 3 | 2 GIF Animation | Mittel | Visueller Wow |
| 4 | 4.3 System Monitor | Mittel | Nützlich |
| 5 | 4.4 Hyprland | Mittel | Linux-spezifisch |
| 6 | 3 Backgrounds | Groß | Visuell |
| 7 | 4.5 Counter | Klein | Nützlich |
| 8 | 5 StreamController | Sehr groß | Optional |
| 9 | 6 Release | Mittel | Distribution |
