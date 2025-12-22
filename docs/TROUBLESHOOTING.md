# PhantomLink Troubleshooting Guide

Complete troubleshooting guide for PhantomLink - the professional audio mixer with RTX AI noise suppression.

## Table of Contents

- [Quick Diagnostics](#quick-diagnostics)
- [Audio Issues](#audio-issues)
- [GhostWave/RTX Issues](#ghostwavertx-issues)
- [GUI Issues](#gui-issues)
- [Performance Issues](#performance-issues)
- [Device Issues](#device-issues)
- [VST Plugin Issues](#vst-plugin-issues)
- [IPC/Integration Issues](#ipcintegration-issues)

---

## Quick Diagnostics

### Check System Status

```bash
# Check if PipeWire is running
systemctl --user status pipewire

# Check audio devices
pw-cli list-objects Node | grep -E "name|media.class"

# Check PhantomLink process
pgrep -a phantomlink

# Check IPC socket
ls -la /run/user/$(id -u)/phantomlink.sock
```

### Log Output

```bash
# Run PhantomLink with debug logging
RUST_LOG=debug phantomlink

# Run with specific module logging
RUST_LOG=phantomlink::audio=trace,phantomlink::ghostwave=debug phantomlink
```

---

## Audio Issues

### No Audio Output

**Symptoms**: No sound from any channel, meters not moving

**Solutions**:

1. **Check audio devices**:
   ```bash
   # List available audio devices
   pw-cli list-objects Node | grep -E "alsa|Audio"

   # Check device routing
   pw-link -l
   ```

2. **Verify PipeWire connection**:
   ```bash
   # Restart PipeWire
   systemctl --user restart pipewire pipewire-pulse wireplumber
   ```

3. **Check channel routing in PhantomLink**:
   - Open Settings > Audio
   - Verify input/output device selection
   - Check that channels are not muted

### Audio Crackling/Dropouts

**Symptoms**: Audio cuts out, pops, crackles, or stutters

**Solutions**:

1. **Increase buffer size**:
   - Open Settings > Audio
   - Increase buffer size from 128 to 256 or 512 frames
   - Trade-off: higher latency but more stable audio

2. **Check CPU usage**:
   ```bash
   # Monitor CPU during playback
   watch -n 1 "grep 'cpu ' /proc/stat"

   # Check PhantomLink CPU usage
   top -p $(pgrep phantomlink)
   ```

3. **Enable real-time priority**:
   ```bash
   # Add user to audio group
   sudo usermod -aG audio $USER

   # Configure limits.conf
   echo '@audio - rtprio 95' | sudo tee -a /etc/security/limits.conf
   echo '@audio - memlock unlimited' | sudo tee -a /etc/security/limits.conf
   ```

4. **Check PipeWire quantum**:
   ```bash
   # Lower quantum for better performance
   pw-metadata -n settings 0 clock.force-quantum 256
   ```

### High Latency

**Symptoms**: Noticeable delay between input and output

**Solutions**:

1. **Reduce buffer size**:
   - Settings > Audio > Buffer Size: Try 64 or 128 frames
   - Requires stable system with real-time priorities

2. **Use ALSA direct mode** (if available):
   - Settings > Audio > Backend: ALSA
   - Bypasses PipeWire for lowest latency

3. **Check processing chain**:
   - Disable unnecessary VST plugins
   - Use "Balanced" profile instead of "Ultra Quality"

### Audio Quality Issues

**Symptoms**: Audio sounds distorted, muffled, or artifacts

**Solutions**:

1. **Check sample rate matching**:
   ```bash
   # Check device sample rate
   pw-cli info $(pw-cli ls Node | grep your-device | awk '{print $2}')
   ```
   Ensure PhantomLink uses the same sample rate as your device.

2. **Disable overprocessing**:
   - Reduce noise suppression strength
   - Turn off de-esser if not needed
   - Check compressor settings

3. **Check clipping**:
   - Watch for red indicators on meters
   - Reduce input gain if clipping occurs

---

## GhostWave/RTX Issues

### GhostWave Not Initializing

**Symptoms**: "GhostWave not available" or "Failed to initialize"

**Solutions**:

1. **Check GhostWave installation**:
   ```bash
   # Verify GhostWave is built with PhantomLink
   phantomlink --help | grep -i ghostwave

   # Check library path
   ldd /path/to/phantomlink | grep ghostwave
   ```

2. **Verify CUDA/RTX drivers**:
   ```bash
   # Check NVIDIA driver
   nvidia-smi

   # Check CUDA version
   nvcc --version
   ```

3. **Run GhostWave diagnostics**:
   ```bash
   ghostwave --doctor
   ```

### RTX Acceleration Not Working

**Symptoms**: Processing uses CPU instead of GPU, high CPU usage

**Solutions**:

1. **Verify RTX GPU**:
   ```bash
   nvidia-smi --query-gpu=name,compute_cap --format=csv
   ```
   Requires RTX 2060 or newer (compute capability 7.5+)

2. **Check CUDA libraries**:
   ```bash
   # Verify CUDA runtime
   ldconfig -p | grep cuda

   # Check CUDA installation
   ls /usr/local/cuda/lib64/
   ```

3. **Update NVIDIA drivers**:
   ```bash
   # Arch Linux
   sudo pacman -Syu nvidia nvidia-utils

   # Ubuntu
   sudo apt update && sudo apt install nvidia-driver-535
   ```

4. **Check GPU memory**:
   ```bash
   nvidia-smi --query-gpu=memory.used,memory.free --format=csv
   ```
   Ensure sufficient VRAM is available.

### Poor Noise Suppression Quality

**Symptoms**: Noise still present, voice sounds robotic or muffled

**Solutions**:

1. **Adjust strength settings**:
   - Try different profiles (Streaming, Balanced, Studio)
   - Reduce strength if voice sounds distorted
   - Increase strength if noise persists

2. **Check input levels**:
   - Input should peak at -12dB to -6dB
   - Avoid clipping (red indicators)

3. **Environment factors**:
   - Reduce background noise at source if possible
   - Use XLR microphone for best results
   - Check for electrical interference

---

## GUI Issues

### GUI Not Rendering

**Symptoms**: Black screen, window doesn't appear, crash on startup

**Solutions**:

1. **Check graphics drivers**:
   ```bash
   # Check OpenGL support
   glxinfo | grep "OpenGL version"

   # For Wayland
   echo $XDG_SESSION_TYPE
   ```

2. **Try software rendering**:
   ```bash
   LIBGL_ALWAYS_SOFTWARE=1 phantomlink
   ```

3. **Check display scaling**:
   ```bash
   # Run with explicit DPI
   GDK_SCALE=1 phantomlink
   ```

### GUI Lag/Slow Response

**Symptoms**: UI feels sluggish, visualizers stutter

**Solutions**:

1. **Reduce visualizer quality**:
   - Settings > Display > Reduce visualizer FPS
   - Disable spectrum analyzer if not needed

2. **Check GPU utilization**:
   ```bash
   nvidia-smi dmon -s u
   ```

3. **Disable VSync**:
   ```bash
   __GL_SYNC_TO_VBLANK=0 phantomlink
   ```

---

## Performance Issues

### High CPU Usage

**Symptoms**: CPU usage above 50%, system becomes sluggish

**Solutions**:

1. **Enable RTX acceleration**:
   - Verify GPU is being used (check logs)
   - RTX reduces CPU load by 70-90%

2. **Reduce processing quality**:
   - Switch to "Balanced" or "Streaming" profile
   - Disable unused features (de-esser, EQ)

3. **Increase buffer size**:
   - Larger buffers = less frequent processing

4. **Check for runaway threads**:
   ```bash
   ps -eLf | grep phantomlink
   ```

### Memory Usage Growing

**Symptoms**: RAM usage increases over time

**Solutions**:

1. **Check for memory leaks**:
   ```bash
   # Monitor memory over time
   watch -n 5 "ps -p $(pgrep phantomlink) -o rss,vsz"
   ```

2. **Restart PhantomLink periodically** (temporary workaround)

3. **Update to latest version** - memory issues may be fixed in newer releases

---

## Device Issues

### Scarlett Solo Not Detected

**Symptoms**: Focusrite Scarlett Solo doesn't appear in device list

**Solutions**:

1. **Check USB connection**:
   ```bash
   lsusb | grep -i focusrite
   ```

2. **Verify ALSA detection**:
   ```bash
   aplay -l | grep Scarlett
   arecord -l | grep Scarlett
   ```

3. **Check permissions**:
   ```bash
   # Add user to audio group
   sudo usermod -aG audio $USER

   # Logout/login required
   ```

4. **Install firmware** (if needed):
   ```bash
   # Install sof-firmware for some devices
   sudo pacman -S sof-firmware  # Arch
   sudo apt install firmware-sof-signed  # Debian/Ubuntu
   ```

### XLR Input Issues

**Symptoms**: XLR mic not working, no 48V phantom power

**Solutions**:

1. **Enable phantom power on interface**:
   - Check hardware switch on Scarlett Solo
   - Some interfaces require software control

2. **Check input selection**:
   - Ensure XLR/Line switch is in correct position
   - Select correct input in PhantomLink

3. **Verify gain settings**:
   - XLR mics often need significant gain
   - Check for clipping at high gains

---

## VST Plugin Issues

### VST Plugins Not Loading

**Symptoms**: Plugins don't appear in list, "Failed to load plugin"

**Solutions**:

1. **Check plugin paths**:
   ```bash
   # Standard VST2 locations
   ls /usr/lib/vst/
   ls /usr/lib/lxvst/
   ls ~/.vst/
   ```

2. **Verify plugin architecture**:
   - PhantomLink requires Linux VST2 (.so) plugins
   - 64-bit plugins only

3. **Check plugin dependencies**:
   ```bash
   ldd /path/to/plugin.so | grep "not found"
   ```

4. **Rescan plugins**:
   - Settings > VST > Rescan Plugins

### VST Plugin Crashes

**Symptoms**: PhantomLink crashes when using certain plugins

**Solutions**:

1. **Identify problematic plugin**:
   - Disable plugins one by one
   - Check logs for crash information

2. **Update plugin**:
   - Check vendor for updates
   - Some plugins have known compatibility issues

3. **Isolate plugin in chain**:
   - Move plugin to end of chain
   - Check for parameter conflicts

---

## IPC/Integration Issues

### IPC Socket Not Created

**Symptoms**: Cannot connect to PhantomLink from other applications

**Solutions**:

1. **Check socket location**:
   ```bash
   ls -la /run/user/$(id -u)/phantomlink.sock
   ls -la /tmp/phantomlink.sock
   ```

2. **Verify IPC is enabled**:
   - IPC should start automatically
   - Check logs for IPC initialization

3. **Check permissions**:
   ```bash
   # Socket should be user-readable
   chmod 600 /run/user/$(id -u)/phantomlink.sock
   ```

### JSON-RPC Commands Not Working

**Symptoms**: IPC commands return errors or no response

**Solutions**:

1. **Test basic connectivity**:
   ```bash
   echo '{"jsonrpc":"2.0","method":"get_status","id":1}' | \
     nc -U /run/user/$(id -u)/phantomlink.sock
   ```

2. **Check JSON format**:
   - Ensure valid JSON syntax
   - Use correct method names

3. **Verify API version**:
   - Check documentation for current API methods

---

## Getting Help

### Collect Diagnostic Information

Before reporting issues, collect:

```bash
# System info
uname -a
cat /etc/os-release

# Audio subsystem
pw-cli info all > pipewire-info.txt
aplay -l > alsa-devices.txt

# GPU info
nvidia-smi > gpu-info.txt

# PhantomLink logs
RUST_LOG=debug phantomlink 2>&1 | tee phantomlink.log
```

### Report Issues

- GitHub Issues: https://github.com/ghostkellz/phantomlink/issues
- Include diagnostic information
- Describe steps to reproduce

---

**Last Updated**: December 2025 (v0.4.0)
