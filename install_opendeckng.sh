#!/bin/bash
set -e

echo "OpenDeckNG Installer for Linux"
echo "================================"

# Detect distribution
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO=$ID
else
    echo "Could not detect distribution. Please install manually."
    exit 1
fi

# Install dependencies
echo "Installing dependencies..."
case "$DISTRO" in
    arch|cachyos|manjaro|endeavouros)
        sudo pacman -S --needed --noconfirm playerctl pipewire-pulse || true
        ;;
    debian|ubuntu|pop)
        sudo apt-get update
        sudo apt-get install -y playerctl pipewire-pulseaudio || true
        ;;
    fedora)
        sudo dnf install -y playerctl pipewire-pulseaudio || true
        ;;
    *)
        echo "Unknown distribution. Please install playerctl and PipeWire/PulseAudio manually."
        ;;
esac

# Install udev rules
echo "Installing udev rules..."
sudo curl -fsSL -o /etc/udev/rules.d/40-streamdeck.rules https://raw.githubusercontent.com/OpenActionAPI/rust-elgato-streamdeck/main/40-streamdeck.rules
sudo udevadm control --reload-rules
sudo udevadm trigger

echo ""
echo "OpenDeckNG installation complete!"
echo ""
echo "Next steps:"
echo "1. Download the latest release from https://github.com/stefor74/OpenDeckNG/releases"
echo "2. Or install from AUR: yay -S opendeckng"
echo "3. Log out and back in for udev rules to take full effect"
