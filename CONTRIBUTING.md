# Contributing to PhantomLink

Thank you for your interest in contributing to PhantomLink! This document provides guidelines for contributing to the project.

## Code of Conduct

Be respectful and constructive. We're all here to make Linux audio better.

## Getting Started

### Prerequisites

- Rust 1.70+ (stable)
- Linux system with ALSA
- Git

### Development Setup

```bash
# Clone the repository
git clone https://github.com/ghostkellz/phantomlink.git
cd phantomlink

# Build
cargo build

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run
```

### Optional: Hardware for Testing

- Focusrite Scarlett Solo 4th Gen (for hardware integration testing)
- NVIDIA RTX GPU (for GhostWave AI testing)

## How to Contribute

### Reporting Bugs

1. Check existing issues first
2. Create a new issue with:
   - Clear title describing the bug
   - Steps to reproduce
   - Expected vs actual behavior
   - System info: distro, kernel, audio setup, GPU
   - Relevant logs (`RUST_LOG=debug`)

### Suggesting Features

1. Check existing issues/discussions
2. Create an issue with:
   - Clear description of the feature
   - Use case / why it's needed
   - Proposed implementation (optional)

### Submitting Code

1. **Fork** the repository
2. **Create a branch** for your feature/fix:
   ```bash
   git checkout -b feature/my-feature
   # or
   git checkout -b fix/issue-123
   ```
3. **Make your changes**
4. **Test** your changes:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```
5. **Commit** with a clear message:
   ```bash
   git commit -m "feat: add XYZ feature"
   # or
   git commit -m "fix: resolve issue with ABC"
   ```
6. **Push** to your fork
7. **Open a Pull Request**

## Code Style

### Rust Style

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Keep functions focused and reasonably sized

### Commit Messages

Use conventional commits:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only
- `refactor:` - Code change that neither fixes nor adds
- `test:` - Adding tests
- `chore:` - Maintenance tasks

### Documentation

- Document public APIs with doc comments
- Update relevant docs in `docs/` for user-facing changes
- Keep README.md updated for significant features

## Project Structure

```
phantomlink/
├── src/
│   ├── main.rs           # Entry point
│   ├── phantomlink.rs    # Core types and exports
│   ├── audio.rs          # Audio engine
│   ├── scarlett.rs       # Scarlett Solo hardware control
│   ├── ghostwave_integration.rs  # GhostWave AI integration
│   ├── gui/
│   │   ├── mod.rs        # Main GUI application
│   │   ├── theme.rs      # Theme system
│   │   ├── widgets.rs    # Custom widgets
│   │   ├── mixer.rs      # Mixer panel
│   │   └── ...
│   ├── vst_host.rs       # VST plugin hosting
│   └── ...
├── docs/                 # Documentation
├── packaging/            # Distro packaging files
│   ├── arch/
│   ├── fedora/
│   ├── debian/
│   └── appimage/
└── assets/               # Icons, screenshots
```

## Areas for Contribution

### Good First Issues

Look for issues labeled `good-first-issue`:
- Documentation improvements
- Small bug fixes
- Additional theme presets

### Intermediate

- Additional hardware support (other Scarlett models)
- GUI improvements
- Test coverage

### Advanced

- Audio engine optimizations
- GhostWave/CUDA improvements
- New audio backends (JACK improvements, etc.)

## Testing

### Unit Tests
```bash
cargo test
```

### With Hardware
If you have a Scarlett Solo:
```bash
cargo test --features hardware-tests
```

### Manual Testing Checklist
- [ ] App launches without errors
- [ ] Scarlett detection works (if connected)
- [ ] GhostWave initializes (check RTX status)
- [ ] Mixer channels respond
- [ ] Theme switching works
- [ ] No crashes on exit

## Questions?

- Open an issue for questions
- Check existing docs in `docs/`

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
