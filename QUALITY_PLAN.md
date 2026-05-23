# OpenDeckNG Quality Plan

## Ziel
Keine Showstopper-Bugs nach Integration neuer Features.

## 1. Pre-Flight Checklist (vor jedem Commit)

```bash
# Rust
(cd src-tauri && cargo clippy --all-targets --all-features)
(cd src-tauri && cargo fmt --check)
(cd src-tauri && cargo test)

# Frontend
deno check
deno lint
deno task check
```

**Hard stop** wenn einer rot ist.

## 2. Test-Pyramide

### Unit Tests (Rust)
- `animated_image::extract_gif_frames()` – Frames korrekt extrahiert?
- `events/inbound/states::set_image()` – GIF erkannt, Animation gestartet?
- `store::profiles` – Background-Felder backward-kompatibel?

### Integration Tests (Rust)
- Plugin-Event-Routing: Sendet `setImage` → Hardware-Update?
- Profile-Save/Load: Background persistiert korrekt?

### E2E Tests (Frontend)
- Button-Klick in SvelteKit → Tauri invoke → Backend antwortet
- Property Inspector lädt und sendet Events

## 3. Hardware-Mocking

Für Tests ohne Stream Deck:
```rust
#[cfg(test)]
mod mock {
    use elgato_streamdeck::AsyncStreamDeck;
    // Mock-Device, das Events logged statt HID zu schreiben
}
```

## 4. Feature-Flags

Jede neue Phase hinter einem Feature-Flag:
```rust
#[cfg(feature = "gif-animation")]
mod animated_image;
```

Falls etwas bricht: `--no-default-features` baut ohne neue Features.

## 5. Rollback-Strategie

```bash
# Falls Showstopper nach Merge:
git revert HEAD
# Oder:
git checkout main -- src-tauri/src/elgato.rs
```

## 6. manuelle Test-Prozedur (mit Hardware)

| Test | Schritte | Erwartet |
|------|----------|----------|
| GIF Animation | GIF-Datei auf Button legen | Animation läuft, Frames wechseln |
| GIF Stop | Button löschen / neues Bild | Animation stoppt, kein Speicherleck |
| Media Action | Play/Pause drücken | Musik pausiert/spielt |
| Volume Dial | Encoder drehen | System-Lautstärke ändert sich |
| Hyprland WS | Workspace-Button drücken | `hyprctl` wechselt Workspace |
| Background | Bild auf Profil legen | Hintergrund sichtbar (nur LCD-Devices) |

## 7. CI/CD Gates

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    steps:
      - run: cargo clippy -- -D warnings
      - run: cargo test
      - run: deno lint
      - run: deno task check
      - run: deno task tauri build  # Full build
```
