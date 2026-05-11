use std::fs;
use std::path::PathBuf;

// Re-export the VST types from vst_host module
pub use crate::vst_host::{VstPluginInfo, VstScanner};

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
                if path.is_file() && path.extension().is_some_and(|ext| ext == "so") {
                    plugins.push(path);
                }
            }
        }
    }
    plugins
}

pub fn scan_vst_plugins() -> Result<Vec<VstPluginInfo>, Box<dyn std::error::Error>> {
    let mut scanner = VstScanner::new();
    scanner.scan()?;
    Ok(scanner.get_plugins().to_vec())
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
        Self { routes: Vec::new() }
    }

    #[allow(dead_code)] // Public API for route configuration
    pub fn add_route(&mut self, from: String, to: String, gain: f32) {
        self.routes.push(AudioRoute {
            from,
            to,
            gain,
            enabled: true,
        });
    }

    #[allow(dead_code)] // Public API for route configuration
    pub fn remove_route(&mut self, from: &str, to: &str) {
        self.routes.retain(|r| !(r.from == from && r.to == to));
    }

    #[allow(dead_code)] // Public API for route configuration
    pub fn set_route_enabled(&mut self, from: &str, to: &str, enabled: bool) {
        if let Some(route) = self
            .routes
            .iter_mut()
            .find(|r| r.from == from && r.to == to)
        {
            route.enabled = enabled;
        }
    }

    #[allow(dead_code)] // Public API for route configuration
    pub fn set_route_gain(&mut self, from: &str, to: &str, gain: f32) {
        if let Some(route) = self
            .routes
            .iter_mut()
            .find(|r| r.from == from && r.to == to)
        {
            route.gain = gain;
        }
    }

    pub fn get_routes(&self) -> &[AudioRoute] {
        &self.routes
    }

    /// Apply routing: map named input channels to named output channels with gain.
    ///
    /// `channel_map` maps channel name (e.g. "ch0") to an index into `input_channels`.
    /// `output_names` lists the destination channel names for the output array.
    /// Returns output channels with routed+summed audio.
    pub fn apply_routing(
        &self,
        input_channels: &[Vec<f32>],
        channel_map: &std::collections::HashMap<String, usize>,
        output_names: &[String],
    ) -> Vec<Vec<f32>> {
        if input_channels.is_empty() {
            return Vec::new();
        }

        let frame_count = input_channels[0].len();
        let mut output_channels = vec![vec![0.0f32; frame_count]; output_names.len()];

        // Build output name → index lookup
        let output_map: std::collections::HashMap<&str, usize> = output_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        for route in &self.routes {
            if !route.enabled {
                continue;
            }

            let src_idx = match channel_map.get(&route.from) {
                Some(&idx) if idx < input_channels.len() => idx,
                _ => continue,
            };

            let dst_idx = match output_map.get(route.to.as_str()) {
                Some(&idx) => idx,
                None => continue,
            };

            let src = &input_channels[src_idx];
            let dst = &mut output_channels[dst_idx];
            for (out_sample, &in_sample) in dst.iter_mut().zip(src.iter()) {
                *out_sample += in_sample * route.gain;
            }
        }

        output_channels
    }
}
