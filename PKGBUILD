# Maintainer: Christopher Kelley <ckelley@ghostkellz.sh>
# Contributor: CK Technology LLC <ckelley@ghostkellz.sh>

pkgname=phantomlink-git
pkgver=0.3.0.r0.g$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
pkgrel=1
pkgdesc="Professional Audio Mixer for Linux - True Wavelink Alternative with Application Routing"
arch=('x86_64')
url="https://github.com/ghostkellz/phantomlink"
license=('MIT')
depends=('alsa-lib' 'gcc-libs' 'glibc' 'pulseaudio')
makedepends=('rust' 'cargo' 'git' 'pkgconf' 'alsa-lib')
optdepends=(
    'jack2: JACK audio server support for low-latency professional audio'
    'pipewire-jack: PipeWire JACK compatibility layer'
    'pipewire-pulse: PipeWire PulseAudio compatibility for application routing'
    'vst-plugins: Additional VST plugin support'
    'ladspa-plugins: LADSPA plugin support'
    'focusrite-scarlett-solo: Hardware support for Scarlett Solo interface'
)
provides=('phantomlink')
conflicts=('phantomlink')
source=("git+https://github.com/ghostkellz/phantomlink.git")
sha256sums=('SKIP')

pkgver() {
    cd "$srcdir/phantomlink"
    printf "0.3.0.r%s.g%s" \
        "$(git rev-list --count HEAD)" \
        "$(git rev-parse --short HEAD)"
}

prepare() {
    cd "$srcdir/phantomlink"
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
    cd "$srcdir/phantomlink"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    
    # Build with optimizations for release
    cargo build \
        --frozen \
        --release \
        --all-features \
        --target "$CARCH-unknown-linux-gnu"
}

check() {
    cd "$srcdir/phantomlink"
    export RUSTUP_TOOLCHAIN=stable
    
    # Run tests (skip GUI tests that require display)
    cargo test --frozen --release --lib --bins
}

package() {
    cd "$srcdir/phantomlink"
    
    # Install the binary
    install -Dm755 "target/$CARCH-unknown-linux-gnu/release/phantomlink" \
        "$pkgdir/usr/bin/phantomlink"
    
    # Install desktop entry
    install -Dm644 <(cat << 'EOF'
[Desktop Entry]
Type=Application
Name=PhantomLink
Comment=Professional Audio Mixer for Linux
Exec=phantomlink
Icon=phantomlink
Categories=AudioVideo;Audio;Mixer;
Terminal=false
StartupNotify=true
Keywords=audio;mixer;professional;recording;streaming;noise;suppression;
EOF
) "$pkgdir/usr/share/applications/phantomlink.desktop"
    
    # Install icons
    for size in 16 32 48 64 128 256 512; do
        install -Dm644 "assets/icons/icon-${size}x${size}.png" \
            "$pkgdir/usr/share/icons/hicolor/${size}x${size}/apps/phantomlink.png"
    done
    
    # Install main icon
    install -Dm644 "assets/icons/phantomlink_icon.png" \
        "$pkgdir/usr/share/pixmaps/phantomlink.png"
    
    # Install documentation
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 CHANGELOG.md "$pkgdir/usr/share/doc/$pkgname/CHANGELOG.md"
    
    # Install license
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    
    # Install example configuration (if it exists)
    if [ -f "examples/phantomlink_config.json" ]; then
        install -Dm644 "examples/phantomlink_config.json" \
            "$pkgdir/usr/share/doc/$pkgname/phantomlink_config.json.example"
    fi
}

# vim:set ts=4 sw=4 et:
