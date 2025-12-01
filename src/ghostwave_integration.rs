//! GhostWave Integration for PhantomLink
//!
//! Real-time RTX-accelerated AI noise suppression powered by GhostWave.
//! Provides NVIDIA Broadcast / RTX Voice quality on Linux.
//!
//! ## RTX 5090 Blackwell Support
//! - FP4 Tensor Core acceleration (2-3x faster than FP16)
//! - nvidia-open 580+ driver support
//! - TensorRT engine caching

#![allow(dead_code)] // Complete GhostWave integration API

use anyhow::{Result, Context};
use std::sync::{Arc, Mutex};

#[cfg(feature = "ghostwave")]
use ghostwave_core::{
    Config, GhostWaveProcessor,
    processor::{AudioProcessor, ProcessingProfile, ParamValue},
};

/// Denoise quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DenoiseQuality {
    /// Fast processing, good quality - suitable for gaming/voice chat
    Fast,
    /// Balanced quality and performance - default for streaming
    #[default]
    Balanced,
    /// Maximum quality - for recording/production
    Quality,
    /// Ultra quality with transformer model - RTX 40/50 recommended
    Ultra,
}

impl DenoiseQuality {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Fast => "Fast",
            Self::Balanced => "Balanced",
            Self::Quality => "Quality",
            Self::Ultra => "Ultra (RTX)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Fast => "Low latency, good for gaming",
            Self::Balanced => "Best balance of quality and speed",
            Self::Quality => "High quality for recording",
            Self::Ultra => "Maximum quality, requires RTX GPU",
        }
    }

    pub fn all() -> &'static [DenoiseQuality] {
        &[Self::Fast, Self::Balanced, Self::Quality, Self::Ultra]
    }
}

/// Processing profiles optimized for PhantomLink use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PhantomLinkProfile {
    /// Optimized for XLR microphones with Scarlett Solo
    /// Lower strength to preserve voice character, ultra-low latency
    XlrStudio,
    /// For streaming with aggressive noise suppression
    /// Higher strength, echo cancellation enabled
    #[default]
    Streaming,
    /// Balanced for general voice use
    Balanced,
    /// Minimal processing for music/instrument recording
    /// Preserves dynamics and harmonics
    Music,
}

impl PhantomLinkProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::XlrStudio => "XLR Studio",
            Self::Streaming => "Streaming",
            Self::Balanced => "Balanced",
            Self::Music => "Music",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::XlrStudio => "Low latency, preserves voice character",
            Self::Streaming => "Strong noise suppression, echo cancellation",
            Self::Balanced => "Good balance of quality and suppression",
            Self::Music => "Minimal processing, preserves dynamics",
        }
    }

    pub fn all() -> &'static [PhantomLinkProfile] {
        &[
            Self::XlrStudio,
            Self::Streaming,
            Self::Balanced,
            Self::Music,
        ]
    }

    #[cfg(feature = "ghostwave")]
    fn to_ghostwave_profile(&self) -> ProcessingProfile {
        match self {
            Self::XlrStudio => ProcessingProfile::Studio,
            Self::Streaming => ProcessingProfile::Streaming,
            Self::Balanced => ProcessingProfile::Balanced,
            Self::Music => ProcessingProfile::Studio,
        }
    }

    fn noise_strength(&self) -> f32 {
        match self {
            Self::XlrStudio => 0.5,
            Self::Streaming => 0.85,
            Self::Balanced => 0.65,
            Self::Music => 0.25,
        }
    }

    fn denoise_quality(&self) -> DenoiseQuality {
        match self {
            Self::XlrStudio => DenoiseQuality::Quality,
            Self::Streaming => DenoiseQuality::Ultra,
            Self::Balanced => DenoiseQuality::Balanced,
            Self::Music => DenoiseQuality::Fast,
        }
    }

    fn echo_cancellation_enabled(&self) -> bool {
        matches!(self, Self::Streaming)
    }
}

/// RTX GPU information and status
#[derive(Debug, Clone, Default)]
pub struct RtxStatus {
    pub available: bool,
    pub gpu_name: String,
    pub driver_version: String,
    pub processing_mode: String,
    pub precision: String,
    pub memory_used_mb: f32,
    pub memory_total_mb: f32,
    pub tensor_cores: bool,
    pub fp4_support: bool,
}

/// Real-time metrics from GhostWave processing
#[derive(Debug, Clone, Default)]
pub struct ProcessingMetrics {
    pub latency_ms: f32,
    pub cpu_usage: f32,
    pub gpu_usage: f32,
    pub noise_reduction_db: f32,
    pub voice_activity: bool,
    pub frames_processed: u64,
    pub xruns: u32,
}

/// Echo cancellation state
#[derive(Debug, Clone, Default)]
pub struct EchoCancellationState {
    pub enabled: bool,
    pub tail_length_ms: u32,
    pub suppression_level: f32,
}

/// Main GhostWave integration for PhantomLink
pub struct GhostWaveIntegration {
    #[cfg(feature = "ghostwave")]
    processor: Arc<Mutex<GhostWaveProcessor>>,

    // Current state
    enabled: bool,
    profile: PhantomLinkProfile,
    noise_strength: f32,
    denoise_quality: DenoiseQuality,

    // Echo cancellation
    echo_cancellation: EchoCancellationState,

    // RTX status
    rtx_status: RtxStatus,

    // Processing metrics
    metrics: ProcessingMetrics,

    // Audio configuration
    sample_rate: u32,
    channels: u32,
    buffer_size: usize,

    // Reference buffer for echo cancellation
    reference_buffer: Vec<f32>,
}

impl GhostWaveIntegration {
    /// Create new GhostWave integration with RTX acceleration
    pub fn new() -> Result<Self> {
        Self::with_config(48000, 2, 256)
    }

    /// Create with specific audio configuration
    pub fn with_config(sample_rate: u32, channels: u32, buffer_size: usize) -> Result<Self> {
        #[cfg(feature = "ghostwave")]
        let processor = {
            let config = Config::default()
                .with_overrides(Some(sample_rate), Some(buffer_size as u32));

            let mut proc = GhostWaveProcessor::new(config)
                .context("Failed to create GhostWave processor")?;

            proc.init(sample_rate, channels, buffer_size)
                .context("Failed to initialize GhostWave processor")?;

            Arc::new(Mutex::new(proc))
        };

        let mut integration = Self {
            #[cfg(feature = "ghostwave")]
            processor,
            enabled: true,
            profile: PhantomLinkProfile::default(),
            noise_strength: 0.65,
            denoise_quality: DenoiseQuality::Balanced,
            echo_cancellation: EchoCancellationState {
                enabled: false,
                tail_length_ms: 200,
                suppression_level: 0.8,
            },
            rtx_status: RtxStatus::default(),
            metrics: ProcessingMetrics::default(),
            sample_rate,
            channels,
            buffer_size,
            reference_buffer: vec![0.0; buffer_size * channels as usize],
        };

        integration.update_rtx_status();
        Ok(integration)
    }

    /// Process audio through GhostWave AI denoising pipeline
    pub fn process(&mut self, buffer: &mut [f32]) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        #[cfg(feature = "ghostwave")]
        {
            let start = std::time::Instant::now();

            if let Ok(mut proc) = self.processor.lock() {
                let frames = buffer.len() / self.channels as usize;
                proc.process_inplace(buffer, frames)?;
            }

            self.metrics.latency_ms = start.elapsed().as_secs_f32() * 1000.0;
            self.metrics.frames_processed += 1;
        }

        Ok(())
    }

    /// Set reference audio for echo cancellation (speaker output)
    pub fn set_reference_audio(&mut self, reference: &[f32]) {
        let len = reference.len().min(self.reference_buffer.len());
        self.reference_buffer[..len].copy_from_slice(&reference[..len]);
    }

    /// Set the processing profile
    pub fn set_profile(&mut self, profile: PhantomLinkProfile) -> Result<()> {
        self.profile = profile;
        self.noise_strength = profile.noise_strength();
        self.denoise_quality = profile.denoise_quality();
        self.echo_cancellation.enabled = profile.echo_cancellation_enabled();

        #[cfg(feature = "ghostwave")]
        if let Ok(mut proc) = self.processor.lock() {
            proc.set_profile(profile.to_ghostwave_profile())?;
            proc.set_param("noise_reduction_strength", ParamValue::Float(self.noise_strength))?;
        }

        Ok(())
    }

    /// Set noise suppression strength (0.0 - 1.0)
    pub fn set_noise_strength(&mut self, strength: f32) -> Result<()> {
        self.noise_strength = strength.clamp(0.0, 1.0);

        #[cfg(feature = "ghostwave")]
        if let Ok(mut proc) = self.processor.lock() {
            proc.set_param("noise_reduction_strength", ParamValue::Float(self.noise_strength))?;
        }

        Ok(())
    }

    /// Set denoise quality level
    pub fn set_denoise_quality(&mut self, quality: DenoiseQuality) {
        self.denoise_quality = quality;
    }

    /// Enable or disable echo cancellation
    pub fn set_echo_cancellation(&mut self, enabled: bool) {
        self.echo_cancellation.enabled = enabled;
    }

    /// Get echo cancellation state
    pub fn get_echo_cancellation(&self) -> &EchoCancellationState {
        &self.echo_cancellation
    }

    /// Enable or disable processing
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if RTX acceleration is active
    pub fn is_rtx_active(&self) -> bool {
        self.rtx_status.available && self.enabled
    }

    /// Get current RTX status
    pub fn get_rtx_status(&self) -> &RtxStatus {
        &self.rtx_status
    }

    /// Get current processing metrics
    pub fn get_metrics(&self) -> &ProcessingMetrics {
        &self.metrics
    }

    /// Get current profile
    pub fn get_profile(&self) -> PhantomLinkProfile {
        self.profile
    }

    /// Get noise strength
    pub fn get_noise_strength(&self) -> f32 {
        self.noise_strength
    }

    /// Get denoise quality
    pub fn get_denoise_quality(&self) -> DenoiseQuality {
        self.denoise_quality
    }

    /// Check if processing is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Update RTX status from processor and system
    fn update_rtx_status(&mut self) {
        let driver_info = detect_nvidia_driver();

        self.rtx_status.gpu_name = driver_info.gpu_name.clone();
        self.rtx_status.driver_version = driver_info.version.clone();
        self.rtx_status.memory_total_mb = driver_info.memory_total_mb;
        self.rtx_status.memory_used_mb = driver_info.memory_used_mb;

        #[cfg(feature = "ghostwave")]
        if let Ok(proc) = self.processor.lock() {
            self.rtx_status.available = proc.has_rtx_acceleration();
            self.rtx_status.processing_mode = proc.get_processing_mode();
        }

        #[cfg(not(feature = "ghostwave"))]
        {
            self.rtx_status.available = false;
            self.rtx_status.processing_mode = "CPU (GhostWave disabled)".to_string();
        }

        // Detect precision mode based on GPU
        if driver_info.is_blackwell {
            self.rtx_status.fp4_support = true;
            self.rtx_status.precision = "FP4 (Blackwell)".to_string();
            self.rtx_status.tensor_cores = true;
        } else if driver_info.is_ada {
            self.rtx_status.precision = "FP16 (Ada)".to_string();
            self.rtx_status.tensor_cores = true;
        } else if driver_info.is_ampere {
            self.rtx_status.precision = "FP16 (Ampere)".to_string();
            self.rtx_status.tensor_cores = true;
        } else {
            self.rtx_status.precision = "FP32 (CUDA)".to_string();
        }
    }

    /// Get a human-readable status string
    pub fn status_string(&self) -> String {
        if !self.enabled {
            return "Disabled".to_string();
        }

        if self.rtx_status.available {
            format!("{} - {}", self.rtx_status.precision, self.rtx_status.processing_mode)
        } else {
            format!("CPU - {}", self.rtx_status.processing_mode)
        }
    }
}

impl Default for GhostWaveIntegration {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            log::error!("Failed to create GhostWave integration: {}", e);
            Self {
                #[cfg(feature = "ghostwave")]
                processor: Arc::new(Mutex::new(
                    GhostWaveProcessor::new(Config::default()).expect("Default config should work")
                )),
                enabled: false,
                profile: PhantomLinkProfile::default(),
                noise_strength: 0.65,
                denoise_quality: DenoiseQuality::Balanced,
                echo_cancellation: EchoCancellationState::default(),
                rtx_status: RtxStatus::default(),
                metrics: ProcessingMetrics::default(),
                sample_rate: 48000,
                channels: 2,
                buffer_size: 256,
                reference_buffer: vec![0.0; 512],
            }
        })
    }
}

/// NVIDIA driver detection
pub fn detect_nvidia_driver() -> DriverInfo {
    let lsmod_output = std::process::Command::new("lsmod")
        .output()
        .ok();

    let (has_nvidia_drm, has_nvidia_modeset, has_nvidia_uvm) = lsmod_output
        .map(|output| {
            let lsmod_str = String::from_utf8_lossy(&output.stdout);
            (
                lsmod_str.contains("nvidia_drm"),
                lsmod_str.contains("nvidia_modeset"),
                lsmod_str.contains("nvidia_uvm"),
            )
        })
        .unwrap_or((false, false, false));

    let nvidia_smi = std::process::Command::new("nvidia-smi")
        .arg("--query-gpu=driver_version,name,memory.total,memory.used")
        .arg("--format=csv,noheader")
        .output()
        .ok();

    let (version, gpu_name, memory_total, memory_used) = nvidia_smi
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| {
            let parts: Vec<&str> = s.trim().split(", ").collect();
            (
                parts.first().unwrap_or(&"Unknown").to_string(),
                parts.get(1).unwrap_or(&"Unknown GPU").to_string(),
                parts.get(2).unwrap_or(&"0 MiB").replace(" MiB", "").parse::<f32>().unwrap_or(0.0),
                parts.get(3).unwrap_or(&"0 MiB").replace(" MiB", "").parse::<f32>().unwrap_or(0.0),
            )
        })
        .unwrap_or_else(|| ("Unknown".to_string(), "No GPU detected".to_string(), 0.0, 0.0));

    let version_num: u32 = version
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let is_open_driver = version_num >= 580 && has_nvidia_drm && has_nvidia_modeset;

    // Detect GPU generation
    let is_blackwell = gpu_name.contains("5090") || gpu_name.contains("5080") ||
                       gpu_name.contains("5070") || gpu_name.contains("5060");
    let is_ada = gpu_name.contains("4090") || gpu_name.contains("4080") ||
                 gpu_name.contains("4070") || gpu_name.contains("4060");
    let is_ampere = gpu_name.contains("3090") || gpu_name.contains("3080") ||
                   gpu_name.contains("3070") || gpu_name.contains("3060");

    DriverInfo {
        is_open_driver,
        has_cuda: has_nvidia_uvm,
        version,
        gpu_name,
        driver_type: if is_open_driver {
            "NVIDIA Open Kernel".to_string()
        } else if version_num > 0 {
            "NVIDIA Proprietary".to_string()
        } else {
            "Not detected".to_string()
        },
        memory_total_mb: memory_total,
        memory_used_mb: memory_used,
        is_blackwell,
        is_ada,
        is_ampere,
    }
}

/// NVIDIA driver information
#[derive(Debug, Clone)]
pub struct DriverInfo {
    pub is_open_driver: bool,
    pub has_cuda: bool,
    pub version: String,
    pub gpu_name: String,
    pub driver_type: String,
    pub memory_total_mb: f32,
    pub memory_used_mb: f32,
    pub is_blackwell: bool,
    pub is_ada: bool,
    pub is_ampere: bool,
}

impl Default for DriverInfo {
    fn default() -> Self {
        detect_nvidia_driver()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_names() {
        assert_eq!(PhantomLinkProfile::XlrStudio.name(), "XLR Studio");
        assert_eq!(PhantomLinkProfile::Streaming.name(), "Streaming");
    }

    #[test]
    fn test_driver_detection() {
        let info = detect_nvidia_driver();
        println!("Driver: {:?}", info);
        println!("GPU: {} (Blackwell: {})", info.gpu_name, info.is_blackwell);
    }

    #[test]
    fn test_denoise_quality() {
        assert_eq!(DenoiseQuality::Fast.name(), "Fast");
        assert_eq!(DenoiseQuality::Ultra.name(), "Ultra (RTX)");
    }

    #[test]
    #[cfg(feature = "ghostwave")]
    fn test_ghostwave_creation() {
        let integration = GhostWaveIntegration::new();
        assert!(integration.is_ok());
    }

    #[test]
    #[cfg(feature = "ghostwave")]
    fn test_audio_processing() {
        let mut integration = GhostWaveIntegration::new().unwrap();
        let mut buffer = vec![0.1f32; 512];
        let result = integration.process(&mut buffer);
        assert!(result.is_ok());
    }
}
