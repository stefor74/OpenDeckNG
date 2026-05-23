# OpenDeckNG

Next-generation Linux software for your Elgato Stream Deck

![Main menu](.github/readme/mainmenu.png)
[More screenshots](#showcase)

OpenDeckNG is a **community-driven Linux-focused fork** of [OpenDeck](https://github.com/nekename/OpenDeck), a desktop application for using stream controller devices like the Elgato Stream Deck. OpenDeckNG brings native Linux desktop integration, GIF animation support, and an enhanced starter pack specifically designed for Linux users running CachyOS, Arch, and Hyprland.

Only Elgato hardware is officially supported, but plugins are available for support for other hardware vendors.

> [!TIP]
> No Stream Deck in front of you? Use OpenDeck with [Tacto](https://tacto.live/) to turn any smartphone into one!

## What's Different in OpenDeckNG?

- **🐧 Linux-first**: Specialized for CachyOS/Arch with Hyprland/Sway/i3 support
- **🎬 Native GIF Animation**: GIFs play directly on your Stream Deck buttons with automatic frame extraction and looping
- **🎨 Profile Backgrounds**: Set background images and colors per profile (experimental)
- **🔊 Linux Starter Pack**: Built-in actions for MPRIS media control, PipeWire/PulseAudio volume, system monitoring, Hyprland workspace switching, and counters
- **GPL-3.0+**: Free and open source, forever

### Why use OpenDeckNG over OpenDeck?

- **Native Linux integration**: No Wine required for core Linux desktop features
- **GIF support**: Plugins can send animated GIFs that play natively on the device
- **Hyprland-ready**: Switch workspaces, kill windows, toggle floating/fullscreen directly
- **PipeWire-native**: Volume control works out of the box on modern Linux desktops

## Installation

### Linux (Recommended)

#### Arch / CachyOS / Manjaro

```bash
# From AUR (build from source)
yay -S opendeckng

# Or binary release
yay -S opendeckng-bin
```

#### Other distributions

> [!TIP]
> For automated installation:
> ```bash
> curl -sSL https://raw.githubusercontent.com/stefor74/OpenDeckNG/main/install_opendeckng.sh | bash
> ```

- Download the latest release from [GitHub Releases](https://github.com/stefor74/OpenDeckNG/releases/latest).
- Install the appropriate udev subsystem rules:
  ```bash
  sudo curl -o /etc/udev/rules.d/40-streamdeck.rules https://raw.githubusercontent.com/OpenActionAPI/rust-elgato-streamdeck/main/40-streamdeck.rules
  sudo udevadm control --reload-rules && sudo udevadm trigger
  ```

#### Dependencies

- `playerctl` – for media control actions
- `wpctl` (PipeWire) or `pactl` (PulseAudio) – for volume control
- `hyprctl` – for Hyprland integration (optional)

### Windows / macOS

OpenDeckNG is primarily focused on Linux. For Windows and macOS, please use the upstream [OpenDeck](https://github.com/nekename/OpenDeck) project.

## Linux Starter Pack Actions

OpenDeckNG includes a built-in plugin with Linux-native actions:

| Action | Description | Requirements |
|--------|-------------|--------------|
| **Media Control** | Play/Pause, Next, Previous, Stop, Mute | `playerctl` |
| **Audio Volume** | Volume Up/Down (dial), Mute Toggle | `wpctl` or `pactl` |
| **System Monitor** | CPU, RAM, Network usage display | none |
| **Hyprland Workspace** | Switch workspaces, toggle floating, kill window | `hyprland` |
| **Counter** | Increment/decrement counter, timer, stopwatch | none |

## Support

### Support forums

- [GitHub Issues](https://github.com/stefor74/OpenDeckNG/issues)

### Building from source

You'll need the [Tauri prerequisites](https://tauri.app/start/prerequisites), [Deno](https://deno.com/), and on Linux `libudev` and `libdbus`.

```bash
deno install
deno task build:plugins
deno task tauri dev   # development
deno task tauri build # production build
```

## Contributing

Before each commit:
1. `cargo clippy` – no warnings
2. `cargo fmt` – formatted
3. `deno check` and `deno lint` – clean TypeScript
4. `deno task check` – clean Svelte
5. `deno fmt --unstable-component` – formatted frontend

Use [Conventional Commits](https://conventionalcommits.org/) and [sign your commits](https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-commits).

OpenDeckNG is licensed under the GNU General Public License version 3.0 or later.

## Showcase

![Main menu](.github/readme/mainmenu.png)
![Multi action](.github/readme/multiaction.png)
![Plugins](.github/readme/plugins.png)
![Profiles](.github/readme/profiles.png)
