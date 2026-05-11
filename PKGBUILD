# Maintainer: Christopher Kelley <ckelley@ghostkellz.sh>
# Contributor: CK Technology <ckelley@ghostkellz.sh>

pkgname=phantomlink-git
pkgver=0.4.0.r0.gunknown
pkgrel=1
pkgdesc="Professional Audio Mixer for Linux with RTX AI noise suppression"
arch=('x86_64')
url="https://github.com/ghostkellz/phantomlink"
license=('MIT')
depends=('alsa-lib' 'gcc-libs' 'glibc')
makedepends=('rust>=1.90' 'cargo' 'git' 'pkgconf' 'alsa-lib' 'jack2')
optdepends=(
    'jack2: JACK audio server support'
    'pipewire-jack: PipeWire JACK compatibility'
    'nvidia-open: RTX acceleration support (545+)'
)
provides=('phantomlink')
conflicts=('phantomlink')
source=("git+https://github.com/ghostkellz/phantomlink.git")
sha256sums=('SKIP')

pkgver() {
    cd "$srcdir/phantomlink"
    local _ver
    _ver=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
    printf "%s.r%s.g%s" \
        "$_ver" \
        "$(git rev-list --count HEAD)" \
        "$(git rev-parse --short HEAD)"
}

prepare() {
    cd "$srcdir/phantomlink"
    cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
    cd "$srcdir/phantomlink"
    export CARGO_TARGET_DIR=target

    cargo build \
        --frozen \
        --release \
        --all-features \
        --target "$CARCH-unknown-linux-gnu"
}

check() {
    cd "$srcdir/phantomlink"

    cargo test --frozen --release --lib --bins
}

package() {
    cd "$srcdir/phantomlink"

    # Install the binary
    install -Dm755 "target/$CARCH-unknown-linux-gnu/release/phantomlink" \
        "$pkgdir/usr/bin/phantomlink"

    # Install desktop entry
    install -Dm644 "packaging/phantomlink.desktop" \
        "$pkgdir/usr/share/applications/phantomlink.desktop"

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
