# Installing PhantomLink on Ubuntu / Debian

## From .deb Package (Recommended)

Download the .deb package from the releases page:

```bash
# Ubuntu 22.04+, Debian 12+
wget https://github.com/ghostkellz/phantomlink/releases/latest/download/phantomlink_0.4.0_amd64.deb

# Install
sudo apt install ./phantomlink_0.4.0_amd64.deb
```

## From PPA (Ubuntu)

```bash
# Add PPA
sudo add-apt-repository ppa:ghostkellz/phantomlink
sudo apt update

# Install
sudo apt install phantomlink
```

## Dependencies

### Required (installed automatically)
- `libasound2` - ALSA library
- `libpipewire-0.3-0` - PipeWire library
- `libxkbcommon0` - Keyboard handling library

### Optional (for RTX acceleration)

#### NVIDIA Driver (Ubuntu)
```bash
# Add graphics drivers PPA
sudo add-apt-repository ppa:graphics-drivers/ppa
sudo apt update

# Install latest driver
sudo apt install nvidia-driver-545
```

#### NVIDIA Driver (Debian)
```bash
# Enable non-free repository
sudo sed -i 's/main/main contrib non-free non-free-firmware/' /etc/apt/sources.list
sudo apt update

# Install driver
sudo apt install nvidia-driver
```

#### CUDA Toolkit
```bash
# Ubuntu
sudo apt install nvidia-cuda-toolkit

# Or from NVIDIA directly
wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-keyring_1.0-1_all.deb
sudo dpkg -i cuda-keyring_1.0-1_all.deb
sudo apt update
sudo apt install cuda-toolkit
```

## Post-Installation

### Audio Group
```bash
sudo usermod -aG audio $USER
```
Log out and back in for changes to take effect.

### Real-time Scheduling (Optional)
```bash
# Install real-time configuration
sudo apt install jackd2

# During installation, allow real-time privileges when prompted
# Or manually configure:
sudo dpkg-reconfigure jackd2
```

Alternatively, add to `/etc/security/limits.d/audio.conf`:
```
@audio   -  rtprio     95
@audio   -  memlock    unlimited
```

### PipeWire Setup (Ubuntu 22.04+)
Ubuntu 22.04+ uses PipeWire by default. Verify:
```bash
pactl info | grep "Server Name"
# Should show: PipeWire
```

For older Ubuntu with PulseAudio:
```bash
# Install PipeWire
sudo apt install pipewire pipewire-pulse wireplumber

# Disable PulseAudio, enable PipeWire
systemctl --user disable --now pulseaudio.service pulseaudio.socket
systemctl --user enable --now pipewire pipewire-pulse wireplumber
```

## Running PhantomLink

```bash
phantomlink
```

Or find it in your applications menu.

## Updating

### From .deb
```bash
sudo apt install ./phantomlink_*_amd64.deb
```

### From PPA
```bash
sudo apt update && sudo apt upgrade phantomlink
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
# Check device
aplay -l | grep -i scarlett

# Check permissions
ls -la /dev/snd/

# Verify group membership
groups | grep audio

# If not in audio group, add yourself
sudo usermod -aG audio $USER
# Then log out and back in
```

### NVIDIA Driver Issues
```bash
# Check driver
nvidia-smi

# If not working, try reinstalling
sudo apt remove --purge nvidia-*
sudo apt autoremove
sudo apt install nvidia-driver-545
sudo reboot
```

### PipeWire Issues
```bash
# Check status
systemctl --user status pipewire

# Restart
systemctl --user restart pipewire pipewire-pulse wireplumber

# Check for errors
journalctl --user -u pipewire -n 50
```

### AppArmor Issues
If PhantomLink can't access audio devices:
```bash
# Check for denials
sudo dmesg | grep -i apparmor | tail -20

# If needed, set PhantomLink to complain mode
sudo aa-complain /usr/bin/phantomlink
```
