# Installing PhantomLink on Fedora

## From COPR (Recommended)

```bash
# Enable the COPR repository
sudo dnf copr enable ghostkellz/phantomlink

# Install PhantomLink
sudo dnf install phantomlink
```

## Manual RPM Installation

Download the RPM from the releases page and install:
```bash
sudo dnf install ./phantomlink-0.4.0-1.fc*.x86_64.rpm
```

## Dependencies

### Required (installed automatically)
- `alsa-lib` - ALSA audio library
- `pipewire` - Audio server
- `libxkbcommon` - Keyboard handling library

### Optional (for RTX acceleration)
```bash
# Enable RPM Fusion (if not already)
sudo dnf install https://mirrors.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm
sudo dnf install https://mirrors.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm

# Install NVIDIA drivers
sudo dnf install akmod-nvidia

# For CUDA support
sudo dnf install cuda
```

## Post-Installation

### Audio Group
```bash
sudo usermod -aG audio $USER
```
Log out and back in.

### Real-time Scheduling (Optional)
```bash
# Install realtime configuration
sudo dnf install realtime-setup

# Add yourself to realtime group
sudo usermod -aG realtime $USER
```

### SELinux (if issues occur)
If you experience permission issues:
```bash
# Check for SELinux denials
sudo ausearch -m avc -ts recent

# If needed, create a policy exception
sudo audit2allow -a -M phantomlink
sudo semodule -i phantomlink.pp
```

## Running PhantomLink

```bash
phantomlink
```

Or find it in your applications menu under "Sound & Video".

## Updating

```bash
sudo dnf upgrade phantomlink
```

## Uninstalling

```bash
sudo dnf remove phantomlink
```

Remove configuration:
```bash
rm -rf ~/.config/phantomlink
rm -rf ~/.local/share/phantomlink
```

## Building from Source

```bash
# Install build dependencies
sudo dnf install rust cargo alsa-lib-devel pipewire-devel libxkbcommon-devel libxcb-devel jack-audio-connection-kit-devel

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
# Check device
aplay -l | grep -i scarlett

# Check permissions
ls -la /dev/snd/

# Check group membership
groups | grep audio
```

### NVIDIA/RTX Issues
```bash
# Check driver status
nvidia-smi

# Verify module loaded
lsmod | grep nvidia

# Check for akmod issues
sudo akmods --force
```

### PipeWire Issues
```bash
# Check PipeWire status
systemctl --user status pipewire

# Restart if needed
systemctl --user restart pipewire pipewire-pulse
```
