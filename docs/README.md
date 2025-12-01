# PhantomLink Documentation

PhantomLink is a professional audio mixer and interface control application for Linux, designed for streamers, podcasters, and content creators. It provides a Wavelink-style experience with RTX-accelerated AI noise suppression.

## Documentation Index

### User Guides
- [Getting Started](getting-started.md) - Installation and initial setup
- [Features Overview](features.md) - Complete feature documentation
- [Scarlett Solo Setup](scarlett-setup.md) - Focusrite Scarlett Solo 4th Gen configuration
- [GhostWave AI](ghostwave.md) - RTX-accelerated noise suppression

### Installation
- [Arch Linux](installation/arch.md)
- [Fedora](installation/fedora.md)
- [Ubuntu/Debian](installation/ubuntu.md)
- [Pop!_OS](installation/pop-os.md)
- [AppImage](installation/appimage.md)

### Technical Documentation
- [Advanced Denoising](ADVANCED_DENOISING.md)
- [GUI Architecture](GUI_REDESIGN.md)
- [VST Plugin Support](VST_IMPLEMENTATION.md)

## Quick Start

```bash
# Arch Linux (AUR)
yay -S phantomlink

# Fedora (COPR)
sudo dnf copr enable ghostkellz/phantomlink
sudo dnf install phantomlink

# Ubuntu/Debian
sudo apt install ./phantomlink_*.deb

# Universal (AppImage)
chmod +x PhantomLink-*.AppImage
./PhantomLink-*.AppImage
```

## Hardware Support

### Audio Interfaces
- **Focusrite Scarlett Solo 4th Gen** (Full support)
  - 48V Phantom Power
  - Air Mode (Presence / Presence + Drive)
  - Input Level (Line / Instrument)
  - Direct Monitoring
  - DSP Routing (Mix A-F)
  - Hardware Level Metering

### GPU Acceleration
- **NVIDIA RTX 50 Series (Blackwell)** - FP4 Tensor Core acceleration
- **NVIDIA RTX 40 Series (Ada Lovelace)** - FP16 Tensor Core acceleration
- **NVIDIA RTX 30 Series (Ampere)** - FP16 Tensor Core acceleration
- **NVIDIA RTX 20 Series (Turing)** - FP16 acceleration
- CPU fallback for non-RTX systems

## Features

### AI Noise Suppression (GhostWave)
- Real-time RTX-accelerated denoising
- NVIDIA Broadcast quality on Linux
- Multiple quality profiles (Fast, Balanced, Quality, Ultra)
- Echo cancellation (AEC)
- Voice activity detection

### Mixer
- Professional channel strips with VU meters
- Hardware-style gain knobs
- Peak hold with clip indicators
- Per-channel VST plugin support
- Mute/Solo per channel
- Application audio routing

### Themes
- Tokyo Night
- Catppuccin Mocha
- Dracula
- Scarlett (Focusrite-inspired)
- Wavelink (Elgato-inspired)

## License

MIT License - See [LICENSE](../LICENSE) for details.

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Support

- Issues: https://github.com/ghostkellz/phantomlink/issues
- Maintainer: Christopher Kelley <ckelley@ghostkellz.sh>
