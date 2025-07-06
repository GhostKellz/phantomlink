use std::sync::{Arc, Mutex};
use anyhow::Result;

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
        
        // TODO: Initialize deep learning denoiser when available
        // self.deep_learning_denoiser = Some(Box::new(FacebookDenoiser::new()?));
        
        // TODO: Initialize spectral denoiser when available
        // self.spectral_denoiser = Some(Box::new(WienerDenoiser::new()?));
        
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
