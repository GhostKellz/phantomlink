# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Switched ghostwave-core to git dependency (github.com/ghostkellz/ghostwave v0.3.0)
- Upgraded cpal 0.16 -> 0.17, alsa 0.7 -> 0.11, jack 0.12 -> 0.13
- Replaced multi-theme system with Tokyo Night only (Night/Moon/Storm variants)
- Removed ghoststream/video integration (will return in future release)
- Removed dead code, empty files, and unused deep-learning feature gate

### Fixed
- Build failure from libspa-sys link conflict between ghostwave-core and ghoststream
- All compiler warnings resolved (was 22+)

## [0.4.0] - 2025-12-15

### Added
- IPC server for external control
- VST parameter automation
- Multi-GPU selection support
- Buffer size control with custom values
- JACK audio backend integration
- Microphone presets (Rode PodMic, SM7B, etc.)
- Professional audio effects chain (Gate, Compressor, Limiter)

## [0.3.0] - 2025-11-01

### Added
- GhostWave AI noise cancellation integration
- RTX GPU acceleration support
- PipeWire virtual device manager
- Spectrum analyzer and waveform visualizer
- Focusrite Scarlett Solo 4th Gen full hardware control

## [0.2.0] - 2025-10-01

### Added
- Basic 4-channel audio mixer
- egui-based GUI with dark theme
- RNNoise noise suppression
- VST 2.4 plugin host
- Application audio routing

## [0.1.0] - 2025-09-01

### Added
- Initial release
- Basic audio engine with cpal
- Simple channel strips with volume/pan
