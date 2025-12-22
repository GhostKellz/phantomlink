//! Application configuration persistence.

#![allow(dead_code)] // Config API for save/load functionality

use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};

const CONFIG_FILE_NAME: &str = "phantomlink_config.json";

/// Professional microphone presets with optimized settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MicrophonePreset {
    /// Rode PodMic - Dynamic, needs moderate gain, warm sound
    #[default]
    RodePodMic,
    /// Shure SM7B - Dynamic, needs high gain (60dB+), broadcast quality
    ShureSM7B,
    /// Rode Podcaster - Dynamic USB/XLR, balanced output
    RodePodcaster,
    /// Rode NT1 - Condenser, very low noise, high sensitivity
    RodeNT1,
    /// Audio-Technica AT2020 - Condenser, versatile
    AT2020,
    /// Scarlett Solo Gen 4 default (generic dynamic)
    ScarlettDefault,
    /// Custom user settings
    Custom,
}

impl MicrophonePreset {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::RodePodMic => "Rode PodMic",
            Self::ShureSM7B => "Shure SM7B",
            Self::RodePodcaster => "Rode Podcaster",
            Self::RodeNT1 => "Rode NT1",
            Self::AT2020 => "AT2020",
            Self::ScarlettDefault => "Generic",
            Self::Custom => "Custom",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::RodePodMic => "Dynamic broadcast mic, warm tone, moderate gain",
            Self::ShureSM7B => "Dynamic broadcast legend, needs high gain (CloudLifter recommended)",
            Self::RodePodcaster => "Dynamic USB/XLR hybrid, balanced output",
            Self::RodeNT1 => "Ultra-low noise condenser, very sensitive",
            Self::AT2020 => "Versatile condenser, good all-rounder",
            Self::ScarlettDefault => "Default settings for generic dynamic mics",
            Self::Custom => "Custom user-defined settings",
        }
    }

    /// Recommended preamp gain in dB (for Scarlett Solo)
    pub fn recommended_gain_db(&self) -> f32 {
        match self {
            Self::RodePodMic => 45.0,      // Moderate gain, efficient dynamic
            Self::ShureSM7B => 60.0,       // Needs lots of gain, low output
            Self::RodePodcaster => 35.0,   // Higher output dynamic
            Self::RodeNT1 => 25.0,         // Very sensitive condenser
            Self::AT2020 => 30.0,          // Moderate sensitivity condenser
            Self::ScarlettDefault => 40.0, // Middle ground
            Self::Custom => 40.0,
        }
    }

    /// Recommended noise gate threshold in dB
    pub fn gate_threshold_db(&self) -> f32 {
        match self {
            Self::RodePodMic => -45.0,      // Dynamic rejects noise well
            Self::ShureSM7B => -50.0,       // Very noise-rejecting
            Self::RodePodcaster => -45.0,
            Self::RodeNT1 => -55.0,         // Very low noise floor
            Self::AT2020 => -50.0,
            Self::ScarlettDefault => -40.0,
            Self::Custom => -40.0,
        }
    }

    /// Recommended compressor threshold in dB
    pub fn compressor_threshold_db(&self) -> f32 {
        match self {
            Self::RodePodMic => -20.0,
            Self::ShureSM7B => -18.0,       // Broadcast standard
            Self::RodePodcaster => -20.0,
            Self::RodeNT1 => -24.0,         // Wider dynamic range
            Self::AT2020 => -22.0,
            Self::ScarlettDefault => -18.0,
            Self::Custom => -18.0,
        }
    }

    /// Recommended compressor ratio
    pub fn compressor_ratio(&self) -> f32 {
        match self {
            Self::RodePodMic => 3.0,
            Self::ShureSM7B => 4.0,         // Broadcast standard 4:1
            Self::RodePodcaster => 3.5,
            Self::RodeNT1 => 2.5,           // Gentle for vocals
            Self::AT2020 => 3.0,
            Self::ScarlettDefault => 4.0,
            Self::Custom => 4.0,
        }
    }

    /// Whether phantom power is typically needed (condensers)
    pub fn needs_phantom_power(&self) -> bool {
        matches!(self, Self::RodeNT1 | Self::AT2020)
    }

    /// Get all available presets
    pub fn all() -> &'static [MicrophonePreset] {
        &[
            Self::RodePodMic,
            Self::ShureSM7B,
            Self::RodePodcaster,
            Self::RodeNT1,
            Self::AT2020,
            Self::ScarlettDefault,
            Self::Custom,
        ]
    }
}

/// GhostWave v0.2.0 configuration
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct GhostWaveConfig {
    /// Processing profile (XlrStudio, Streaming, Balanced, Music)
    #[serde(default)]
    pub profile: String,
    /// Latency mode (LowLatency, Balanced, HighQuality)
    #[serde(default)]
    pub latency_mode: String,
    /// Noise suppression strength (0.0 - 1.0)
    #[serde(default = "default_noise_strength")]
    pub noise_strength: f32,
    /// AI denoising enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Show metrics panel
    #[serde(default)]
    pub show_metrics: bool,
}

/// PipeWire audio configuration
#[derive(Serialize, Deserialize, Clone)]
pub struct PipeWireConfig {
    /// Audio preset (Gaming, Streaming, Recording, Rtx50)
    #[serde(default = "default_pipewire_preset")]
    pub preset: String,
    /// Buffer size in samples
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    /// Sample rate
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
    /// Create virtual device for apps
    #[serde(default = "default_true")]
    pub create_virtual_device: bool,
    /// Auto-link GhostWave to default source
    #[serde(default = "default_true")]
    pub auto_link_source: bool,
    /// Virtual device node name
    #[serde(default = "default_virtual_device_name")]
    pub virtual_device_name: String,
    /// Preferred input device name (for auto-selection)
    #[serde(default)]
    pub preferred_input: String,
}

impl Default for PipeWireConfig {
    fn default() -> Self {
        Self {
            preset: default_pipewire_preset(),
            buffer_size: default_buffer_size(),
            sample_rate: default_sample_rate(),
            create_virtual_device: true,
            auto_link_source: true,
            virtual_device_name: default_virtual_device_name(),
            preferred_input: String::new(),
        }
    }
}

fn default_pipewire_preset() -> String {
    "Streaming".to_string()
}

fn default_buffer_size() -> usize {
    512
}

fn default_sample_rate() -> u32 {
    48000
}

fn default_virtual_device_name() -> String {
    "PhantomLink Clean".to_string()
}

fn default_noise_strength() -> f32 {
    0.65
}

fn default_true() -> bool {
    true
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub vst_plugin_paths: Vec<PathBuf>,
    pub channel_volumes: Vec<f32>,
    pub channel_plugins: Vec<Option<usize>>,
    pub channel_muted: Vec<bool>,
    pub scarlett_gain: f32,
    pub scarlett_monitor: bool,
    pub rnnoise_enabled: bool,
    /// Theme preset name (TokyoNight, Dracula, Nord, ScarlettSolo, etc.)
    pub theme: String,
    pub sample_rate: f32,
    pub buffer_size: usize,
    /// GhostWave v0.2.0 settings
    #[serde(default)]
    pub ghostwave: GhostWaveConfig,
    /// PipeWire audio settings
    #[serde(default)]
    pub pipewire: PipeWireConfig,
    /// Echo cancellation enabled
    #[serde(default)]
    pub echo_cancellation: bool,
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        
        if let Ok(config_str) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&config_str) {
                return config;
            }
        }
        
        // Return default config if loading fails
        let mut default_config = Self::default();
        default_config.channel_volumes = vec![0.8; 4]; // 4 channels with default volume
        default_config.channel_plugins = vec![None; 4];
        default_config.channel_muted = vec![false; 4];
        default_config.scarlett_gain = 0.5;
        default_config.theme = "TokyoNight".to_string();
        default_config.sample_rate = 48000.0;
        default_config.buffer_size = 1024;
        default_config.ghostwave = GhostWaveConfig {
            profile: "Streaming".to_string(),
            latency_mode: "Balanced".to_string(),
            noise_strength: 0.65,
            enabled: true,
            show_metrics: false,
        };

        default_config
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let config_str = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, config_str)?;
        
        Ok(())
    }
    
    fn get_config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("phantomlink").join(CONFIG_FILE_NAME)
        } else {
            // Fallback to home directory
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".phantomlink")
                .join(CONFIG_FILE_NAME)
        }
    }
    
    pub fn update_channel(&mut self, channel_idx: usize, volume: f32, muted: bool, plugin: Option<usize>) {
        // Ensure vectors are large enough
        while self.channel_volumes.len() <= channel_idx {
            self.channel_volumes.push(0.8);
            self.channel_plugins.push(None);
            self.channel_muted.push(false);
        }
        
        self.channel_volumes[channel_idx] = volume;
        self.channel_muted[channel_idx] = muted;
        self.channel_plugins[channel_idx] = plugin;
    }
    
    pub fn get_channel_volume(&self, channel_idx: usize) -> f32 {
        self.channel_volumes.get(channel_idx).copied().unwrap_or(0.8)
    }
    
    pub fn get_channel_muted(&self, channel_idx: usize) -> bool {
        self.channel_muted.get(channel_idx).copied().unwrap_or(false)
    }
    
    pub fn get_channel_plugin(&self, channel_idx: usize) -> Option<usize> {
        self.channel_plugins.get(channel_idx).and_then(|p| *p)
    }
}