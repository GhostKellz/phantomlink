use std::fs;
use std::path::PathBuf;

// Re-export the VST types from vst_host module
pub use crate::vst_host::{VstProcessor, VstScanner, VstPluginInfo};

pub fn find_vst_plugins() -> Vec<PathBuf> {
    let dirs = vec![
        dirs::home_dir().map(|h| h.join(".vst")),
        Some(PathBuf::from("/usr/lib/vst")),
        Some(PathBuf::from("/usr/local/lib/vst")),
        Some(PathBuf::from("/usr/lib/lxvst")),
        Some(PathBuf::from("/usr/local/lib/lxvst")),
    ];
    let mut plugins = Vec::new();
    for dir in dirs.into_iter().flatten() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "so") {
                    plugins.push(path);
                }
            }
        }
    }
    plugins
}

// Audio routing configuration
#[derive(Debug, Clone)]
pub struct AudioRoute {
    pub from: String,
    pub to: String,
    pub gain: f32,
    pub enabled: bool,
}

pub struct AudioRouter {
    routes: Vec<AudioRoute>,
}

impl AudioRouter {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
        }
    }
    
    pub fn add_route(&mut self, from: String, to: String, gain: f32) {
        self.routes.push(AudioRoute {
            from,
            to,
            gain,
            enabled: true,
        });
    }
    
    pub fn remove_route(&mut self, from: &str, to: &str) {
        self.routes.retain(|r| !(r.from == from && r.to == to));
    }
    
    pub fn set_route_enabled(&mut self, from: &str, to: &str, enabled: bool) {
        if let Some(route) = self.routes.iter_mut().find(|r| r.from == from && r.to == to) {
            route.enabled = enabled;
        }
    }
    
    pub fn set_route_gain(&mut self, from: &str, to: &str, gain: f32) {
        if let Some(route) = self.routes.iter_mut().find(|r| r.from == from && r.to == to) {
            route.gain = gain;
        }
    }
    
    pub fn get_routes(&self) -> &[AudioRoute] {
        &self.routes
    }
    
    pub fn apply_routing(&self, input_channels: &[Vec<f32>]) -> Vec<Vec<f32>> {
        let mut output_channels = vec![vec![0.0f32; input_channels[0].len()]; input_channels.len()];
        
        for route in &self.routes {
            if !route.enabled {
                continue;
            }
            
            // Simple routing logic - this would be more complex in a real implementation
            // For now, just apply gain to the appropriate channels
            for (i, input_channel) in input_channels.iter().enumerate() {
                for (j, sample) in input_channel.iter().enumerate() {
                    if j < output_channels[i].len() {
                        output_channels[i][j] += sample * route.gain;
                    }
                }
            }
        }
        
        output_channels
    }
}