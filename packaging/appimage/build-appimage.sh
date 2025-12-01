#!/bin/bash
# Build PhantomLink AppImage
# Run from project root: ./packaging/appimage/build-appimage.sh

set -e

VERSION="0.2.0"
APPDIR="AppDir"

echo "Building PhantomLink $VERSION AppImage..."

# Build release binary
cargo build --release --locked

# Create AppDir structure
rm -rf $APPDIR
mkdir -p $APPDIR/usr/bin
mkdir -p $APPDIR/usr/share/applications
mkdir -p $APPDIR/usr/share/icons/hicolor/256x256/apps
mkdir -p $APPDIR/usr/share/icons/hicolor/scalable/apps

# Copy binary
cp target/release/phantomlink $APPDIR/usr/bin/

# Copy desktop file
cp packaging/phantomlink.desktop $APPDIR/usr/share/applications/
cp packaging/phantomlink.desktop $APPDIR/

# Copy icons
mkdir -p $APPDIR/usr/share/icons/hicolor/{16x16,32x32,48x48,64x64,128x128,256x256,512x512}/apps
cp assets/icons/icon-16x16.png $APPDIR/usr/share/icons/hicolor/16x16/apps/phantomlink.png
cp assets/icons/icon-32x32.png $APPDIR/usr/share/icons/hicolor/32x32/apps/phantomlink.png
cp assets/icons/icon-48x48.png $APPDIR/usr/share/icons/hicolor/48x48/apps/phantomlink.png
cp assets/icons/icon-64x64.png $APPDIR/usr/share/icons/hicolor/64x64/apps/phantomlink.png
cp assets/icons/icon-128x128.png $APPDIR/usr/share/icons/hicolor/128x128/apps/phantomlink.png
cp assets/icons/icon-256x256.png $APPDIR/usr/share/icons/hicolor/256x256/apps/phantomlink.png
cp assets/icons/icon-512x512.png $APPDIR/usr/share/icons/hicolor/512x512/apps/phantomlink.png
# Root icon for AppImage
cp assets/icons/icon-256x256.png $APPDIR/phantomlink.png

# Create AppRun
cat > $APPDIR/AppRun << 'EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${HERE}/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH}"
exec "${HERE}/usr/bin/phantomlink" "$@"
EOF
chmod +x $APPDIR/AppRun

# Download appimagetool if not present
if [ ! -f appimagetool-x86_64.AppImage ]; then
    echo "Downloading appimagetool..."
    wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x appimagetool-x86_64.AppImage
fi

# Create AppImage
ARCH=x86_64 ./appimagetool-x86_64.AppImage $APPDIR PhantomLink-$VERSION-x86_64.AppImage

echo "Created PhantomLink-$VERSION-x86_64.AppImage"
