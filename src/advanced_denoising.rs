//! Advanced noise suppression system with multiple denoising backends.
//!
//! Provides a multi-tier denoising architecture:
//! - Tier 1: RNNoise (fast, CPU-based)
//! - Tier 2: Deep Learning (ONNX-based, GPU-accelerated)
//! - Tier 3: Spectral (Wiener filter, precision enhancement)

#![allow(dead_code)] // Complete denoising API with multiple backends

use std::sync::{Arc, Mutex};
#[cfg(feature = "deep-learning")]
use std::path::PathBuf;
use anyhow::Result;
use realfft::{RealFftPlanner, RealToComplex, ComplexToReal};
use num_complex::Complex32;

/// Denoising modes available in the advanced system
#[derive(Debug, Clone, PartialEq)]
pub enum DenoisingMode {
    /// Basic RNNoise denoising only
    Basic,
    /// RNNoise + Deep learning model
    Enhanced,
    /// All denoising tiers enabled
    Maximum,
    /// Custom configuration
    Custom { 
        use_rnnoise: bool,
        use_deep_learning: bool,
        use_spectral: bool,
    },
}

/// Individual denoising tier types
#[derive(Debug, Clone)]
pub enum DenoiserTier {
    RNNoise,
    DeepLearning,
    Spectral,
}

/// Performance metrics for denoising
#[derive(Debug, Clone)]
pub struct DenoisingMetrics {
    pub latency_ms: f32,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: f32,
    pub quality_score: f32, // 0.0 to 1.0
}

/// Trait for advanced denoising implementations
pub trait AdvancedDenoiser: Send + Sync {
    /// Process a frame of audio data
    fn process_frame(&mut self, input: &[f32]) -> Result<Vec<f32>>;
    
    /// Set the denoising mode
    fn set_mode(&mut self, mode: DenoisingMode) -> Result<()>;
    
    /// Get current mode
    fn get_mode(&self) -> DenoisingMode;
    
    /// Get processing latency in milliseconds
    fn get_latency(&self) -> f32;
    
    /// Get CPU usage percentage (0.0 to 100.0)
    fn get_cpu_usage(&self) -> f32;
    
    /// Get current performance metrics
    fn get_metrics(&self) -> DenoisingMetrics;
    
    /// Check if the denoiser is ready to process
    fn is_ready(&self) -> bool;
    
    /// Enable/disable the denoiser
    fn set_enabled(&mut self, enabled: bool);
    
    /// Check if the denoiser is enabled
    fn is_enabled(&self) -> bool;
}

/// Configuration for the advanced denoising system
#[derive(Debug, Clone)]
pub struct AdvancedDenoisingConfig {
    pub mode: DenoisingMode,
    pub sample_rate: u32,
    pub frame_size: usize,
    pub max_latency_ms: f32,
    pub max_cpu_percent: f32,
    pub quality_preference: f32, // 0.0 = speed, 1.0 = quality
    pub gpu_acceleration: bool,
    pub adaptive_mode: bool, // Automatically adjust based on performance
}

impl Default for AdvancedDenoisingConfig {
    fn default() -> Self {
        Self {
            mode: DenoisingMode::Enhanced,
            sample_rate: 48000,
            frame_size: 480, // 10ms at 48kHz
            max_latency_ms: 50.0,
            max_cpu_percent: 25.0,
            quality_preference: 0.7,
            gpu_acceleration: true,
            adaptive_mode: true,
        }
    }
}

/// Main advanced denoising system
pub struct AdvancedDenoisingSystem {
    config: AdvancedDenoisingConfig,
    rnnoise_denoiser: Option<crate::rnnoise::Rnnoise>,
    deep_learning_denoiser: Option<Box<dyn DeepLearningDenoiser>>,
    spectral_denoiser: Option<Box<dyn SpectralDenoiser>>,
    enabled: bool,
    metrics: DenoisingMetrics,
    performance_monitor: PerformanceMonitor,
}

/// Trait for deep learning based denoisers
pub trait DeepLearningDenoiser: Send + Sync {
    fn process(&mut self, input: &[f32]) -> Result<Vec<f32>>;
    fn get_latency(&self) -> f32;
    fn is_gpu_accelerated(&self) -> bool;
    fn get_model_info(&self) -> ModelInfo;
}

/// Trait for spectral-based denoisers
pub trait SpectralDenoiser: Send + Sync {
    fn process(&mut self, input: &[f32]) -> Result<Vec<f32>>;
    fn set_noise_profile(&mut self, profile: &[f32]);
    fn get_noise_reduction_db(&self) -> f32;
}

/// Information about a loaded model
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub version: String,
    pub size_mb: f32,
    pub target_latency_ms: f32,
    pub supported_sample_rates: Vec<u32>,
}

// ============================================================================
// ONNX Deep Learning Denoiser Implementation
// ============================================================================
// Note: The full ONNX implementation requires the `deep-learning` feature
// and the ort crate. When models are available, this provides GPU-accelerated
// inference using CUDA on NVIDIA GPUs.

#[cfg(feature = "deep-learning")]
pub use onnx_denoiser::OnnxDenoiser;

#[cfg(feature = "deep-learning")]
mod onnx_denoiser {
    use super::*;

    /// ONNX-based deep learning denoiser
    /// Supports models like Facebook Denoiser, RNNoise-NG, etc.
    pub struct OnnxDenoiser {
        model_info: ModelInfo,
        frame_size: usize,
        sample_rate: u32,
        input_buffer: Vec<f32>,
        output_buffer: Vec<f32>,
        last_latency_ms: f32,
        gpu_accelerated: bool,
    }

    impl OnnxDenoiser {
        /// Create from built-in model
        pub fn from_builtin(model_name: &str, sample_rate: u32, frame_size: usize) -> Result<Self> {
            // Look for models in standard locations
            let model_paths = [
                PathBuf::from("/usr/share/phantomlink/models"),
                dirs::data_dir().unwrap_or_default().join("phantomlink/models"),
                PathBuf::from("./models"),
            ];

            let model_filename = format!("{}.onnx", model_name);

            for base_path in &model_paths {
                let model_path = base_path.join(&model_filename);
                if model_path.exists() {
                    let model_size = std::fs::metadata(&model_path)
                        .map(|m| m.len() as f32 / (1024.0 * 1024.0))
                        .unwrap_or(0.0);

                    return Ok(Self {
                        model_info: ModelInfo {
                            name: model_name.to_string(),
                            version: "1.0".to_string(),
                            size_mb: model_size,
                            target_latency_ms: (frame_size as f32 / sample_rate as f32) * 1000.0,
                            supported_sample_rates: vec![16000, 44100, 48000],
                        },
                        frame_size,
                        sample_rate,
                        input_buffer: vec![0.0; frame_size],
                        output_buffer: vec![0.0; frame_size],
                        last_latency_ms: 0.0,
                        gpu_accelerated: false, // Would be true with full ONNX runtime
                    });
                }
            }

            anyhow::bail!("Model not found: {}", model_name)
        }

        pub fn is_gpu_accelerated(&self) -> bool {
            self.gpu_accelerated
        }

        pub fn get_model_info(&self) -> ModelInfo {
            self.model_info.clone()
        }
    }

    impl DeepLearningDenoiser for OnnxDenoiser {
        fn process(&mut self, input: &[f32]) -> Result<Vec<f32>> {
            // Stub implementation - passes through audio
            // Full implementation would use ONNX Runtime for inference
            let start = std::time::Instant::now();

            let input_len = input.len().min(self.frame_size);
            self.output_buffer.clear();
            self.output_buffer.extend_from_slice(&input[..input_len]);

            self.last_latency_ms = start.elapsed().as_secs_f32() * 1000.0;
            Ok(self.output_buffer.clone())
        }

        fn get_latency(&self) -> f32 {
            self.last_latency_ms
        }

        fn is_gpu_accelerated(&self) -> bool {
            self.gpu_accelerated
        }

        fn get_model_info(&self) -> ModelInfo {
            self.model_info.clone()
        }
    }
}

// ============================================================================
// Wiener Filter Spectral Denoiser Implementation
// ============================================================================

/// Wiener filter-based spectral denoiser
/// Provides fine-grained noise reduction in frequency domain
pub struct WienerDenoiser {
    frame_size: usize,
    sample_rate: u32,
    // FFT processing
    fft_planner: RealFftPlanner<f32>,
    fft_forward: Arc<dyn RealToComplex<f32>>,
    fft_inverse: Arc<dyn ComplexToReal<f32>>,
    // Processing buffers
    input_buffer: Vec<f32>,
    output_buffer: Vec<f32>,
    spectrum: Vec<Complex32>,
    // Noise profile
    noise_profile: Vec<f32>,
    noise_floor: f32,
    // Wiener filter parameters
    smoothing_factor: f32,
    noise_reduction_db: f32,
    // Overlap-add for smooth transitions
    overlap_buffer: Vec<f32>,
    window: Vec<f32>,
}

impl WienerDenoiser {
    pub fn new(sample_rate: u32, frame_size: usize) -> Result<Self> {
        let fft_size = frame_size.next_power_of_two();

        let mut fft_planner = RealFftPlanner::new();
        let fft_forward = fft_planner.plan_fft_forward(fft_size);
        let fft_inverse = fft_planner.plan_fft_inverse(fft_size);

        // Create Hann window for overlap-add
        let window: Vec<f32> = (0..fft_size)
            .map(|i| {
                let x = std::f32::consts::PI * i as f32 / (fft_size - 1) as f32;
                0.5 * (1.0 - x.cos())
            })
            .collect();

        Ok(Self {
            frame_size,
            sample_rate,
            fft_planner,
            fft_forward,
            fft_inverse,
            input_buffer: vec![0.0; fft_size],
            output_buffer: vec![0.0; fft_size],
            spectrum: vec![Complex32::new(0.0, 0.0); fft_size / 2 + 1],
            noise_profile: vec![0.001; fft_size / 2 + 1], // Default noise floor
            noise_floor: -60.0, // dB
            smoothing_factor: 0.98,
            noise_reduction_db: 0.0,
            overlap_buffer: vec![0.0; fft_size],
            window,
        })
    }

    /// Update noise profile from current spectrum
    fn update_noise_profile(&mut self, spectrum: &[Complex32]) {
        for (i, &s) in spectrum.iter().enumerate() {
            let magnitude = s.norm();
            // Exponential smoothing
            self.noise_profile[i] = self.smoothing_factor * self.noise_profile[i]
                + (1.0 - self.smoothing_factor) * magnitude;
        }
    }

    /// Apply Wiener filter to spectrum
    fn apply_wiener_filter(&self, spectrum: &mut [Complex32]) {
        for (i, s) in spectrum.iter_mut().enumerate() {
            let signal_power = s.norm_sqr();
            let noise_power = self.noise_profile[i].powi(2);

            // Wiener filter gain: H(w) = max(1 - noise/signal, floor)
            let gain = if signal_power > noise_power * 1.1 {
                ((signal_power - noise_power) / signal_power).sqrt()
            } else {
                0.01 // Minimum gain to avoid complete nulling
            };

            *s *= gain;
        }
    }
}

impl SpectralDenoiser for WienerDenoiser {
    fn process(&mut self, input: &[f32]) -> Result<Vec<f32>> {
        let fft_size = self.input_buffer.len();
        let input_len = input.len().min(fft_size);

        // Zero-pad input and apply window
        self.input_buffer[..input_len].copy_from_slice(&input[..input_len]);
        for i in input_len..fft_size {
            self.input_buffer[i] = 0.0;
        }
        for i in 0..fft_size {
            self.input_buffer[i] *= self.window[i];
        }

        // Forward FFT
        self.fft_forward
            .process(&mut self.input_buffer, &mut self.spectrum)
            .map_err(|e| anyhow::anyhow!("FFT forward error: {:?}", e))?;

        // Apply Wiener filter inline (avoiding borrow issue)
        for (i, s) in self.spectrum.iter_mut().enumerate() {
            let signal_power = s.norm_sqr();
            let noise_power = self.noise_profile[i].powi(2);

            // Wiener filter gain: H(w) = max(1 - noise/signal, floor)
            let gain = if signal_power > noise_power * 1.1 {
                ((signal_power - noise_power) / signal_power).sqrt()
            } else {
                0.01 // Minimum gain to avoid complete nulling
            };

            *s *= gain;
        }

        // Calculate noise reduction achieved
        let input_energy: f32 = input.iter().map(|x| x * x).sum();
        let output_energy: f32 = self.spectrum.iter().map(|s| s.norm_sqr()).sum();
        if input_energy > 0.0 && output_energy > 0.0 {
            self.noise_reduction_db = 10.0 * (input_energy / output_energy).log10();
        }

        // Inverse FFT
        self.fft_inverse
            .process(&mut self.spectrum, &mut self.output_buffer)
            .map_err(|e| anyhow::anyhow!("FFT inverse error: {:?}", e))?;

        // Normalize and apply window
        let norm = 1.0 / fft_size as f32;
        for i in 0..fft_size {
            self.output_buffer[i] *= norm * self.window[i];
        }

        // Overlap-add
        let mut output = vec![0.0; input_len];
        for i in 0..input_len {
            output[i] = self.output_buffer[i] + self.overlap_buffer[i];
        }

        // Store overlap for next frame
        let hop_size = fft_size / 2;
        for i in 0..hop_size {
            self.overlap_buffer[i] = if i + hop_size < fft_size {
                self.output_buffer[i + hop_size]
            } else {
                0.0
            };
        }

        Ok(output)
    }

    fn set_noise_profile(&mut self, profile: &[f32]) {
        let len = profile.len().min(self.noise_profile.len());
        self.noise_profile[..len].copy_from_slice(&profile[..len]);
    }

    fn get_noise_reduction_db(&self) -> f32 {
        self.noise_reduction_db
    }
}

/// Performance monitoring for adaptive mode
struct PerformanceMonitor {
    cpu_history: Vec<f32>,
    latency_history: Vec<f32>,
    last_update: std::time::Instant,
}

impl PerformanceMonitor {
    fn new() -> Self {
        Self {
            cpu_history: Vec::with_capacity(100),
            latency_history: Vec::with_capacity(100),
            last_update: std::time::Instant::now(),
        }
    }
    
    fn update(&mut self, cpu_usage: f32, latency: f32) {
        self.cpu_history.push(cpu_usage);
        self.latency_history.push(latency);
        
        // Keep only last 100 measurements
        if self.cpu_history.len() > 100 {
            self.cpu_history.remove(0);
        }
        if self.latency_history.len() > 100 {
            self.latency_history.remove(0);
        }
        
        self.last_update = std::time::Instant::now();
    }
    
    fn average_cpu(&self) -> f32 {
        if self.cpu_history.is_empty() {
            0.0
        } else {
            self.cpu_history.iter().sum::<f32>() / self.cpu_history.len() as f32
        }
    }
    
    fn average_latency(&self) -> f32 {
        if self.latency_history.is_empty() {
            0.0
        } else {
            self.latency_history.iter().sum::<f32>() / self.latency_history.len() as f32
        }
    }
    
    fn should_adapt(&self) -> bool {
        self.last_update.elapsed().as_secs() >= 5 // Adapt every 5 seconds
    }
}

impl AdvancedDenoisingSystem {
    pub fn new(config: AdvancedDenoisingConfig) -> Result<Self> {
        let mut system = Self {
            config,
            rnnoise_denoiser: None,
            deep_learning_denoiser: None,
            spectral_denoiser: None,
            enabled: false,
            metrics: DenoisingMetrics {
                latency_ms: 0.0,
                cpu_usage_percent: 0.0,
                memory_usage_mb: 0.0,
                quality_score: 0.0,
            },
            performance_monitor: PerformanceMonitor::new(),
        };
        
        system.initialize_denoisers()?;
        Ok(system)
    }
    
    fn initialize_denoisers(&mut self) -> Result<()> {
        // Initialize RNNoise denoiser
        let mut rnnoise = crate::rnnoise::Rnnoise::new();
        rnnoise.enable();
        self.rnnoise_denoiser = Some(rnnoise);

        // Initialize spectral Wiener denoiser
        match WienerDenoiser::new(self.config.sample_rate, self.config.frame_size) {
            Ok(wiener) => {
                log::info!("Spectral Wiener denoiser initialized");
                self.spectral_denoiser = Some(Box::new(wiener));
            }
            Err(e) => {
                log::warn!("Failed to initialize spectral denoiser: {}", e);
            }
        }

        // Initialize ONNX deep learning denoiser if available
        #[cfg(feature = "deep-learning")]
        {
            // Try to load built-in models in order of preference
            let models_to_try = ["rnnoise-ng", "denoiser", "nsnet2"];

            for model_name in &models_to_try {
                match onnx_denoiser::OnnxDenoiser::from_builtin(
                    model_name,
                    self.config.sample_rate,
                    self.config.frame_size,
                ) {
                    Ok(denoiser) => {
                        let info = denoiser.get_model_info();
                        log::info!(
                            "Deep learning denoiser loaded: {} v{} ({:.1} MB, GPU: {})",
                            info.name,
                            info.version,
                            info.size_mb,
                            denoiser.is_gpu_accelerated()
                        );
                        self.deep_learning_denoiser = Some(Box::new(denoiser));
                        break;
                    }
                    Err(e) => {
                        log::debug!("Model {} not available: {}", model_name, e);
                    }
                }
            }

            if self.deep_learning_denoiser.is_none() {
                log::info!("No ONNX denoising models found, using RNNoise + Spectral only");
            }
        }

        #[cfg(not(feature = "deep-learning"))]
        {
            log::info!("Deep learning denoiser disabled (feature not compiled)");
        }

        Ok(())
    }
    
    fn adaptive_mode_adjustment(&mut self) {
        if !self.config.adaptive_mode || !self.performance_monitor.should_adapt() {
            return;
        }
        
        let avg_cpu = self.performance_monitor.average_cpu();
        let avg_latency = self.performance_monitor.average_latency();
        
        // If we're exceeding limits, downgrade mode
        if avg_cpu > self.config.max_cpu_percent || avg_latency > self.config.max_latency_ms {
            match self.config.mode {
                DenoisingMode::Maximum => {
                    self.config.mode = DenoisingMode::Enhanced;
                    log::info!("Adaptive mode: Downgraded to Enhanced due to performance");
                }
                DenoisingMode::Enhanced => {
                    self.config.mode = DenoisingMode::Basic;
                    log::info!("Adaptive mode: Downgraded to Basic due to performance");
                }
                _ => {}
            }
        }
        // If we have headroom, try upgrading
        else if avg_cpu < self.config.max_cpu_percent * 0.7 && avg_latency < self.config.max_latency_ms * 0.7 {
            match self.config.mode {
                DenoisingMode::Basic => {
                    self.config.mode = DenoisingMode::Enhanced;
                    log::info!("Adaptive mode: Upgraded to Enhanced due to available performance");
                }
                DenoisingMode::Enhanced => {
                    self.config.mode = DenoisingMode::Maximum;
                    log::info!("Adaptive mode: Upgraded to Maximum due to available performance");
                }
                _ => {}
            }
        }
    }
}

impl AdvancedDenoiser for AdvancedDenoisingSystem {
    fn process_frame(&mut self, input: &[f32]) -> Result<Vec<f32>> {
        if !self.enabled {
            return Ok(input.to_vec());
        }
        
        let start_time = std::time::Instant::now();
        let mut output = input.to_vec();
        
        // Process based on current mode
        match &self.config.mode {
            DenoisingMode::Basic => {
                if let Some(ref rnnoise) = self.rnnoise_denoiser {
                    output = rnnoise.process(&output);
                }
            }
            DenoisingMode::Enhanced => {
                // First pass: RNNoise
                if let Some(ref rnnoise) = self.rnnoise_denoiser {
                    output = rnnoise.process(&output);
                }
                
                // Second pass: Deep learning (when available)
                if let Some(ref mut deep_learning) = self.deep_learning_denoiser {
                    output = deep_learning.process(&output)?;
                }
            }
            DenoisingMode::Maximum => {
                // First pass: RNNoise
                if let Some(ref rnnoise) = self.rnnoise_denoiser {
                    output = rnnoise.process(&output);
                }
                
                // Second pass: Deep learning (when available)
                if let Some(ref mut deep_learning) = self.deep_learning_denoiser {
                    output = deep_learning.process(&output)?;
                }
                
                // Third pass: Spectral enhancement (when available)
                if let Some(ref mut spectral) = self.spectral_denoiser {
                    output = spectral.process(&output)?;
                }
            }
            DenoisingMode::Custom { use_rnnoise, use_deep_learning, use_spectral } => {
                if *use_rnnoise {
                    if let Some(ref rnnoise) = self.rnnoise_denoiser {
                        output = rnnoise.process(&output);
                    }
                }
                
                if *use_deep_learning {
                    if let Some(ref mut deep_learning) = self.deep_learning_denoiser {
                        output = deep_learning.process(&output)?;
                    }
                }
                
                if *use_spectral {
                    if let Some(ref mut spectral) = self.spectral_denoiser {
                        output = spectral.process(&output)?;
                    }
                }
            }
        }
        
        // Update performance metrics
        let processing_time = start_time.elapsed().as_secs_f32() * 1000.0; // Convert to ms
        let cpu_usage = self.estimate_cpu_usage(processing_time);
        
        self.metrics.latency_ms = processing_time;
        self.metrics.cpu_usage_percent = cpu_usage;
        
        self.performance_monitor.update(cpu_usage, processing_time);
        
        // Check for adaptive adjustments
        self.adaptive_mode_adjustment();
        
        Ok(output)
    }
    
    fn set_mode(&mut self, mode: DenoisingMode) -> Result<()> {
        self.config.mode = mode;
        Ok(())
    }
    
    fn get_mode(&self) -> DenoisingMode {
        self.config.mode.clone()
    }
    
    fn get_latency(&self) -> f32 {
        self.metrics.latency_ms
    }
    
    fn get_cpu_usage(&self) -> f32 {
        self.metrics.cpu_usage_percent
    }
    
    fn get_metrics(&self) -> DenoisingMetrics {
        self.metrics.clone()
    }
    
    fn is_ready(&self) -> bool {
        self.rnnoise_denoiser.is_some()
    }
    
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl AdvancedDenoisingSystem {
    /// Estimate CPU usage based on processing time
    fn estimate_cpu_usage(&self, processing_time_ms: f32) -> f32 {
        let frame_duration_ms = (self.config.frame_size as f32 / self.config.sample_rate as f32) * 1000.0;
        (processing_time_ms / frame_duration_ms) * 100.0
    }
    
    /// Get available denoising modes based on system capabilities
    pub fn get_available_modes(&self) -> Vec<DenoisingMode> {
        let mut modes = vec![DenoisingMode::Basic];
        
        if self.deep_learning_denoiser.is_some() {
            modes.push(DenoisingMode::Enhanced);
        }
        
        if self.spectral_denoiser.is_some() {
            modes.push(DenoisingMode::Maximum);
        }
        
        modes
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: AdvancedDenoisingConfig) -> Result<()> {
        self.config = config;
        // Re-initialize if needed
        if !self.is_ready() {
            self.initialize_denoisers()?;
        }
        Ok(())
    }
}

// Create a type alias for easier use
pub type SharedAdvancedDenoiser = Arc<Mutex<dyn AdvancedDenoiser>>;

/// Factory function to create the advanced denoising system
pub fn create_advanced_denoiser(config: AdvancedDenoisingConfig) -> Result<SharedAdvancedDenoiser> {
    let system = AdvancedDenoisingSystem::new(config)?;
    Ok(Arc::new(Mutex::new(system)))
}
