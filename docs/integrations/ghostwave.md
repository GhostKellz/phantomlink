# GhostWave AI Noise Suppression

GhostWave is the RTX-accelerated AI noise suppression engine powering PhantomLink's denoising capabilities. It provides NVIDIA Broadcast-quality noise removal on Linux.

## Overview

GhostWave uses deep neural networks running on NVIDIA Tensor Cores to remove:
- Background noise (fans, HVAC, traffic)
- Keyboard and mouse clicks
- Room reverb and echo
- Other non-voice audio

## GPU Acceleration

### Supported GPUs

| GPU Generation | Architecture | Precision | Performance |
|----------------|--------------|-----------|-------------|
| RTX 50 Series | Blackwell | FP4 | 2-3x faster than FP16 |
| RTX 40 Series | Ada Lovelace | FP16 | Baseline |
| RTX 30 Series | Ampere | FP16 | ~90% of Ada |
| RTX 20 Series | Turing | FP16 | ~70% of Ada |
| GTX / Non-NVIDIA | - | FP32 CPU | Fallback |

### RTX 5090 Blackwell (FP4)

The RTX 5090 with Blackwell architecture supports FP4 (4-bit floating point) Tensor Core operations, providing:
- 2-3x faster inference than FP16
- Lower power consumption
- Same quality output

PhantomLink auto-detects Blackwell GPUs and enables FP4 mode.

### Driver Requirements

| Driver | Minimum | Recommended |
|--------|---------|-------------|
| nvidia-open | 545 | 580+ |
| nvidia-proprietary | 545 | 555+ |

Check your driver:
```bash
nvidia-smi
# Look for Driver Version: 5XX.XX
```

## Processing Profiles

### XLR Studio
- **Use case:** Recording, podcasting with XLR mics
- **Suppression:** Low-medium (preserves voice character)
- **Latency:** Low (~10ms)
- **Echo cancellation:** Off
- **Best for:** Rode PodMic, SM7B, condenser mics

### Streaming
- **Use case:** Live streaming, gaming
- **Suppression:** High (aggressive noise removal)
- **Latency:** Medium (~8ms)
- **Echo cancellation:** On
- **Best for:** Noisy environments, open-back headphones

### Balanced
- **Use case:** General voice chat, meetings
- **Suppression:** Medium
- **Latency:** Low (~5ms)
- **Echo cancellation:** Off
- **Best for:** Discord, Zoom, Teams

### Music
- **Use case:** Instrument recording, music production
- **Suppression:** Minimal (preserves dynamics)
- **Latency:** Low (~3ms)
- **Echo cancellation:** Off
- **Best for:** Guitars, keyboards, vocals with effects

## Quality Levels

| Level | Latency | GPU Usage | Quality | Recommended For |
|-------|---------|-----------|---------|-----------------|
| Fast | ~2ms | Low | Good | Gaming, real-time |
| Balanced | ~5ms | Medium | Great | Streaming |
| Quality | ~10ms | High | Excellent | Recording |
| Ultra | ~15ms | Very High | Maximum | Production, RTX 40/50 |

## Echo Cancellation (AEC)

Acoustic Echo Cancellation removes speaker audio from your microphone input, preventing:
- Feedback loops
- Hearing yourself in recordings
- Echo in voice chat

### How It Works
1. GhostWave captures your speaker output as a reference
2. AI model learns the acoustic characteristics
3. Speaker audio is subtracted from mic input in real-time

### Configuration
- **Tail Length:** 50-500ms (how far back to look for echo)
- **Suppression Level:** 0.0-1.0 (how aggressively to remove)

### When to Use
- Open-back headphones
- Speakers instead of headphones
- High-volume monitoring
- Conference calls

### When to Disable
- Closed-back headphones with good isolation
- Recording music (may affect quality)
- Low monitoring volume

## Configuration in PhantomLink

### Enable/Disable
Toggle the GhostWave panel on/off to enable or disable processing.

### Select Profile
Choose from the dropdown:
- XLR Studio
- Streaming
- Balanced
- Music

### Adjust Strength
Slider from 0% to 100%:
- 0-30%: Minimal, preserves everything
- 30-60%: Light noise reduction
- 60-80%: Moderate, good for most use
- 80-100%: Aggressive, may affect voice

### Echo Cancellation Toggle
Enable when using speakers or open-back headphones.

## Monitoring RTX Status

The GhostWave panel shows:
- **GPU Name:** e.g., "NVIDIA GeForce RTX 5090"
- **Precision:** FP4 (Blackwell), FP16 (Ada/Ampere), or CPU
- **Processing Mode:** RTX Tensor Core or CPU fallback
- **Memory Usage:** VRAM used/total
- **Latency:** Current processing latency

## Performance Tips

### Minimize Latency
1. Use "Fast" quality for real-time applications
2. Ensure buffer size matches (256 samples recommended)
3. Use RTX GPU for hardware acceleration

### Maximize Quality
1. Use "Ultra" quality (RTX 40/50 recommended)
2. Enable higher suppression strength
3. Use "Quality" or "Ultra" denoise level

### Reduce GPU Usage
1. Use "Fast" quality level
2. Lower suppression strength
3. Disable echo cancellation if not needed

## Troubleshooting

### RTX Not Detected

```bash
# Check NVIDIA driver
nvidia-smi

# Check CUDA
nvcc --version

# Verify nvidia-open module
lsmod | grep nvidia
```

If using nouveau driver, switch to nvidia-open:
```bash
# Arch Linux
sudo pacman -S nvidia-open

# Fedora
sudo dnf install akmod-nvidia

# Ubuntu
sudo apt install nvidia-driver-545
```

### High Latency

1. Switch to "Fast" quality
2. Reduce buffer size
3. Check GPU isn't thermal throttling:
   ```bash
   nvidia-smi -q -d TEMPERATURE
   ```

### Voice Sounds Robotic

1. Lower suppression strength
2. Switch to "XLR Studio" or "Music" profile
3. Use "Quality" instead of "Fast" level

### Echo Cancellation Not Working

1. Ensure speaker output is routed to GhostWave reference
2. Increase tail length (try 200-300ms)
3. Increase suppression level
4. Check speaker and mic aren't on same physical device

## Technical Details

### Audio Pipeline
```
Mic Input → PhantomLink → GhostWave AI → Channel Processing → Output
                ↑
        Speaker Reference (for AEC)
```

### Neural Network
- RNNoise-based architecture with custom extensions
- Trained on diverse noise datasets
- Real-time inference via TensorRT (NVIDIA) or ONNX (CPU)

### Memory Usage
- ~200-500MB VRAM depending on quality level
- CPU fallback uses ~500MB system RAM
