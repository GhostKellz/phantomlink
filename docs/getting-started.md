# Getting Started with PhantomLink

This guide will help you install and configure PhantomLink for the first time.

## System Requirements

### Minimum
- Linux kernel 5.10+
- ALSA or PipeWire audio
- 4GB RAM
- Any x86_64 CPU

### Recommended (for AI features)
- Linux kernel 6.0+
- PipeWire (with WirePlumber)
- 8GB RAM
- NVIDIA RTX 20 series or newer
- nvidia-open driver 545+

### Optimal (for best AI performance)
- Linux kernel 6.6+
- PipeWire 1.0+
- 16GB RAM
- NVIDIA RTX 40/50 series
- nvidia-open driver 580+

## Installation

Choose your distribution:

### Arch Linux
```bash
# From AUR
yay -S phantomlink

# Or with paru
paru -S phantomlink
```

### Fedora
```bash
sudo dnf copr enable ghostkellz/phantomlink
sudo dnf install phantomlink
```

### Ubuntu / Debian
```bash
# Download the .deb package from releases
sudo apt install ./phantomlink_0.2.0_amd64.deb
```

### Pop!_OS
```bash
# Pop!_OS uses apt
sudo apt install ./phantomlink_0.2.0_amd64.deb
```

### AppImage (Universal)
```bash
chmod +x PhantomLink-0.2.0-x86_64.AppImage
./PhantomLink-0.2.0-x86_64.AppImage
```

## First Launch

1. **Connect your audio interface** (Scarlett Solo recommended)
2. **Launch PhantomLink**
   ```bash
   phantomlink
   ```
3. PhantomLink will auto-detect your Scarlett Solo

## Initial Configuration

### Audio Interface Setup

If PhantomLink detects your Scarlett Solo:
1. The interface panel will show "Connected"
2. Configure your input settings:
   - **Phantom Power**: Enable for condenser mics
   - **Air Mode**: Choose Off, Presence, or Presence + Drive
   - **Input Level**: Line or Instrument

### GhostWave AI Setup

If you have an RTX GPU:
1. GhostWave will auto-detect your GPU
2. Select a processing profile:
   - **XLR Studio**: Low latency, preserves voice character
   - **Streaming**: Strong suppression, echo cancellation
   - **Balanced**: Good balance for general use
   - **Music**: Minimal processing for instruments
3. Adjust suppression strength as needed

### Mixer Configuration

1. Add channels for your inputs
2. Configure gain and pan per channel
3. Set up output routing
4. Add VST plugins if desired

## Verifying Setup

### Check RTX Acceleration
Look in the GhostWave panel for:
- "RTX 5090 (Blackwell) - FP4" for RTX 50 series
- "RTX 4090 (Ada) - FP16" for RTX 40 series
- "CPU Fallback" if no RTX detected

### Check Scarlett Connection
The Scarlett panel should show:
- Device name: "Scarlett Solo 4th Gen"
- Card number
- Firmware version
- Hardware level meters active

## Troubleshooting

### Scarlett Not Detected
1. Check USB connection
2. Verify device appears in `aplay -l`
3. Ensure you have permissions: `sudo usermod -aG audio $USER`
4. Log out and back in

### RTX Not Detected
1. Check NVIDIA driver: `nvidia-smi`
2. Ensure nvidia-open driver is installed
3. Verify CUDA toolkit: `nvcc --version`
4. Check driver version is 545+

### Audio Issues
1. Check PipeWire/ALSA is running
2. Verify sample rate matches (48kHz recommended)
3. Check buffer size (256 samples recommended)

## Next Steps

- [Features Overview](features.md) - Learn about all features
- [Scarlett Solo Setup](scarlett-setup.md) - Detailed interface configuration
- [GhostWave AI](ghostwave.md) - AI noise suppression tuning
