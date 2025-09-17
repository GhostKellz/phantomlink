# PhantomLink ↔ GhostWave Integration Guide

<div align="center">
  <img src="assets/ghostwave-logo.png" alt="GhostWave Logo" width="128" height="128">

  **Professional XLR Interface Integration for PhantomLink**
  _NVIDIA Open Kernel Modules · RTX-Accelerated Processing · Sub-15ms Latency_

  [![PhantomLink](https://img.shields.io/badge/PhantomLink-Compatible-blue.svg)](https://github.com/ghostkellz/phantomlink)
  [![GhostWave](https://img.shields.io/badge/GhostWave-Core-orange.svg)](https://github.com/ghostkellz/ghostwave)
  [![NVIDIA RTX](https://img.shields.io/badge/NVIDIA-Open%20Drivers-76B900.svg)](https://github.com/NVIDIA/open-gpu-kernel-modules)
</div>

---

## 📋 Overview

This guide covers integrating **GhostWave's RTX-accelerated noise suppression** into **PhantomLink's professional mixer architecture**. This integration replaces PhantomLink's experimental Zig-based NVIDIA driver support with GhostWave's mature NVIDIA open kernel module implementation.

### Integration Benefits

- 🎛️ **Professional Mixer Integration** - Native GhostWave processing in PhantomLink's 4-channel mixer
- ⚡ **Enhanced Performance** - <15ms latency vs PhantomLink's <20ms target
- 🔧 **Simplified Architecture** - Replace Zig NVIDIA integration with Rust-native solution
- 🎯 **Focusrite Optimization** - Enhanced Scarlett Solo Gen 4 support
- 🧠 **Advanced AI Models** - RTX Voice-grade noise suppression vs basic denoising

### Architecture Overview

```
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│   PhantomLink       │    │   GhostWave Core    │    │   NVIDIA RTX        │
│   Professional      │    │   Library           │    │   Open Drivers      │
│   Mixer             │    │                     │    │                     │
│  ┌───────────────┐  │    │  ┌───────────────┐  │    │  ┌───────────────┐  │
│  │ 4-Ch Mixer    │  │◄──►│  │ Noise Engine  │  │◄──►│  │ CUDA Runtime  │  │
│  │ VST Plugins   │  │    │  │ Real-time DSP │  │    │  │ GPU Accel     │  │
│  │ Scarlett I/O  │  │    │  │ Lock-free     │  │    │  │ Memory Mgmt   │  │
│  └───────────────┘  │    │  └───────────────┘  │    │  └───────────────┘  │
│                     │    │                     │    │                     │
│  egui Interface     │    │  Rust Native API    │    │  Open Kernel Mods   │
│  Spectrum Analysis  │    │  JSON-RPC IPC       │    │  RTX 20+ Series     │
└─────────────────────┘    └─────────────────────┘    └─────────────────────┘
```

---

## 🔧 Crate Integration

### Cargo.toml Configuration

Add GhostWave as a dependency to PhantomLink's `Cargo.toml`:

```toml
[dependencies]
ghostwave-core = {
    git = "https://github.com/ghostkellz/ghostwave",
    features = [
        "pipewire-backend",    # PhantomLink's preferred audio backend
        "alsa-backend",        # Fallback for direct hardware access
        "nvidia-rtx",          # RTX acceleration
        "json-rpc"             # IPC integration
    ]
}

# Optional: Version pinning for production
ghostwave-core = {
    git = "https://github.com/ghostkellz/ghostwave",
    tag = "v0.1.0",
    features = ["pipewire-backend", "alsa-backend", "nvidia-rtx", "json-rpc"]
}
```

### Feature Flag Coordination

Align PhantomLink's feature flags with GhostWave integration:

```toml
[features]
default = ["pipewire", "ghostwave-integration"]

# Audio backends
pipewire = ["ghostwave-core/pipewire-backend"]
alsa = ["ghostwave-core/alsa-backend"]
jack = ["ghostwave-core/jack-backend"]

# Hardware acceleration
nvidia-rtx = ["ghostwave-core/nvidia-rtx"]
cpu-fallback = ["ghostwave-core/cpu-processing"]

# Integration features
ghostwave-integration = ["ghostwave-core/json-rpc"]
advanced-denoising = ["ghostwave-core", "nvidia-rtx"]
```

---

## 🎛️ Mixer Integration

### PhantomLink Audio Engine Integration

Replace PhantomLink's basic denoising with GhostWave's RTX-accelerated processing:

```rust
use ghostwave_core::{
    NoiseProcessor, Config, AudioBuffer,
    backend::PipeWireBackend,
    ipc::JsonRpcServer
};

pub struct PhantomLinkAudioEngine {
    // Existing PhantomLink components
    mixer: FourChannelMixer,
    scarlett_interface: ScarlettSoloGen4,
    spectrum_analyzer: SpectrumAnalyzer,
    vst_host: VstHost,

    // GhostWave integration
    ghostwave_processor: NoiseProcessor,
    ghostwave_config: Config,
    audio_backend: PipeWireBackend,
}

impl PhantomLinkAudioEngine {
    pub fn new() -> anyhow::Result<Self> {
        // Initialize GhostWave with PhantomLink-optimized settings
        let ghostwave_config = Config::load("phantomlink_profile")?;
        let ghostwave_processor = NoiseProcessor::new(&ghostwave_config.noise_suppression)?;

        // Use GhostWave's enhanced Scarlett detection
        let audio_backend = PipeWireBackend::new_with_device_detection()?;

        // Initialize existing PhantomLink components
        let mixer = FourChannelMixer::new();
        let scarlett_interface = ScarlettSoloGen4::new()?;
        let spectrum_analyzer = SpectrumAnalyzer::new();
        let vst_host = VstHost::new();

        Ok(Self {
            mixer,
            scarlett_interface,
            spectrum_analyzer,
            vst_host,
            ghostwave_processor,
            ghostwave_config,
            audio_backend,
        })
    }

    pub fn process_audio_frame(&mut self, input: &[f32], output: &mut [f32]) -> anyhow::Result<()> {
        // Create audio buffers
        let mut ghostwave_input = AudioBuffer::from_interleaved(input);
        let mut ghostwave_output = AudioBuffer::new(input.len());

        // Apply GhostWave noise suppression
        self.ghostwave_processor.process(&ghostwave_input, &mut ghostwave_output)?;

        // Convert back to PhantomLink's mixer format
        let processed_audio = ghostwave_output.to_interleaved();

        // Apply PhantomLink's mixer processing
        self.mixer.process_4_channel(&processed_audio, output)?;

        // Update spectrum analysis with processed audio
        self.spectrum_analyzer.update(output);

        Ok(())
    }
}
```

### Replacing Zig NVIDIA Integration

Remove PhantomLink's experimental Zig-based NVIDIA driver code and replace with GhostWave:

```rust
// OLD: PhantomLink's Zig-based approach (REMOVE)
// mod zig_nvidia_integration;
// use zig_nvidia_integration::NvidiaDriver;

// NEW: GhostWave's mature Rust implementation
use ghostwave_core::nvidia::{RtxAccelerator, CudaContext};

pub struct EnhancedNvidiaIntegration {
    rtx_accelerator: RtxAccelerator,
    cuda_context: CudaContext,
}

impl EnhancedNvidiaIntegration {
    pub fn new() -> anyhow::Result<Self> {
        // GhostWave handles NVIDIA open driver detection automatically
        let cuda_context = CudaContext::new_with_open_drivers()?;
        let rtx_accelerator = RtxAccelerator::new(&cuda_context)?;

        Ok(Self {
            rtx_accelerator,
            cuda_context,
        })
    }

    pub fn get_gpu_utilization(&self) -> f32 {
        self.cuda_context.get_utilization()
    }

    pub fn get_memory_usage(&self) -> (usize, usize) {
        self.cuda_context.get_memory_info()
    }
}
```

---

## 📊 Performance Optimization

### Latency Optimization for PhantomLink

Configure GhostWave to meet PhantomLink's professional audio requirements:

```rust
use ghostwave_core::{Config, LatencyProfile};

impl PhantomLinkAudioEngine {
    fn configure_for_professional_use(&mut self) -> anyhow::Result<()> {
        // Create PhantomLink-specific configuration
        let mut config = Config::default();

        // Optimize for PhantomLink's <20ms target (aim for <15ms)
        config.latency_profile = LatencyProfile::Professional {
            target_latency_ms: 12.0,
            max_acceptable_ms: 15.0,
            prefer_stability: true,
        };

        // Configure for Scarlett Solo Gen 4 optimally
        config.audio.sample_rate = 96000; // Higher quality for professional use
        config.audio.frames_per_buffer = 256; // Balanced latency/stability
        config.audio.channels = 2; // Stereo for Scarlett Solo

        // RTX acceleration settings
        config.noise_suppression.rtx_enabled = true;
        config.noise_suppression.model = "professional"; // Higher quality model
        config.noise_suppression.strength = 0.8; // Aggressive for XLR

        // Real-time optimization
        config.performance.use_realtime_priority = true;
        config.performance.cpu_affinity = Some(vec![2, 3]); // Dedicated cores
        config.performance.memory_pool_size = 64 * 1024 * 1024; // 64MB

        self.ghostwave_config = config;
        self.ghostwave_processor.reconfigure(&self.ghostwave_config)?;

        Ok(())
    }
}
```

### Multi-Channel Processing

Integrate GhostWave with PhantomLink's 4-channel mixer:

```rust
use ghostwave_core::AudioBuffer;

pub struct MultiChannelProcessor {
    channel_processors: [NoiseProcessor; 4],
    mixer: FourChannelMixer,
}

impl MultiChannelProcessor {
    pub fn process_channels(
        &mut self,
        inputs: [&[f32]; 4],
        outputs: [&mut [f32]; 4]
    ) -> anyhow::Result<()> {
        // Process each channel through GhostWave
        for (i, (input, output)) in inputs.iter().zip(outputs.iter_mut()).enumerate() {
            let input_buffer = AudioBuffer::from_slice(input);
            let mut output_buffer = AudioBuffer::new(input.len());

            self.channel_processors[i].process(&input_buffer, &mut output_buffer)?;
            output_buffer.copy_to_slice(output);
        }

        // Apply PhantomLink's mixer processing
        self.mixer.mix_4_channels(&outputs)?;

        Ok(())
    }
}
```

---

## 🎚️ XLR Interface Integration

### Enhanced Scarlett Solo Gen 4 Support

Leverage GhostWave's improved device detection for PhantomLink:

```rust
use ghostwave_core::device::{DeviceDetector, ScarlettConfig};

impl PhantomLinkAudioEngine {
    pub fn initialize_scarlett_integration(&mut self) -> anyhow::Result<()> {
        // Use GhostWave's enhanced device detection
        let detector = DeviceDetector::new()?;

        if let Some(scarlett_device) = detector.find_scarlett_solo_gen4()? {
            println!("Found Scarlett Solo Gen 4: {}", scarlett_device.name);

            // Configure optimal settings for XLR input
            let scarlett_config = ScarlettConfig {
                phantom_power: true,        // Enable for condenser mics
                input_gain: 0.75,          // Optimal gain for XLR
                direct_monitor: false,     // Disable for processed monitoring
                sample_rate: 96000,        // Professional quality
                bit_depth: 24,             // Studio quality
            };

            // Apply configuration through GhostWave
            self.audio_backend.configure_scarlett(&scarlett_config)?;

            // Update PhantomLink's interface
            self.scarlett_interface.sync_with_ghostwave(&scarlett_config)?;
        }

        Ok(())
    }
}
```

### Direct Monitor Control Integration

Integrate GhostWave's hardware control with PhantomLink's interface:

```rust
use ghostwave_core::hardware::ScarlettController;

pub struct PhantomLinkHardwareController {
    ghostwave_controller: ScarlettController,
    ui_state: ScarlettUIState,
}

impl PhantomLinkHardwareController {
    pub fn update_gain(&mut self, gain: f32) -> anyhow::Result<()> {
        // Update hardware through GhostWave
        self.ghostwave_controller.set_input_gain(gain)?;

        // Sync UI state
        self.ui_state.input_gain = gain;

        Ok(())
    }

    pub fn toggle_phantom_power(&mut self) -> anyhow::Result<()> {
        let new_state = !self.ui_state.phantom_power;

        // Control hardware through GhostWave
        self.ghostwave_controller.set_phantom_power(new_state)?;

        // Update UI
        self.ui_state.phantom_power = new_state;

        Ok(())
    }
}
```

---

## 🖥️ GUI Integration

### egui Interface Integration

Integrate GhostWave controls into PhantomLink's egui interface:

```rust
use eframe::egui;
use ghostwave_core::ui::GhostWaveControls;

impl PhantomLinkApp {
    fn render_ghostwave_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.collapsing("GhostWave Noise Suppression", |ui| {
            ui.horizontal(|ui| {
                ui.label("Suppression Strength:");
                ui.add(egui::Slider::new(&mut self.ghostwave_strength, 0.0..=1.0));
            });

            ui.horizontal(|ui| {
                ui.label("Processing Mode:");
                egui::ComboBox::from_label("")
                    .selected_text(&self.ghostwave_mode)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.ghostwave_mode, "Balanced".to_string(), "Balanced");
                        ui.selectable_value(&mut self.ghostwave_mode, "Streaming".to_string(), "Streaming");
                        ui.selectable_value(&mut self.ghostwave_mode, "Studio".to_string(), "Studio");
                    });
            });

            // Real-time performance metrics
            ui.separator();
            ui.label(format!("Latency: {:.2}ms", self.audio_engine.get_latency()));
            ui.label(format!("GPU Usage: {:.1}%", self.audio_engine.get_gpu_usage()));

            // Status indicators
            ui.horizontal(|ui| {
                let rtx_active = self.audio_engine.is_rtx_active();
                ui.colored_label(
                    if rtx_active { egui::Color32::GREEN } else { egui::Color32::RED },
                    if rtx_active { "RTX Active" } else { "RTX Inactive" }
                );

                let latency_good = self.audio_engine.get_latency() < 15.0;
                ui.colored_label(
                    if latency_good { egui::Color32::GREEN } else { egui::Color32::YELLOW },
                    if latency_good { "Low Latency" } else { "High Latency" }
                );
            });
        });
    }
}
```

### Real-time Monitoring Integration

Add GhostWave metrics to PhantomLink's monitoring:

```rust
use ghostwave_core::monitoring::{LatencyMonitor, PerformanceMetrics};

pub struct EnhancedMonitoring {
    ghostwave_monitor: LatencyMonitor,
    phantomlink_metrics: PhantomLinkMetrics,
}

impl EnhancedMonitoring {
    pub fn update_realtime_display(&mut self, ui: &mut egui::Ui) {
        let ghostwave_metrics = self.ghostwave_monitor.get_current_metrics();
        let phantomlink_metrics = &self.phantomlink_metrics;

        // Combined performance display
        ui.group(|ui| {
            ui.label("Audio Processing Performance");

            ui.horizontal(|ui| {
                ui.label(format!("GhostWave Latency: {:.2}ms", ghostwave_metrics.latency_ms));
                ui.label(format!("Total Pipeline: {:.2}ms",
                    ghostwave_metrics.latency_ms + phantomlink_metrics.mixer_latency_ms));
            });

            ui.horizontal(|ui| {
                ui.label(format!("CPU Usage: {:.1}%", ghostwave_metrics.cpu_usage));
                ui.label(format!("GPU Usage: {:.1}%", ghostwave_metrics.gpu_usage));
            });

            // XRun detection
            if ghostwave_metrics.xruns > 0 {
                ui.colored_label(egui::Color32::RED,
                    format!("Audio Dropouts: {}", ghostwave_metrics.xruns));
            }
        });
    }
}
```

---

## 🔌 JSON-RPC IPC Integration

### PhantomLink Control Server

Enable external control of both PhantomLink and GhostWave:

```rust
use ghostwave_core::ipc::{JsonRpcServer, RpcMethod};
use serde_json::{json, Value};

pub struct PhantomLinkRpcServer {
    ghostwave_rpc: JsonRpcServer,
    phantomlink_state: PhantomLinkState,
}

impl PhantomLinkRpcServer {
    pub fn new() -> anyhow::Result<Self> {
        let mut ghostwave_rpc = JsonRpcServer::new("127.0.0.1:9001")?;

        // Register PhantomLink-specific methods
        ghostwave_rpc.register_method("phantomlink.set_mixer_level", Self::set_mixer_level)?;
        ghostwave_rpc.register_method("phantomlink.toggle_channel", Self::toggle_channel)?;
        ghostwave_rpc.register_method("phantomlink.get_status", Self::get_status)?;

        Ok(Self {
            ghostwave_rpc,
            phantomlink_state: PhantomLinkState::default(),
        })
    }

    fn set_mixer_level(&mut self, params: Value) -> anyhow::Result<Value> {
        let channel: usize = params["channel"].as_u64().unwrap() as usize;
        let level: f32 = params["level"].as_f64().unwrap() as f32;

        // Update PhantomLink mixer
        self.phantomlink_state.mixer.set_channel_level(channel, level)?;

        // Also update GhostWave if processing this channel
        if self.phantomlink_state.channels[channel].ghostwave_enabled {
            self.ghostwave_rpc.call("ghostwave.set_processing_level", json!({
                "channel": channel,
                "level": level
            }))?;
        }

        Ok(json!({"success": true}))
    }

    fn get_status(&self, _params: Value) -> anyhow::Result<Value> {
        let ghostwave_status = self.ghostwave_rpc.call("ghostwave.get_status", json!({}))?;

        Ok(json!({
            "phantomlink": {
                "mixer": self.phantomlink_state.mixer.get_levels(),
                "scarlett": self.phantomlink_state.scarlett.get_config(),
                "channels": self.phantomlink_state.channels
            },
            "ghostwave": ghostwave_status
        }))
    }
}
```

---

## 🔄 Migration Guide

### Removing Zig Integration

Step-by-step migration from experimental Zig code to GhostWave:

```rust
// 1. Remove Zig dependencies from Cargo.toml
// [dependencies]
// zig-nvidia-bindings = { path = "../zig-nvidia" }  // REMOVE

// 2. Replace Zig imports
// use zig_nvidia_bindings::*;  // REMOVE
use ghostwave_core::nvidia::*;   // ADD

// 3. Replace Zig function calls
impl PhantomLinkAudioEngine {
    pub fn migrate_nvidia_processing(&mut self) -> anyhow::Result<()> {
        // OLD Zig approach (REMOVE)
        // let zig_result = unsafe {
        //     zig_nvidia_process_audio(audio_ptr, length, strength)
        // };

        // NEW GhostWave approach (ADD)
        let input_buffer = AudioBuffer::from_slice(&self.current_audio);
        let mut output_buffer = AudioBuffer::new(self.current_audio.len());

        self.ghostwave_processor.process(&input_buffer, &mut output_buffer)?;
        output_buffer.copy_to_slice(&mut self.current_audio);

        Ok(())
    }
}

// 4. Update configuration structures
pub struct MigratedConfig {
    // OLD (REMOVE)
    // pub zig_nvidia_config: ZigNvidiaConfig,

    // NEW (ADD)
    pub ghostwave_config: ghostwave_core::Config,
}
```

### Configuration Migration

Convert existing PhantomLink configurations to work with GhostWave:

```rust
use ghostwave_core::Config;

pub fn migrate_phantomlink_config(
    old_config: &PhantomLinkConfig
) -> anyhow::Result<Config> {
    let mut ghostwave_config = Config::default();

    // Map PhantomLink's denoising levels to GhostWave profiles
    ghostwave_config.noise_suppression.strength = match old_config.denoising_mode {
        PhantomLinkDenoising::Basic => 0.3,
        PhantomLinkDenoising::Enhanced => 0.6,
        PhantomLinkDenoising::Maximum => 0.9,
    };

    // Map latency preferences
    ghostwave_config.latency_profile = if old_config.latency_target_ms <= 15.0 {
        LatencyProfile::Professional {
            target_latency_ms: old_config.latency_target_ms,
            max_acceptable_ms: 20.0,
            prefer_stability: true
        }
    } else {
        LatencyProfile::Balanced
    };

    // Map hardware settings
    ghostwave_config.audio.sample_rate = old_config.sample_rate;
    ghostwave_config.audio.frames_per_buffer = old_config.buffer_size;

    Ok(ghostwave_config)
}
```

---

## 🧪 Testing Integration

### Comprehensive Test Suite

Test PhantomLink + GhostWave integration thoroughly:

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use ghostwave_core::testing::AudioTestHarness;

    #[tokio::test]
    async fn test_phantomlink_ghostwave_latency() -> anyhow::Result<()> {
        let mut engine = PhantomLinkAudioEngine::new()?;
        let test_harness = AudioTestHarness::new();

        // Generate test audio
        let test_audio = test_harness.generate_noisy_signal(1024, 48000.0);
        let mut output = vec![0.0f32; 1024];

        // Measure processing latency
        let start = std::time::Instant::now();
        engine.process_audio_frame(&test_audio, &mut output)?;
        let latency = start.elapsed();

        // Verify latency target
        assert!(latency.as_millis() < 15, "Latency too high: {}ms", latency.as_millis());

        // Verify noise reduction
        let noise_reduction = test_harness.measure_noise_reduction(&test_audio, &output);
        assert!(noise_reduction > 10.0, "Insufficient noise reduction: {}dB", noise_reduction);

        Ok(())
    }

    #[test]
    fn test_scarlett_detection_migration() -> anyhow::Result<()> {
        // Test that GhostWave's device detection works for PhantomLink
        let detector = DeviceDetector::new()?;

        // This should work on the system with Scarlett Solo Gen 4
        if let Some(device) = detector.find_scarlett_solo_gen4()? {
            assert_eq!(device.name, "Scarlett Solo 4th Gen");
            assert!(device.supports_phantom_power);
            assert!(device.supports_96khz);
        }

        Ok(())
    }
}
```

---

## 🚀 Production Deployment

### Service Integration

Deploy PhantomLink with GhostWave as a unified service:

```toml
# /etc/systemd/user/phantomlink-enhanced.service
[Unit]
Description=PhantomLink Professional Mixer with GhostWave Integration
Requires=pipewire.service
After=pipewire.service

[Service]
Type=simple
ExecStart=/usr/local/bin/phantomlink --ghostwave-enabled --profile professional
Restart=always
RestartSec=5

# Real-time audio permissions
LimitRTPRIO=95
LimitMEMLOCK=unlimited

# NVIDIA GPU access
SupplementaryGroups=audio video

[Install]
WantedBy=default.target
```

### Performance Monitoring

Monitor the integrated system in production:

```rust
use ghostwave_core::monitoring::ProductionMonitor;

pub struct PhantomLinkProductionMonitor {
    ghostwave_monitor: ProductionMonitor,
    phantomlink_metrics: MetricsCollector,
    alert_system: AlertManager,
}

impl PhantomLinkProductionMonitor {
    pub async fn run_monitoring_loop(&mut self) -> anyhow::Result<()> {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            // Collect metrics from both systems
            let ghostwave_metrics = self.ghostwave_monitor.collect().await?;
            let phantomlink_metrics = self.phantomlink_metrics.collect().await?;

            // Check for performance issues
            if ghostwave_metrics.latency_ms > 20.0 {
                self.alert_system.send_alert("High audio latency detected").await?;
            }

            if ghostwave_metrics.xruns > 0 {
                self.alert_system.send_alert("Audio dropouts occurring").await?;
            }

            // Log combined metrics
            log::info!(
                "Audio Pipeline: PhantomLink={:.2}ms + GhostWave={:.2}ms = {:.2}ms total",
                phantomlink_metrics.processing_time_ms,
                ghostwave_metrics.latency_ms,
                phantomlink_metrics.processing_time_ms + ghostwave_metrics.latency_ms
            );
        }
    }
}
```

---

## 📈 Performance Benchmarks

### Expected Performance Improvements

| Metric | PhantomLink Standalone | PhantomLink + GhostWave | Improvement |
|--------|----------------------|-------------------------|-------------|
| **Audio Latency** | <20ms target | <15ms guaranteed | 25% reduction |
| **Noise Suppression** | Basic/Enhanced/Max | RTX Voice-grade AI | Professional quality |
| **GPU Utilization** | N/A (CPU only) | 15-25% RTX GPU | Offloaded processing |
| **Scarlett Detection** | Basic ALSA | Enhanced mapping | Improved reliability |
| **Real-time Stability** | Good | Excellent | Lock-free buffers |

### System Requirements

- **Minimum**: RTX 2060, 8GB RAM, 4-core CPU
- **Recommended**: RTX 3070+, 16GB RAM, 8-core CPU
- **Professional**: RTX 4080+, 32GB RAM, Ryzen 7/Intel i7

---

## 🔧 Troubleshooting

### Common Integration Issues

**1. NVIDIA Driver Conflicts**
```bash
# Verify open kernel modules are loaded
lsmod | grep nvidia
nvidia_uvm             1544192  0
nvidia_drm              73728  0
nvidia_modeset        1142784  1 nvidia_drm
nvidia              56623104  2 nvidia_uvm,nvidia_modeset

# Check CUDA availability
nvidia-smi
./phantomlink --ghostwave-doctor
```

**2. Audio Latency Issues**
```rust
// Debug latency in PhantomLink integration
use ghostwave_core::debug::LatencyProfiler;

let profiler = LatencyProfiler::new();
profiler.start_frame();

// Your audio processing
engine.process_audio_frame(&input, &mut output)?;

let latency = profiler.end_frame();
if latency.as_millis() > 15 {
    eprintln!("High latency: {:?}", latency);
}
```

**3. Migration Issues**
```bash
# Clean old Zig artifacts
cargo clean
rm -rf target/zig-cache

# Rebuild with GhostWave
cargo build --features ghostwave-integration
```

---

## 🎯 Future Enhancements

### Planned Integrations

- **Advanced AI Models** - Custom-trained models for specific microphone types
- **Multi-GPU Support** - Distributed processing across multiple RTX cards
- **Plugin Architecture** - GhostWave as VST plugin within PhantomLink
- **Cloud Integration** - Remote processing for resource-constrained systems

### Contributing to Integration

```bash
# Fork both repositories
git clone https://github.com/ghostkellz/phantomlink
git clone https://github.com/ghostkellz/ghostwave

# Create integration branch
cd phantomlink
git checkout -b feature/ghostwave-integration

# Make changes and test
cargo test --features ghostwave-integration
./target/debug/phantomlink --ghostwave-enabled --verbose

# Submit pull request with integration improvements
```

---

This integration guide provides a complete roadmap for migrating PhantomLink from experimental Zig-based NVIDIA support to GhostWave's production-ready Rust implementation with NVIDIA open kernel modules. The result is a more stable, performant, and maintainable professional audio solution.

For technical support, see the individual project documentation:
- [GhostWave Documentation](DOCS.md)
- [PhantomLink Repository](https://github.com/ghostkellz/phantomlink)