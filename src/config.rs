use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};

const CONFIG_FILE_NAME: &str = "phantomlink_config.json";

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub vst_plugin_paths: Vec<PathBuf>,
    pub channel_volumes: Vec<f32>,
    pub channel_plugins: Vec<Option<usize>>,
    pub channel_muted: Vec<bool>,
    pub scarlett_gain: f32,
    pub scarlett_monitor: bool,
    pub rnnoise_enabled: bool,
    pub theme: String,
    pub sample_rate: f32,
    pub buffer_size: usize,
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
        default_config.theme = "dark".to_string();
        default_config.sample_rate = 48000.0;
        default_config.buffer_size = 1024;
        
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