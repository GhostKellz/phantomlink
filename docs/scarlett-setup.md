# Focusrite Scarlett Solo 4th Gen Setup

Complete guide for configuring your Scarlett Solo 4th Gen with PhantomLink.

## Hardware Overview

The Scarlett Solo 4th Gen is a 2-in/2-out USB audio interface with:
- 1x XLR input with 48V phantom power and Air mode
- 1x 1/4" instrument/line input
- 2x 1/4" monitor outputs
- Headphone output with independent level control
- USB-C connectivity

## Prerequisites

### Linux Kernel
Ensure you're running Linux kernel 5.14+ for best Scarlett 4th Gen support:
```bash
uname -r
```

### ALSA Scarlett Driver
The Scarlett 4th Gen uses the `snd-usb-audio` driver with Scarlett Gen 4 mixer support. Verify it's loaded:
```bash
lsmod | grep snd_usb_audio
```

### Permissions
Add yourself to the audio group:
```bash
sudo usermod -aG audio $USER
# Log out and back in
```

## Connecting Your Scarlett

1. **Connect via USB-C** to your computer
2. **Verify detection**:
   ```bash
   aplay -l | grep -i scarlett
   # Should show: card X: Solo4thGen [Scarlett Solo 4th Gen]
   ```
3. **Check ALSA controls**:
   ```bash
   amixer -c X contents | head -50
   # Replace X with your card number
   ```

## PhantomLink Auto-Detection

When you launch PhantomLink, it automatically:
1. Scans for Scarlett Solo 4th Gen
2. Opens ALSA control interface
3. Reads current hardware state
4. Displays in the Scarlett panel

If not detected, check:
- USB connection
- Card appears in `aplay -l`
- You're in the `audio` group

## Hardware Controls

### 48V Phantom Power

For condenser microphones (Rode PodMic, AT2020, etc.):

**In PhantomLink:**
- Toggle the "48V" button in the Scarlett panel
- Red indicator when active

**Via command line:**
```bash
amixer -c X set 'Line In 2 Phantom Power' on
amixer -c X set 'Line In 2 Phantom Power' off
```

**Warning:** Only enable for condenser mics. Dynamic mics (SM58, SM7B) don't need it.

### Air Mode

Focusrite's ISA transformer emulation for presence and clarity:

| Mode | Description | Best For |
|------|-------------|----------|
| Off | No processing | Flat response |
| Presence | High-frequency boost | Vocals, speech |
| Presence + Drive | Presence + harmonic saturation | Vocals needing warmth |

**In PhantomLink:**
- Select from the Air dropdown in Scarlett panel

**Via command line:**
```bash
amixer -c X set 'Line In 2 Air' 'Off'
amixer -c X set 'Line In 2 Air' 'Presence'
amixer -c X set 'Line In 2 Air' 'Presence + Drive'
```

### Input Level (1/4" Jack)

For the instrument/line input:

| Mode | Use Case |
|------|----------|
| Line | Keyboards, synths, audio players (+4dBu) |
| Instrument | Guitars, bass (Hi-Z) |

**In PhantomLink:**
- Toggle Line/Inst button

**Via command line:**
```bash
amixer -c X set 'Line In 1 Level' 'Line'
amixer -c X set 'Line In 1 Level' 'Inst'
```

### Direct Monitoring

Zero-latency hardware monitoring (bypasses computer):

**In PhantomLink:**
- Toggle "Direct Monitor" button

**Via command line:**
```bash
amixer -c X set 'Direct Monitor' on
amixer -c X set 'Direct Monitor' off
```

## DSP Routing

The Scarlett Solo 4th Gen has internal DSP routing capabilities.

### Capture Sources
Available routing sources:
- Off
- Analogue 1 (XLR/Mic)
- Analogue 2 (Line/Inst)
- Mix A through Mix F
- DSP 1, DSP 2
- PCM 1, PCM 2

### DSP Input Routing
Route audio to the hardware DSP:
```bash
# Set DSP Input 1 to Analogue 1
amixer -c X set 'DSP Input 1 Capture Enum' 'Analogue 1'
```

### PCM Capture Source
Select what your DAW/application records:
```bash
# Direct analog input
amixer -c X set 'PCM 01 Capture Enum' 'Analogue 1'
amixer -c X set 'PCM 02 Capture Enum' 'Analogue 2'

# Or capture from mixer
amixer -c X set 'PCM 01 Capture Enum' 'Mix A'
```

## Hardware Level Meters

PhantomLink reads the 12-channel hardware level meters:

| Channel | Source |
|---------|--------|
| 0 | Analogue 1 (XLR) |
| 1 | Analogue 2 (Line/Inst) |
| 2-3 | DSP 1-2 |
| 4-5 | Mix A L/R |
| 6-7 | Mix B L/R |
| 8-11 | PCM 1-4 |

Read manually:
```bash
amixer -c X cget numid=46
# Returns 12 values (0-4095 range)
```

## Sample Rate Configuration

The Scarlett Solo 4th Gen supports:
- 44.1 kHz
- 48 kHz (recommended)
- 88.2 kHz
- 96 kHz
- 176.4 kHz
- 192 kHz

Set via:
```bash
# Check current
amixer -c X get 'Sample Rate'

# Set to 48kHz (if supported by control)
amixer -c X set 'Sample Rate' 48000
```

Or configure in your audio application/PipeWire config.

## Troubleshooting

### No Sound
1. Check physical connections
2. Verify gain knobs aren't at zero
3. Check `alsamixer -c X` levels
4. Ensure correct output routing

### Crackling/Distortion
1. Lower buffer size might cause xruns
2. Check CPU usage
3. Enable real-time scheduling:
   ```bash
   sudo usermod -aG realtime $USER
   ```

### Device Disconnects
1. Try different USB port (USB 3.0 recommended)
2. Check `dmesg` for USB errors
3. Update firmware via Focusrite Control (Windows/Mac)

### High Latency
1. Use smaller buffer sizes (128-256 samples)
2. Enable real-time kernel or `PREEMPT_RT`
3. Use PipeWire with low quantum:
   ```bash
   pw-metadata -n settings 0 clock.force-quantum 256
   ```

## Recommended Settings

### For Podcasting/Streaming
- Sample Rate: 48 kHz
- Buffer: 256 samples
- Phantom Power: On (for condenser mics)
- Air Mode: Presence
- GhostWave: Streaming profile

### For Music Recording
- Sample Rate: 48 kHz or 96 kHz
- Buffer: 128-256 samples
- Phantom Power: As needed
- Air Mode: Off or Presence
- GhostWave: Music profile (minimal processing)

### For Gaming/Voice Chat
- Sample Rate: 48 kHz
- Buffer: 256-512 samples
- GhostWave: Fast or Balanced profile
