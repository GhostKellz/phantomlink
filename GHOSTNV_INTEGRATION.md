# PhantomLink Integration Guide

## GhostNV RTX Voice Enhanced Integration

This guide shows you how to integrate GhostNV's advanced RTX Voice noise cancellation into your PhantomLink (Wavelink clone) Rust project. GhostNV provides superior voice processing with music-aware enhancement, multi-user support, and real-time optimization.

---

## 🚀 Quick Start

### 1. Add GhostNV RTX Voice to Your Cargo.toml

```toml
[dependencies]
ghostnv-rtx-voice = { path = "../ghostnv/phantomlink-ffi" }
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
serde = { version = "1.0", features = ["derive"] }
```

### 2. Basic Integration Example

```rust
use ghostnv_rtx_voice::{RtxVoice, PhantomLink, SessionConfig, EnhancementMode};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::init();
    
    // Create RTX Voice instance
    let rtx_voice = RtxVoice::new().await?;
    
    // Create PhantomLink integration
    let phantomlink = Arc::new(PhantomLink::new(rtx_voice).await?);
    
    // Configure for voice chat with gaming optimization
    let config = SessionConfig::voice_chat()
        .with_low_latency()
        .with_enhancement(EnhancementMode::Aggressive);
    
    // Create a session for user
    phantomlink.create_session(12345, config).await?;
    
    println!("🎮 PhantomLink with GhostNV RTX Voice is ready!");
    
    Ok(())
}
```

---

## 🎯 Core Features

### Multi-User Voice Processing

PhantomLink supports multiple users simultaneously with individual voice profiles:

```rust
use ghostnv_rtx_voice::{PhantomLink, UserAudioInput, AudioBuffer};

async fn process_multi_user_chat(
    phantomlink: &PhantomLink,
    users: Vec<(u32, AudioBuffer)>,
    background_music: Option<AudioBuffer>
) -> Result<AudioBuffer, Box<dyn std::error::Error>> {
    // Convert to user inputs
    let user_inputs: Vec<UserAudioInput> = users
        .into_iter()
        .map(|(user_id, audio)| UserAudioInput::new(user_id, audio))
        .collect();
    
    // Process with music awareness
    let (mixed_output, stats) = phantomlink
        .process_multi_user(&user_inputs, background_music.as_ref())
        .await?;
    
    println!("📊 Processed {} voices, {:.1}ms latency", 
             stats.active_voice_count, stats.total_latency_ms);
    
    Ok(mixed_output)
}
```

### Music-Aware Voice Enhancement

GhostNV automatically detects and adapts to background music:

```rust
use ghostnv_rtx_voice::{MusicAnalysis, MusicGenre};

async fn adaptive_voice_processing(
    phantomlink: &PhantomLink,
    user_id: u32,
    voice: AudioBuffer,
    music: Option<AudioBuffer>
) -> Result<AudioBuffer, Box<dyn std::error::Error>> {
    // Analyze music if present
    if let Some(ref music_buffer) = music {
        let analysis = phantomlink.analyze_music(music_buffer).await?;
        
        match analysis.genre {
            MusicGenre::Electronic => {
                println!("🎵 Electronic music detected, adapting processing");
            },
            MusicGenre::Rock => {
                println!("🎸 Rock music detected, increasing voice isolation");
            },
            _ => {}
        }
    }
    
    // Process with music context
    let (enhanced_voice, result) = phantomlink
        .process_user_audio(user_id, &voice, music.as_ref())
        .await?;
    
    println!("✨ Voice quality: {:.1}%, suppression: {:.1}dB", 
             result.voice_quality_score * 100.0,
             result.music_suppression_db);
    
    Ok(enhanced_voice)
}
```

---

## 🎮 Gaming Integration

### Competitive Gaming Setup

For competitive gaming (esports), use maximum performance settings:

```rust
use ghostnv_rtx_voice::{PhantomLinkConfig, SessionConfig, UsageMode};

async fn setup_competitive_gaming(
    phantomlink: &PhantomLink
) -> Result<(), Box<dyn std::error::Error>> {
    // Configure for competitive gaming
    let config = PhantomLinkConfig {
        max_concurrent_users: 5, // Team size
        music_ducking_enabled: false, // No music in competitive
        real_time_eq_enabled: true,
        voice_isolation_strength: 0.95, // Maximum isolation
        music_suppression_level: 0.0,
        processing_latency_target_ms: 1.0, // Ultra-low latency
        ..Default::default()
    };
    
    phantomlink.configure(config).await?;
    
    // Create competitive session
    let session_config = SessionConfig {
        usage_mode: UsageMode::VoiceChat,
        enhancement_mode: EnhancementMode::Aggressive,
        latency_priority: true, // Prioritize latency over quality
        high_quality_mode: false,
    };
    
    for user_id in 1..=5 {
        phantomlink.create_session(user_id, session_config.clone()).await?;
    }
    
    println!("⚡ Competitive gaming mode activated!");
    Ok(())
}
```

### Real-Time Voice Chat Integration

Integrate with your existing voice chat system:

```rust
use tokio::sync::mpsc;
use ghostnv_rtx_voice::{AudioBuffer, SampleRate};

struct VoiceChatHandler {
    phantomlink: Arc<PhantomLink>,
    audio_tx: mpsc::Sender<(u32, AudioBuffer)>,
    audio_rx: mpsc::Receiver<(u32, AudioBuffer)>,
}

impl VoiceChatHandler {
    pub async fn new(phantomlink: Arc<PhantomLink>) -> Self {
        let (audio_tx, audio_rx) = mpsc::channel(100);
        
        Self {
            phantomlink,
            audio_tx,
            audio_rx,
        }
    }
    
    pub async fn start_processing(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some((user_id, raw_audio)) = self.audio_rx.recv().await {
            // Process audio with GhostNV
            let (enhanced_audio, stats) = self.phantomlink
                .process_user_audio(user_id, &raw_audio, None)
                .await?;
            
            // Send to other users (your existing voice chat logic)
            self.broadcast_to_users(user_id, enhanced_audio).await?;
            
            // Log processing stats
            if stats.latency_ms > 5.0 {
                tracing::warn!("High latency detected: {:.1}ms", stats.latency_ms);
            }
        }
        
        Ok(())
    }
    
    async fn broadcast_to_users(&self, sender_id: u32, audio: AudioBuffer) -> Result<(), Box<dyn std::error::Error>> {
        // Your existing broadcast logic here
        println!("📡 Broadcasting enhanced audio from user {}", sender_id);
        Ok(())
    }
}
```

---

## 🎵 Music Production Integration

### Live Streaming Setup

Perfect for streamers who want clean voice with background music:

```rust
use ghostnv_rtx_voice::{VoiceProfile, MusicPreferences, MusicGenre};

async fn setup_streaming_profile(
    phantomlink: &PhantomLink,
    streamer_id: u32
) -> Result<(), Box<dyn std::error::Error>> {
    // Create streaming session
    let config = SessionConfig::live_streaming()
        .with_enhancement(EnhancementMode::StudioQuality);
    
    phantomlink.create_session(streamer_id, config).await?;
    
    // Configure voice profile for streaming
    let voice_profile = VoiceProfile {
        user_id: streamer_id,
        clarity_enhancement: 0.8,
        base_settings: VoiceSettings {
            noise_suppression_level: 0.9,
            ambient_preservation: 0.3, // Preserve some room tone
            clarity_enhancement: 0.85,
            dynamic_range_compression: 0.7, // Consistent levels
            high_frequency_boost_db: 2.0, // Crisp voice
        },
        frequency_profile: FrequencyProfile {
            low_cut_hz: 80.0, // Remove rumble
            high_cut_hz: 12000.0, // Broadcast quality
            presence_boost_hz: 3000.0, // Voice presence
            presence_boost_db: 3.0,
        },
    };
    
    phantomlink.set_voice_profile(streamer_id, voice_profile).await?;
    
    // Configure music preferences
    let music_prefs = MusicPreferences {
        genre_preference: MusicGenre::Electronic, // Common for gaming streams
        dynamic_ducking: true, // Duck music when speaking
        preserve_stereo_field: true, // Keep music stereo
    };
    
    phantomlink.set_music_preferences(streamer_id, music_prefs).await?;
    
    println!("🎬 Streaming profile configured!");
    Ok(())
}
```

### Podcast Recording

High-quality recording setup:

```rust
async fn setup_podcast_recording(
    phantomlink: &PhantomLink,
    host_id: u32,
    guest_ids: Vec<u32>
) -> Result<(), Box<dyn std::error::Error>> {
    // Host configuration (primary speaker)
    let host_config = SessionConfig::podcast_recording()
        .with_enhancement(EnhancementMode::StudioQuality);
    
    phantomlink.create_session(host_id, host_config).await?;
    
    // Guest configurations
    for guest_id in guest_ids {
        let guest_config = SessionConfig::podcast_recording()
            .with_enhancement(EnhancementMode::Balanced);
        
        phantomlink.create_session(guest_id, guest_config).await?;
        
        // Set individual voice profiles
        let voice_profile = VoiceProfile::professional_recording(guest_id);
        phantomlink.set_voice_profile(guest_id, voice_profile).await?;
    }
    
    println!("🎙️ Podcast recording setup complete!");
    Ok(())
}
```

---

## 📊 Monitoring and Analytics

### Real-Time Performance Monitoring

Track performance metrics for optimization:

```rust
use std::time::Duration;

struct PerformanceMonitor {
    phantomlink: Arc<PhantomLink>,
}

impl PerformanceMonitor {
    pub async fn start_monitoring(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        
        loop {
            interval.tick().await;
            
            // Get active sessions
            let active_sessions = self.phantomlink.get_active_sessions().await;
            
            for user_id in active_sessions {
                if let Ok(stats) = self.phantomlink.get_user_stats(user_id).await {
                    // Log performance metrics
                    tracing::info!(
                        user_id = user_id,
                        avg_latency_us = stats.average_processing_time_us,
                        peak_latency_ms = stats.peak_latency_ms,
                        voice_activity = %format!("{:.1}%", stats.voice_activity_percentage * 100.0),
                        noise_reduction_db = stats.noise_reduction_average_db,
                        "Voice processing stats"
                    );
                    
                    // Alert on high latency
                    if stats.peak_latency_ms > 10.0 {
                        tracing::warn!(
                            user_id = user_id,
                            latency_ms = stats.peak_latency_ms,
                            "High latency detected"
                        );
                    }
                }
            }
        }
    }
}
```

### Analytics Integration

Collect analytics for quality improvements:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct VoiceAnalytics {
    session_id: u32,
    duration_seconds: u64,
    average_quality_score: f32,
    total_noise_reduction_db: f32,
    music_suppression_events: u32,
    latency_violations: u32,
}

async fn collect_session_analytics(
    phantomlink: &PhantomLink,
    user_id: u32,
    session_start: std::time::Instant
) -> Result<VoiceAnalytics, Box<dyn std::error::Error>> {
    let stats = phantomlink.get_user_stats(user_id).await?;
    let duration = session_start.elapsed().as_secs();
    
    let analytics = VoiceAnalytics {
        session_id: user_id,
        duration_seconds: duration,
        average_quality_score: 0.85, // Would be tracked during session
        total_noise_reduction_db: stats.noise_reduction_average_db,
        music_suppression_events: 0, // Would be tracked during session
        latency_violations: if stats.peak_latency_ms > 5.0 { 1 } else { 0 },
    };
    
    // Send to your analytics service
    println!("📈 Analytics: {:?}", analytics);
    
    Ok(analytics)
}
```

---

## 🛠️ Advanced Configuration

### Custom Audio Pipeline

Build your own audio processing pipeline:

```rust
use ghostnv_rtx_voice::{AudioBuffer, AudioFormat, SampleRate};

struct CustomAudioPipeline {
    phantomlink: Arc<PhantomLink>,
    input_format: AudioFormat,
    output_format: AudioFormat,
}

impl CustomAudioPipeline {
    pub fn new(phantomlink: Arc<PhantomLink>) -> Self {
        Self {
            phantomlink,
            input_format: AudioFormat {
                sample_rate: SampleRate::Hz48000,
                channels: 1, // Mono input
                bit_depth: 16,
            },
            output_format: AudioFormat {
                sample_rate: SampleRate::Hz48000,
                channels: 2, // Stereo output
                bit_depth: 24, // High quality output
            },
        }
    }
    
    pub async fn process_with_custom_effects(
        &self,
        user_id: u32,
        raw_audio: &[i16],
        music: Option<&[f32]>
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // Convert input format
        let input_buffer = self.convert_to_audio_buffer(raw_audio)?;
        let music_buffer = music.map(|m| AudioBuffer::from_f32_slice(m));
        
        // Process with GhostNV
        let (enhanced_audio, _stats) = self.phantomlink
            .process_user_audio(user_id, &input_buffer, music_buffer.as_ref())
            .await?;
        
        // Apply custom post-processing
        let processed = self.apply_custom_effects(enhanced_audio)?;
        
        // Convert to output format
        let output = self.convert_to_output_format(processed)?;
        
        Ok(output)
    }
    
    fn convert_to_audio_buffer(&self, raw: &[i16]) -> Result<AudioBuffer, Box<dyn std::error::Error>> {
        // Convert i16 samples to f32
        let f32_samples: Vec<f32> = raw
            .iter()
            .map(|&sample| sample as f32 / 32768.0)
            .collect();
        
        Ok(AudioBuffer::from_f32_slice(&f32_samples))
    }
    
    fn apply_custom_effects(&self, audio: AudioBuffer) -> Result<AudioBuffer, Box<dyn std::error::Error>> {
        // Your custom audio effects here
        // Example: reverb, EQ, compression, etc.
        Ok(audio)
    }
    
    fn convert_to_output_format(&self, audio: AudioBuffer) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // Convert to your output format
        Ok(audio.to_f32_vec())
    }
}
```

---

## 🚨 Error Handling and Recovery

### Robust Error Handling

Handle common errors gracefully:

```rust
use ghostnv_rtx_voice::{GhostNVError, PhantomLink};

async fn robust_voice_processing(
    phantomlink: &PhantomLink,
    user_id: u32,
    audio: AudioBuffer
) -> Result<AudioBuffer, Box<dyn std::error::Error>> {
    match phantomlink.process_user_audio(user_id, &audio, None).await {
        Ok((enhanced_audio, _stats)) => Ok(enhanced_audio),
        
        Err(GhostNVError::SessionNotFound) => {
            // Recreate session
            tracing::warn!("Session not found for user {}, recreating", user_id);
            let config = SessionConfig::voice_chat();
            phantomlink.create_session(user_id, config).await?;
            
            // Retry processing
            let (enhanced_audio, _) = phantomlink
                .process_user_audio(user_id, &audio, None)
                .await?;
            Ok(enhanced_audio)
        },
        
        Err(GhostNVError::MaxUsersExceeded) => {
            tracing::error!("Maximum users exceeded, falling back to passthrough");
            Ok(audio) // Return original audio
        },
        
        Err(GhostNVError::InitializationFailed) => {
            tracing::error!("GhostNV initialization failed, check GPU drivers");
            Err("GPU acceleration unavailable".into())
        },
        
        Err(e) => {
            tracing::error!("Voice processing error: {:?}", e);
            Ok(audio) // Fallback to original audio
        }
    }
}
```

### Automatic Recovery

Implement automatic recovery for critical systems:

```rust
struct ResilientVoiceProcessor {
    phantomlink: Option<Arc<PhantomLink>>,
    rtx_voice: Option<Arc<RtxVoice>>,
    retry_count: u32,
    max_retries: u32,
}

impl ResilientVoiceProcessor {
    pub async fn new() -> Self {
        Self {
            phantomlink: None,
            rtx_voice: None,
            retry_count: 0,
            max_retries: 3,
        }
    }
    
    pub async fn ensure_initialized(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.phantomlink.is_some() {
            return Ok(());
        }
        
        if self.retry_count >= self.max_retries {
            return Err("Max retry attempts exceeded".into());
        }
        
        self.retry_count += 1;
        
        match self.try_initialize().await {
            Ok(_) => {
                self.retry_count = 0; // Reset on success
                Ok(())
            },
            Err(e) => {
                tracing::warn!("Initialization attempt {} failed: {}", self.retry_count, e);
                tokio::time::sleep(Duration::from_secs(self.retry_count as u64)).await;
                Err(e)
            }
        }
    }
    
    async fn try_initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let rtx_voice = RtxVoice::new().await?;
        let phantomlink = PhantomLink::new(rtx_voice.clone()).await?;
        
        self.rtx_voice = Some(Arc::new(rtx_voice));
        self.phantomlink = Some(Arc::new(phantomlink));
        
        tracing::info!("Voice processor initialized successfully");
        Ok(())
    }
}
```

---

## 🔧 Build Configuration

### Cargo.toml for Production

```toml
[package]
name = "phantomlink-voice"
version = "1.0.0"
edition = "2021"

[dependencies]
# GhostNV RTX Voice integration
ghostnv-rtx-voice = { path = "../ghostnv/phantomlink-ffi" }

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Logging and tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Audio processing (if needed)
cpal = "0.15" # For audio I/O
hound = "3.5" # For WAV file support

# Error handling
anyhow = "1.0"
thiserror = "1.0"

[build-dependencies]
# Link GhostNV native library
cc = "1.0"

[profile.release]
# Optimize for performance
opt-level = 3
lto = true
codegen-units = 1
```

### Build Script (build.rs)

```rust
// build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    let ghostnv_path = env::var("GHOSTNV_PATH")
        .unwrap_or_else(|_| "../ghostnv/zig-nvidia".to_string());
    
    // Link GhostNV libraries
    println!("cargo:rustc-link-search=native={}/lib", ghostnv_path);
    println!("cargo:rustc-link-lib=static=ghostnv-rtx-voice");
    println!("cargo:rustc-link-lib=static=ghostnv-phantomlink");
    
    // Link system libraries
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=dylib=asound"); // ALSA
        println!("cargo:rustc-link-lib=dylib=pulse");   // PulseAudio
    }
    
    // Rerun if GhostNV library changes
    println!("cargo:rerun-if-changed={}/lib", ghostnv_path);
}
```

---

## 🎯 Production Deployment

### Docker Configuration

```dockerfile
# Dockerfile
FROM nvidia/cuda:12.0-runtime-ubuntu22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    libasound2-dev \
    libpulse-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy application
COPY target/release/phantomlink-voice /usr/local/bin/
COPY ghostnv/ /opt/ghostnv/

# Set environment
ENV GHOSTNV_PATH=/opt/ghostnv
ENV RUST_LOG=info

# Run application
CMD ["phantomlink-voice"]
```

### Systemd Service

```ini
# /etc/systemd/system/phantomlink-voice.service
[Unit]
Description=PhantomLink Voice Service with GhostNV RTX Voice
After=network.target

[Service]
Type=simple
User=phantomlink
Group=audio
ExecStart=/usr/local/bin/phantomlink-voice
Restart=always
RestartSec=10

# Environment
Environment=RUST_LOG=info
Environment=GHOSTNV_PATH=/opt/ghostnv

# Security
NoNewPrivileges=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

---

## 📚 API Reference

### Core Types

```rust
// Audio buffer for processing
pub struct AudioBuffer {
    pub data: Vec<f32>,
    pub sample_rate: SampleRate,
    pub channels: u8,
}

// Processing result with statistics
pub struct AudioResult {
    pub processing_time_us: u32,
    pub voice_quality_score: f32,
    pub music_suppression_db: f32,
    pub voice_clarity_enhancement: f32,
    pub latency_ms: f32,
    pub session_id: u32,
}

// User session configuration
pub struct SessionConfig {
    pub usage_mode: UsageMode,
    pub enhancement_mode: EnhancementMode,
    pub latency_priority: bool,
    pub high_quality_mode: bool,
}

// Voice profile for customization
pub struct VoiceProfile {
    pub user_id: u32,
    pub clarity_enhancement: f32,
    pub base_settings: VoiceSettings,
    pub frequency_profile: FrequencyProfile,
}
```

### Error Types

```rust
pub enum GhostNVError {
    InitializationFailed,
    SessionNotFound,
    MaxUsersExceeded,
    InvalidParameter(String),
    ProcessingError(String),
    HardwareError(String),
}
```

---

## 🏆 Best Practices

### Performance Optimization

1. **Reuse Sessions**: Create sessions once and reuse them
2. **Batch Processing**: Process multiple users together when possible  
3. **Monitor Latency**: Keep processing under 5ms for real-time applications
4. **Profile Usage**: Use profile-guided optimization for hot paths
5. **Cache Audio Buffers**: Reuse audio buffer allocations

### Quality Settings

1. **Gaming**: Use aggressive enhancement with low latency priority
2. **Streaming**: Use balanced enhancement with music awareness
3. **Recording**: Use studio quality with maximum enhancement
4. **Conferencing**: Use minimal enhancement for natural sound

### Resource Management

1. **GPU Memory**: Monitor GPU memory usage for large sessions
2. **CPU Cores**: Reserve cores for audio processing if possible
3. **Thread Pools**: Use async/await for better concurrency
4. **Buffer Sizes**: Use 512-sample buffers for optimal latency/quality

---

## ❓ Troubleshooting

### Common Issues

**Q: "InitializationFailed" error on startup**
A: Check that NVIDIA drivers are installed and GPU supports CUDA 11.0+

**Q: High latency in competitive gaming**
A: Enable competitive mode and set latency_priority to true

**Q: Poor noise cancellation with music**
A: Ensure music_ducking_enabled is true and set appropriate genre preferences

**Q: Memory usage growing over time**
A: Implement session cleanup and monitor for memory leaks

### Debug Logging

```rust
// Enable debug logging
RUST_LOG=ghostnv_rtx_voice=debug,phantomlink=debug cargo run
```

### Performance Profiling

```rust
// Add timing measurements
let start = std::time::Instant::now();
let result = phantomlink.process_user_audio(user_id, &audio, None).await?;
println!("Processing took: {:?}", start.elapsed());
```

---

## 🚀 What's Next?

1. **Spatial Audio**: 3D positional voice chat
2. **AI Voice Cloning**: Real-time voice transformation
3. **Emotion Detection**: Mood-aware voice processing
4. **Band Isolation**: Multi-instrument music separation
5. **Real-time Translation**: Language translation with voice preservation

---

## 📞 Support

- **GitHub Issues**: Report bugs and feature requests
- **Discord**: Join our community for real-time help
- **Documentation**: Check the full API documentation
- **Examples**: See more examples in the `/examples` directory

---

**🎮 Game on with GhostNV RTX Voice Enhanced!**

*The ultimate voice processing solution for competitive gaming, streaming, and music production.*