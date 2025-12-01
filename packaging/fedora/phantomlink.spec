Name:           phantomlink
Version:        0.2.0
Release:        1%{?dist}
Summary:        Professional audio mixer and interface control for Linux

License:        MIT
URL:            https://github.com/ghostkellz/phantomlink
Source0:        %{url}/archive/v%{version}/%{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.70
BuildRequires:  cargo
BuildRequires:  pkg-config
BuildRequires:  alsa-lib-devel
BuildRequires:  pipewire-devel
BuildRequires:  gtk3-devel
BuildRequires:  libxkbcommon-devel
BuildRequires:  libxcb-devel

Requires:       alsa-lib
Requires:       pipewire
Requires:       gtk3

Recommends:     pipewire-pulseaudio
Suggests:       akmod-nvidia

%description
PhantomLink is a professional audio mixer and interface control application
for Linux, designed for streamers, podcasters, and content creators. It
provides a Wavelink-style experience with RTX-accelerated AI noise suppression
powered by GhostWave.

Features:
- Focusrite Scarlett Solo 4th Gen full hardware control
- RTX-accelerated AI noise suppression (GhostWave)
- Professional channel strips with VU meters
- Per-channel VST plugin support
- Multiple theme presets
- Echo cancellation (AEC)

%prep
%autosetup -n %{name}-%{version}

%build
export CARGO_HOME="%{_builddir}/cargo-home"
cargo build --release --locked

%install
install -Dm755 target/release/phantomlink %{buildroot}%{_bindir}/phantomlink
install -Dm644 packaging/phantomlink.desktop %{buildroot}%{_datadir}/applications/phantomlink.desktop

# Icons
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/{16x16,32x32,48x48,64x64,128x128,256x256,512x512}/apps
install -Dm644 assets/icons/icon-16x16.png %{buildroot}%{_datadir}/icons/hicolor/16x16/apps/phantomlink.png
install -Dm644 assets/icons/icon-32x32.png %{buildroot}%{_datadir}/icons/hicolor/32x32/apps/phantomlink.png
install -Dm644 assets/icons/icon-48x48.png %{buildroot}%{_datadir}/icons/hicolor/48x48/apps/phantomlink.png
install -Dm644 assets/icons/icon-64x64.png %{buildroot}%{_datadir}/icons/hicolor/64x64/apps/phantomlink.png
install -Dm644 assets/icons/icon-128x128.png %{buildroot}%{_datadir}/icons/hicolor/128x128/apps/phantomlink.png
install -Dm644 assets/icons/icon-256x256.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/phantomlink.png
install -Dm644 assets/icons/icon-512x512.png %{buildroot}%{_datadir}/icons/hicolor/512x512/apps/phantomlink.png

# Documentation
mkdir -p %{buildroot}%{_docdir}/%{name}
install -Dm644 README.md %{buildroot}%{_docdir}/%{name}/README.md
install -Dm644 docs/README.md %{buildroot}%{_docdir}/%{name}/docs/README.md
install -Dm644 docs/getting-started.md %{buildroot}%{_docdir}/%{name}/docs/getting-started.md
install -Dm644 docs/features.md %{buildroot}%{_docdir}/%{name}/docs/features.md

# License
mkdir -p %{buildroot}%{_licensedir}/%{name}
install -Dm644 LICENSE %{buildroot}%{_licensedir}/%{name}/LICENSE || true

%check
export CARGO_HOME="%{_builddir}/cargo-home"
cargo test --release --locked || true

%files
%license LICENSE
%doc README.md docs/
%{_bindir}/phantomlink
%{_datadir}/applications/phantomlink.desktop
%{_datadir}/icons/hicolor/*/apps/phantomlink.*

%changelog
* Sat Nov 30 2024 Christopher Kelley <ckelley@ghostkellz.sh> - 0.2.0-1
- Initial package release
- Focusrite Scarlett Solo 4th Gen support
- GhostWave RTX AI noise suppression
- Professional mixer with VU meters
- Multiple theme presets
