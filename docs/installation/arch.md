# Installing PhantomLink on Arch Linux

## From AUR (Recommended)

### Using yay
```bash
yay -S phantomlink
```

### Using paru
```bash
paru -S phantomlink
```

### Manual AUR Installation
```bash
git clone https://aur.archlinux.org/phantomlink.git
cd phantomlink
makepkg -si
```

## Dependencies

PhantomLink will automatically pull these dependencies:

### Required
- `alsa-lib` - ALSA audio library
- `pipewire` - Audio server (recommended over PulseAudio)
- `gtk3` - GUI toolkit

### Optional (for RTX acceleration)
- `nvidia-open` - NVIDIA open kernel driver (545+)
- `cuda` - NVIDIA CUDA toolkit
- `cudnn` - CUDA Deep Neural Network library

## Post-Installation

### Audio Group
Add yourself to the audio group for direct hardware access:
```bash
sudo usermod -aG audio $USER
```
Log out and back in for changes to take effect.

### Real-time Scheduling (Optional)
For lowest latency, enable real-time scheduling:

1. Install `realtime-privileges`:
   ```bash
   sudo pacman -S realtime-privileges
   ```

2. Add yourself to the realtime group:
   ```bash
   sudo usermod -aG realtime $USER
   ```

3. Log out and back in.

### NVIDIA Driver Setup (for RTX features)

1. Install nvidia-open driver:
   ```bash
   sudo pacman -S nvidia-open
   ```

2. Reboot to load the new driver:
   ```bash
   sudo reboot
   ```

3. Verify driver:
   ```bash
   nvidia-smi
   ```

## Running PhantomLink

```bash
phantomlink
```

Or launch from your application menu under "Audio" or "Multimedia".

## Updating

```bash
yay -Syu phantomlink
```

## Uninstalling

```bash
yay -R phantomlink
```

To remove configuration files:
```bash
rm -rf ~/.config/phantomlink
rm -rf ~/.local/share/phantomlink
```

## Troubleshooting

### Scarlett Solo Not Detected
```bash
# Check if device appears
aplay -l | grep -i scarlett

# Check ALSA permissions
ls -la /dev/snd/

# Ensure you're in audio group
groups | grep audio
```

### RTX Acceleration Not Working
```bash
# Check NVIDIA driver
nvidia-smi

# Verify nvidia-open module
lsmod | grep nvidia

# Check CUDA
nvcc --version
```

### Audio Crackling/Xruns
1. Lower buffer size in PhantomLink settings
2. Enable real-time scheduling (see above)
3. Set CPU governor to performance:
   ```bash
   sudo cpupower frequency-set -g performance
   ```

## Building from Source

```bash
# Install build dependencies
sudo pacman -S rust cargo alsa-lib pipewire gtk3

# Clone repository
git clone https://github.com/ghostkellz/phantomlink.git
cd phantomlink

# Build
cargo build --release

# Install
sudo install -Dm755 target/release/phantomlink /usr/bin/phantomlink
```
