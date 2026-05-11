//! Application configuration persistence.

#![allow(dead_code)] // Config API for save/load functionality

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
            Self::ShureSM7B => {
                "Dynamic broadcast legend, needs high gain (CloudLifter recommended)"
            }
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
            Self::RodePodMic => -45.0, // Dynamic rejects noise well
            Self::ShureSM7B => -50.0,  // Very noise-rejecting
            Self::RodePodcaster => -45.0,
            Self::RodeNT1 => -55.0, // Very low noise floor
            Self::AT2020 => -50.0,
            Self::ScarlettDefault => -40.0,
            Self::Custom => -40.0,
        }
    }

    /// Recommended compressor threshold in dB
    pub fn compressor_threshold_db(&self) -> f32 {
        match self {
            Self::RodePodMic => -20.0,
            Self::ShureSM7B => -18.0, // Broadcast standard
            Self::RodePodcaster => -20.0,
            Self::RodeNT1 => -24.0, // Wider dynamic range
            Self::AT2020 => -22.0,
            Self::ScarlettDefault => -18.0,
            Self::Custom => -18.0,
        }
    }

    /// Recommended compressor ratio
    pub fn compressor_ratio(&self) -> f32 {
        match self {
            Self::RodePodMic => 3.0,
            Self::ShureSM7B => 4.0, // Broadcast standard 4:1
            Self::RodePodcaster => 3.5,
            Self::RodeNT1 => 2.5, // Gentle for vocals
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

/// GhostWave configuration
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
    /// Theme preset name (TokyoNight, TokyoNightStorm, TokyoNightMoon)
    pub theme: String,
    pub sample_rate: f32,
    pub buffer_size: usize,
    /// GhostWave settings
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

        if let Ok(config_str) = fs::read_to_string(&config_path)
            && let Ok(config) = serde_json::from_str::<AppConfig>(&config_str)
        {
            return config;
        }

        // Return default config if loading fails
        Self {
            channel_volumes: vec![0.8; 4],
            channel_plugins: vec![None; 4],
            channel_muted: vec![false; 4],
            scarlett_gain: 0.5,
            theme: "TokyoNight".to_string(),
            sample_rate: 48000.0,
            buffer_size: 1024,
            ghostwave: GhostWaveConfig {
                profile: "Streaming".to_string(),
                latency_mode: "Balanced".to_string(),
                noise_strength: 0.65,
                enabled: true,
                show_metrics: false,
            },
            ..Self::default()
        }
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

    pub fn update_channel(
        &mut self,
        channel_idx: usize,
        volume: f32,
        muted: bool,
        plugin: Option<usize>,
    ) {
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
        self.channel_volumes
            .get(channel_idx)
            .copied()
            .unwrap_or(0.8)
    }

    pub fn get_channel_muted(&self, channel_idx: usize) -> bool {
        self.channel_muted
            .get(channel_idx)
            .copied()
            .unwrap_or(false)
    }

    pub fn get_channel_plugin(&self, channel_idx: usize) -> Option<usize> {
        self.channel_plugins.get(channel_idx).and_then(|p| *p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_microphone_preset_names() {
        assert_eq!(MicrophonePreset::RodePodMic.name(), "Rode PodMic");
        assert_eq!(MicrophonePreset::ShureSM7B.name(), "Shure SM7B");
        assert_eq!(MicrophonePreset::Custom.name(), "Custom");
    }

    #[test]
    fn test_microphone_preset_gains() {
        assert!(MicrophonePreset::ShureSM7B.recommended_gain_db() > MicrophonePreset::RodePodMic.recommended_gain_db());
        assert!(MicrophonePreset::RodeNT1.recommended_gain_db() < MicrophonePreset::ShureSM7B.recommended_gain_db());
    }

    #[test]
    fn test_microphone_preset_phantom_power() {
        assert!(MicrophonePreset::RodeNT1.needs_phantom_power());
        assert!(MicrophonePreset::AT2020.needs_phantom_power());
        assert!(!MicrophonePreset::RodePodMic.needs_phantom_power());
        assert!(!MicrophonePreset::ShureSM7B.needs_phantom_power());
    }

    #[test]
    fn test_microphone_preset_all() {
        let all = MicrophonePreset::all();
        assert_eq!(all.len(), 7);
    }

    #[test]
    fn test_app_config_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.theme, "");
        assert_eq!(config.scarlett_gain, 0.0);
    }

    #[test]
    fn test_app_config_load_returns_fallback() {
        let config = AppConfig::load();
        assert_eq!(config.channel_volumes.len(), 4);
        assert_eq!(config.channel_volumes[0], 0.8);
        assert_eq!(config.theme, "TokyoNight");
        assert_eq!(config.buffer_size, 1024);
        assert!(config.ghostwave.enabled);
    }

    #[test]
    fn test_app_config_update_channel() {
        let mut config = AppConfig::load();
        config.update_channel(0, 0.5, true, Some(1));
        assert_eq!(config.get_channel_volume(0), 0.5);
        assert!(config.get_channel_muted(0));
        assert_eq!(config.get_channel_plugin(0), Some(1));
    }

    #[test]
    fn test_app_config_channel_expansion() {
        let mut config = AppConfig::load();
        config.update_channel(10, 0.9, false, None);
        assert!(config.channel_volumes.len() > 10);
        assert_eq!(config.get_channel_volume(10), 0.9);
    }

    #[test]
    fn test_app_config_out_of_bounds_defaults() {
        let config = AppConfig::load();
        assert_eq!(config.get_channel_volume(99), 0.8);
        assert!(!config.get_channel_muted(99));
        assert_eq!(config.get_channel_plugin(99), None);
    }

    #[test]
    fn test_ghostwave_config_defaults() {
        let config = GhostWaveConfig::default();
        assert_eq!(config.noise_strength, 0.0); // serde default
        assert!(!config.show_metrics);
    }

    #[test]
    fn test_pipewire_config_defaults() {
        let config = PipeWireConfig::default();
        assert_eq!(config.buffer_size, 512);
        assert_eq!(config.sample_rate, 48000);
        assert!(config.create_virtual_device);
        assert!(config.auto_link_source);
    }

    #[test]
    fn test_config_save_load_roundtrip() {
        let dir = std::env::temp_dir().join("phantomlink_test_roundtrip");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test_config.json");

        let config = AppConfig {
            theme: "TokyoNightStorm".to_string(),
            channel_volumes: vec![0.7, 0.5, 0.3, 1.0],
            channel_muted: vec![false, true, false, false],
            channel_plugins: vec![None, Some(2), None, None],
            scarlett_gain: 0.75,
            buffer_size: 512,
            sample_rate: 48000.0,
            ghostwave: GhostWaveConfig {
                profile: "Streaming".to_string(),
                latency_mode: "Balanced".to_string(),
                noise_strength: 0.8,
                enabled: true,
                show_metrics: true,
            },
            echo_cancellation: true,
            ..AppConfig::default()
        };

        // Save to temp file
        let config_str = serde_json::to_string_pretty(&config).unwrap();
        fs::write(&path, &config_str).unwrap();

        // Load back
        let loaded_str = fs::read_to_string(&path).unwrap();
        let loaded: AppConfig = serde_json::from_str(&loaded_str).unwrap();

        assert_eq!(loaded.theme, "TokyoNightStorm");
        assert_eq!(loaded.channel_volumes, vec![0.7, 0.5, 0.3, 1.0]);
        assert_eq!(loaded.channel_muted, vec![false, true, false, false]);
        assert_eq!(loaded.scarlett_gain, 0.75);
        assert_eq!(loaded.buffer_size, 512);
        assert!(loaded.ghostwave.enabled);
        assert_eq!(loaded.ghostwave.noise_strength, 0.8);
        assert!(loaded.ghostwave.show_metrics);
        assert!(loaded.echo_cancellation);

        // Cleanup
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_config_theme_persistence() {
        let config = AppConfig {
            theme: "TokyoNightMoon".to_string(),
            ..AppConfig::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        let loaded: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.theme, "TokyoNightMoon");
    }

    #[test]
    fn test_config_missing_fields_use_defaults() {
        // Simulate loading a config with missing optional fields
        let json = r#"{
            "vst_plugin_paths": [],
            "channel_volumes": [0.8],
            "channel_plugins": [],
            "channel_muted": [],
            "scarlett_gain": 0.5,
            "scarlett_monitor": false,
            "rnnoise_enabled": false,
            "theme": "TokyoNight",
            "sample_rate": 48000.0,
            "buffer_size": 1024
        }"#;

        let config: AppConfig = serde_json::from_str(json).unwrap();
        // ghostwave and pipewire should use serde defaults
        assert!(!config.ghostwave.show_metrics);
        assert_eq!(config.pipewire.buffer_size, 512);
        assert_eq!(config.pipewire.sample_rate, 48000);
        assert!(!config.echo_cancellation);
    }

    #[test]
    fn test_config_ghostwave_settings_roundtrip() {
        let gw = GhostWaveConfig {
            profile: "XlrStudio".to_string(),
            latency_mode: "LowLatency".to_string(),
            noise_strength: 0.95,
            enabled: false,
            show_metrics: true,
        };

        let json = serde_json::to_string(&gw).unwrap();
        let loaded: GhostWaveConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.profile, "XlrStudio");
        assert_eq!(loaded.latency_mode, "LowLatency");
        assert_eq!(loaded.noise_strength, 0.95);
        assert!(!loaded.enabled);
        assert!(loaded.show_metrics);
    }

    #[test]
    fn test_pipewire_config_roundtrip() {
        let pw = PipeWireConfig {
            preset: "Recording".to_string(),
            buffer_size: 1024,
            sample_rate: 96000,
            create_virtual_device: false,
            auto_link_source: false,
            virtual_device_name: "TestDevice".to_string(),
            preferred_input: "hw:Scarlett".to_string(),
        };

        let json = serde_json::to_string(&pw).unwrap();
        let loaded: PipeWireConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.preset, "Recording");
        assert_eq!(loaded.buffer_size, 1024);
        assert_eq!(loaded.sample_rate, 96000);
        assert!(!loaded.create_virtual_device);
        assert_eq!(loaded.preferred_input, "hw:Scarlett");
    }
}
