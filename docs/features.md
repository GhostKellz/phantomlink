# PhantomLink Features

Complete feature documentation for PhantomLink.

## Audio Interface Control

### Focusrite Scarlett Solo 4th Gen

Full hardware control for the Scarlett Solo 4th Gen USB audio interface:

#### 48V Phantom Power
- Powers condenser microphones
- Toggle via GUI or saved in preferences
- Status indicator in interface panel

#### Air Mode
Three modes for the XLR input:
- **Off**: No processing
- **Presence**: High-frequency clarity boost (ISA transformer emulation)
- **Presence + Drive**: Presence boost with harmonic saturation

#### Input Level
For the 1/4" jack input:
- **Line**: +4dBu for keyboards, synths, audio players
- **Instrument**: Hi-Z for guitars and bass

#### Direct Monitoring
- Zero-latency hardware monitoring
- Bypasses software for real-time monitoring
- Toggle via GUI

#### DSP Routing
Advanced routing capabilities:
- **DSP Input 1/2**: Select source for hardware DSP
- **Mixer Input**: Route to internal mixer
- **PCM Capture**: Select capture source
- **Monitor Mix A-F**: 6 independent monitor mixes
- Mix volume control per input

#### Hardware Level Metering
Real-time level meters from hardware:
- 12-channel metering
- Analogue 1/2 (inputs)
- DSP 1/2
- Mix A L/R, Mix B L/R
- PCM 1-4 (DAW playback)

---

## GhostWave AI Noise Suppression

RTX-accelerated AI denoising powered by GhostWave.

### Processing Profiles

#### XLR Studio
- Optimized for XLR microphones
- Lower suppression to preserve voice character
- Low latency (~10ms)
- Best for: Recording, podcasting

#### Streaming
- Aggressive noise suppression
- Echo cancellation enabled
- Higher latency acceptable
- Best for: Live streaming, gaming

#### Balanced
- Good balance of quality and suppression
- Moderate latency
- Best for: General voice chat, meetings

#### Music
- Minimal processing
- Preserves dynamics and harmonics
- Best for: Instrument recording

### Quality Levels

| Level | Latency | CPU | GPU | Use Case |
|-------|---------|-----|-----|----------|
| Fast | ~2ms | Low | Low | Gaming, real-time |
| Balanced | ~5ms | Medium | Medium | Streaming |
| Quality | ~10ms | High | High | Recording |
| Ultra | ~15ms | Very High | High | Production |

### Echo Cancellation (AEC)
- Removes speaker audio from microphone input
- Uses speaker output as reference
- Configurable tail length (50-500ms)
- Suppression level adjustment

### RTX Acceleration

| GPU Generation | Precision | Performance |
|----------------|-----------|-------------|
| RTX 50 (Blackwell) | FP4 | 2-3x faster than FP16 |
| RTX 40 (Ada) | FP16 | Baseline |
| RTX 30 (Ampere) | FP16 | ~90% of Ada |
| RTX 20 (Turing) | FP16 | ~70% of Ada |
| Non-RTX | FP32/CPU | Fallback |

---

## Mixer

### Channel Strips

Professional channel strips with:

#### VU Meters
- Segmented LED-style display
- Peak and RMS levels
- Peak hold indicator (1.5s hold, decay)
- Clip indicator (2s hold)
- Color-coded: Green (-60 to -18dB), Yellow (-18 to -6dB), Orange (-6 to 0dB), Red (0dB+)

#### Hardware-Style Gain Knob
- Drag vertically to adjust
- Arc indicator showing position
- -20dB to +20dB range
- dB readout display

#### Pan Control
- Centered dot indicator
- L/R markers
- Full stereo spread

#### Volume Fader
- Vertical fader style
- 0-100% range
- Smooth response

#### Channel Controls
- **Mute**: Silences channel (red indicator when active)
- **Solo**: Isolates channel (yellow indicator when active)
- Channel border changes color to indicate state

### VST Plugin Support
- Per-channel VST 2.4 plugin loading
- Plugin scanner for system plugins
- Vendor and name display
- Bypass capable

### Application Audio Routing
- Per-application volume control
- Route specific apps to specific outputs
- PipeWire integration for app detection

---

## Themes

### Tokyo Night
Tokyo Night theme family with three variants:
- **Night**: Default dark theme with blue/purple accents. Calm, focused aesthetic.
- **Moon**: Softer dark variant with muted tones.
- **Storm**: Higher contrast dark variant.

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Space` | Toggle mute on selected channel |
| `S` | Toggle solo on selected channel |
| `M` | Open mixer panel |
| `G` | Open GhostWave settings |
| `I` | Open interface panel |
| `Ctrl+,` | Open preferences |

---

## Configuration Files

PhantomLink stores configuration in:
- `~/.config/phantomlink/config.toml` - Main configuration
- `~/.config/phantomlink/themes/` - Custom themes
- `~/.local/share/phantomlink/vst/` - VST plugin cache

---

## Audio Pipeline

```
[Scarlett Solo] → [ALSA] → [PhantomLink Input]
                                    ↓
                           [GhostWave AI Denoise]
                                    ↓
                           [Channel Processing]
                           (Gain → VST → Volume → Pan)
                                    ↓
                           [Mixer Output]
                                    ↓
                    [PipeWire/ALSA] → [Output Device]
```
