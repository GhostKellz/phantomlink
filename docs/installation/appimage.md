# Installing PhantomLink via AppImage

AppImage is a universal Linux package format that works on any distribution without installation.

## Download

Download the latest AppImage from the releases page:
```bash
wget https://github.com/ghostkellz/phantomlink/releases/latest/download/PhantomLink-0.2.0-x86_64.AppImage
```

## Make Executable

```bash
chmod +x PhantomLink-0.2.0-x86_64.AppImage
```

## Run

```bash
./PhantomLink-0.2.0-x86_64.AppImage
```

## Optional: System Integration

### Using AppImageLauncher (Recommended)

AppImageLauncher automatically integrates AppImages into your system:

**Arch Linux:**
```bash
yay -S appimagelauncher
```

**Ubuntu/Debian:**
```bash
sudo add-apt-repository ppa:appimagelauncher-team/stable
sudo apt update
sudo apt install appimagelauncher
```

**Fedora:**
```bash
sudo dnf install appimagelauncher
```

After installing AppImageLauncher, simply double-click the AppImage to integrate it.

### Manual Integration

1. Move to a permanent location:
   ```bash
   mkdir -p ~/.local/bin
   mv PhantomLink-*.AppImage ~/.local/bin/phantomlink.AppImage
   ```

2. Create desktop entry:
   ```bash
   cat > ~/.local/share/applications/phantomlink.desktop << 'EOF'
   [Desktop Entry]
   Name=PhantomLink
   GenericName=Audio Mixer
   Comment=Professional audio mixer with RTX AI noise suppression
   Exec=/home/$USER/.local/bin/phantomlink.AppImage
   Icon=phantomlink
   Terminal=false
   Type=Application
   Categories=Audio;AudioVideo;Mixer;
   EOF
   ```

3. Update desktop database:
   ```bash
   update-desktop-database ~/.local/share/applications/
   ```

## Dependencies

The AppImage bundles most dependencies, but you may need:

### Required (system libraries)
- ALSA library (usually pre-installed)
- GTK3 (usually pre-installed)

### For RTX Acceleration
Install your distro's NVIDIA driver:

**Arch:**
```bash
sudo pacman -S nvidia-open
```

**Fedora:**
```bash
sudo dnf install akmod-nvidia
```

**Ubuntu:**
```bash
sudo apt install nvidia-driver-545
```

## Updating

1. Download the new AppImage
2. Replace the old one:
   ```bash
   mv PhantomLink-*-x86_64.AppImage ~/.local/bin/phantomlink.AppImage
   ```

## Uninstalling

```bash
# Remove AppImage
rm ~/.local/bin/phantomlink.AppImage

# Remove desktop entry
rm ~/.local/share/applications/phantomlink.desktop

# Remove configuration (optional)
rm -rf ~/.config/phantomlink
rm -rf ~/.local/share/phantomlink
```

## Troubleshooting

### AppImage Won't Run
```bash
# Check if FUSE is installed
which fusermount

# Install FUSE if missing
# Arch:
sudo pacman -S fuse2

# Ubuntu/Debian:
sudo apt install fuse libfuse2

# Fedora:
sudo dnf install fuse
```

### Extraction Method (No FUSE)
If you can't install FUSE, extract the AppImage:
```bash
./PhantomLink-*.AppImage --appimage-extract
./squashfs-root/AppRun
```

### Audio Device Not Detected
The AppImage may need access to your audio devices:
```bash
# Ensure you're in the audio group
sudo usermod -aG audio $USER
# Log out and back in
```

### Permission Denied
```bash
chmod +x PhantomLink-*.AppImage
```

## Building AppImage from Source

```bash
# Clone repository
git clone https://github.com/ghostkellz/phantomlink.git
cd phantomlink

# Build AppImage
./packaging/appimage/build-appimage.sh
```
