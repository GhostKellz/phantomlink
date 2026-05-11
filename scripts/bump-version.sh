#!/bin/bash
# Propagate version from Cargo.toml to all packaging and doc files.
# Usage: ./scripts/bump-version.sh
#
# Cargo.toml is the single source of truth. Update version there first,
# then run this script to propagate everywhere.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

if [ -z "$VERSION" ]; then
    echo "ERROR: Could not read version from Cargo.toml"
    exit 1
fi

echo "Propagating version $VERSION from Cargo.toml..."

# Track what we update
updated=()

# --- Packaging files ---

# packaging/arch/PKGBUILD (release PKGBUILD)
if [ -f packaging/arch/PKGBUILD ]; then
    sed -i "s/^pkgver=.*/pkgver=$VERSION/" packaging/arch/PKGBUILD
    updated+=("packaging/arch/PKGBUILD")
fi

# packaging/arch/.SRCINFO (update version and source URL)
if [ -f packaging/arch/.SRCINFO ]; then
    sed -i "s/pkgver = .*/pkgver = $VERSION/" packaging/arch/.SRCINFO
    sed -i "s|phantomlink-[0-9]\+\.[0-9]\+\.[0-9]\+\.tar\.gz|phantomlink-${VERSION}.tar.gz|g" packaging/arch/.SRCINFO
    sed -i "s|/v[0-9]\+\.[0-9]\+\.[0-9]\+\.tar\.gz|/v${VERSION}.tar.gz|g" packaging/arch/.SRCINFO
    updated+=("packaging/arch/.SRCINFO")
fi

# packaging/appimage/AppImageBuilder.yml
if [ -f packaging/appimage/AppImageBuilder.yml ]; then
    sed -i "s/version: .*/version: $VERSION/" packaging/appimage/AppImageBuilder.yml
    updated+=("packaging/appimage/AppImageBuilder.yml")
fi

# packaging/fedora/phantomlink.spec
if [ -f packaging/fedora/phantomlink.spec ]; then
    sed -i "s/^Version:.*/Version:        $VERSION/" packaging/fedora/phantomlink.spec
    updated+=("packaging/fedora/phantomlink.spec")
fi

# --- Documentation: download URLs and filenames ---

# .deb references
find docs/ -name '*.md' -exec sed -i \
    "s/phantomlink_[0-9]\+\.[0-9]\+\.[0-9]\+_amd64\.deb/phantomlink_${VERSION}_amd64.deb/g" {} +

# .rpm references
find docs/ -name '*.md' -exec sed -i \
    "s/phantomlink-[0-9]\+\.[0-9]\+\.[0-9]\+-1\.fc\*/phantomlink-${VERSION}-1.fc*/g" {} +

# AppImage references
find docs/ -name '*.md' -exec sed -i \
    "s/PhantomLink-[0-9]\+\.[0-9]\+\.[0-9]\+-x86_64\.AppImage/PhantomLink-${VERSION}-x86_64.AppImage/g" {} +

updated+=("docs/ (download URLs)")

echo ""
echo "Updated:"
for f in "${updated[@]}"; do
    echo "  - $f"
done

echo ""
echo "Files NOT auto-updated (require manual changelog entries):"
echo "  - packaging/debian/changelog  (add new version entry)"
echo "  - packaging/fedora/phantomlink.spec %changelog section"
echo "  - CHANGELOG.md"
echo ""
echo "Done. Run 'git diff' to verify changes."
