# Installing PhantomLink on Pop!_OS

Pop!_OS is based on Ubuntu, so installation is similar with some Pop-specific considerations.

## From .deb Package (Recommended)

```bash
# Download the .deb package
wget https://github.com/ghostkellz/phantomlink/releases/latest/download/phantomlink_0.4.0_amd64.deb

# Install using apt
sudo apt install ./phantomlink_0.4.0_amd64.deb
```

Or double-click the .deb file to install via Eddy (Pop's package installer).

## Dependencies

Dependencies are automatically installed:
- `libasound2` - ALSA library
- `libpipewire-0.3-0` - PipeWire library
- `libxkbcommon0` - Keyboard handling library

## NVIDIA Driver (Pop!_OS Advantage)

Pop!_OS has excellent NVIDIA support out of the box!

### If you have the NVIDIA ISO variant
Your driver is already installed. Verify with:
```bash
nvidia-smi
```

### If you need to install/update the driver
```bash
# Via Pop!_Shop or command line
sudo apt install system76-driver-nvidia
```

### Switching Graphics Modes
Pop!_OS allows easy GPU switching:
```bash
# Check current mode
system76-power graphics

# Switch to NVIDIA (for RTX acceleration)
sudo system76-power graphics nvidia

# Switch to Hybrid (power saving)
sudo system76-power graphics hybrid
```

For best PhantomLink RTX performance, use NVIDIA mode.

## Post-Installation

### Audio Group
```bash
sudo usermod -aG audio $USER
```
Log out and back in.

### Real-time Scheduling
```bash
# Add real-time privileges
sudo apt install jackd2
sudo dpkg-reconfigure jackd2
# Select "Yes" when asked about real-time privileges
```

## Running PhantomLink

Launch from:
- **Pop!_Shell**: Press `Super`, type "PhantomLink"
- **Applications**: Find under "Sound & Video"
- **Terminal**: `phantomlink`

## NVIDIA RTX Acceleration

Pop!_OS + NVIDIA is an ideal combination for PhantomLink:

1. Ensure you're in NVIDIA graphics mode:
   ```bash
   system76-power graphics nvidia
   ```

2. Verify RTX is detected:
   ```bash
   nvidia-smi
   ```

3. Launch PhantomLink - GhostWave should show your RTX GPU

## Pop!_Shop Installation (Coming Soon)

We're working on getting PhantomLink into the Pop!_Shop for one-click installation.

## Updating

```bash
sudo apt install ./phantomlink_*_amd64.deb
```

## Uninstalling

```bash
sudo apt remove phantomlink

# Remove configuration
rm -rf ~/.config/phantomlink
rm -rf ~/.local/share/phantomlink
```

## Building from Source

```bash
# Install build dependencies
sudo apt install build-essential pkg-config libssl-dev \
    libasound2-dev libpipewire-0.3-dev libxkbcommon-dev libxcb1-dev libjack-jackd2-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and build
git clone https://github.com/ghostkellz/phantomlink.git
cd phantomlink
cargo build --release

# Install
sudo install -Dm755 target/release/phantomlink /usr/bin/phantomlink
```

## Troubleshooting

### Scarlett Solo Not Detected
```bash
# Check device appears
aplay -l | grep -i scarlett

# Verify audio group
groups | grep audio

# If not in group
sudo usermod -aG audio $USER
# Log out and back in
```

### RTX Not Working
```bash
# Check graphics mode
system76-power graphics

# Ensure NVIDIA mode
sudo system76-power graphics nvidia
# Log out and back in

# Verify driver
nvidia-smi
```

### PipeWire Issues
Pop!_OS 22.04+ uses PipeWire by default:
```bash
# Check status
systemctl --user status pipewire

# Restart if needed
systemctl --user restart pipewire pipewire-pulse wireplumber
```

### COSMIC Desktop (Pop!_OS 24.04+)
PhantomLink works with COSMIC desktop. If you experience any issues:
```bash
# Check for Wayland compatibility
echo $XDG_SESSION_TYPE

# PhantomLink supports both X11 and Wayland
```
