#!/bin/bash

# PhantomLink Installation Script

set -e

echo "🎛️ PhantomLink - Professional Audio Mixer Installation"
echo "================================================="

# Check for dependencies
echo "Checking system dependencies..."

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check for ALSA development libraries
if ! pkg-config --exists alsa; then
    echo "❌ ALSA development libraries not found."
    echo "   Ubuntu/Debian: sudo apt install libasound2-dev"
    echo "   Fedora: sudo dnf install alsa-lib-devel"
    echo "   Arch: sudo pacman -S alsa-lib"
    exit 1
fi

# Check for JACK (optional)
if pkg-config --exists jack; then
    echo "✅ JACK Audio Connection Kit found - low latency audio available"
else
    echo "⚠️  JACK not found - using ALSA only (higher latency)"
fi

# Check for PulseAudio utilities
if command -v pactl &> /dev/null; then
    echo "✅ PulseAudio found - application routing available"
else
    echo "⚠️  PulseAudio not found - limited application routing"
fi

echo ""
echo "Building PhantomLink..."

# Build in release mode
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
else
    echo "❌ Build failed!"
    exit 1
fi

echo ""
echo "Installing PhantomLink..."

# Create installation directory
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

# Copy binary
cp target/release/phantomlink "$INSTALL_DIR/"

# Make executable
chmod +x "$INSTALL_DIR/phantomlink"

# Add to PATH if not already there
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "Adding $INSTALL_DIR to PATH..."
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
    echo "Please run: source ~/.bashrc or restart your terminal"
fi

# Create desktop entry
DESKTOP_DIR="$HOME/.local/share/applications"
mkdir -p "$DESKTOP_DIR"

cat > "$DESKTOP_DIR/phantomlink.desktop" << EOF
[Desktop Entry]
Name=PhantomLink
Comment=Professional Audio Mixer for Linux
Exec=$INSTALL_DIR/phantomlink
Icon=audio-card
Terminal=false
Type=Application
Categories=AudioVideo;Audio;Mixer;
Keywords=audio;mixer;vst;jack;recording;streaming;
EOF

echo ""
echo "🎉 PhantomLink installed successfully!"
echo ""
echo "Quick Start:"
echo "1. Launch: phantomlink"
echo "2. Click 'START' to begin audio processing"
echo "3. Go to 'Applications' tab and click 'Scan Applications'"
echo "4. Route Discord to headphones, games to both headphones + stream"
echo ""
echo "Hardware Setup (Scarlett Solo):"
echo "1. Connect your Scarlett Solo via USB"
echo "2. Set it as default audio device in system settings"
echo "3. Use the hardware controls in PhantomLink's Mixer tab"
echo ""
echo "For VST plugins:"
echo "1. Install VST plugins to ~/.vst or /usr/lib/vst"
echo "2. Restart PhantomLink to scan for new plugins"
echo ""
echo "Enjoy your professional audio setup! 🎵"