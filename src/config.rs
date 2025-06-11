use std::path::PathBuf;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub vst_plugin_paths: Vec<PathBuf>,
    pub channel_volumes: Vec<f32>,
    pub channel_plugins: Vec<Option<usize>>,
    pub scarlett_gain: f32,
    pub scarlett_monitor: bool,
    pub theme: String,
}

impl AppConfig {
    pub fn load() -> Self {
        // Load config from file (stub)
        Self::default()
    }
    pub fn save(&self) {
        // Save config to file (stub)
    }
}