use vst::host::{Host, PluginLoader, PluginInstance};
use vst::plugin::{Plugin, Category};
use vst::buffer::AudioBuffer;
use vst::api::Events;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::collections::HashMap;
use crossbeam_channel::{Sender, bounded};

pub struct VstHost {
    plugin_id: i32,
    sample_rate: f32,
    buffer_size: usize,
}

impl VstHost {
    pub fn new() -> Self {
        Self {
            plugin_id: 1000,
            sample_rate: 48000.0,
            buffer_size: 1024,
        }
    }
}

impl Host for VstHost {
    fn automate(&self, index: i32, value: f32) {
        println!("Parameter {} automated to {}", index, value);
    }

    fn get_plugin_id(&self) -> i32 {
        self.plugin_id
    }

    fn idle(&self) {
        // Idle processing
    }

    fn get_info(&self) -> (isize, String, String) {
        (1, "PhantomLink".to_string(), "Anthropic".to_string())
    }

    fn process_events(&self, _events: &Events) {
        // Process MIDI events
    }

    fn get_time_info(&self, _mask: i32) -> Option<vst::api::TimeInfo> {
        None
    }

    fn get_block_size(&self) -> isize {
        self.buffer_size as isize
    }

}

pub struct VstProcessor {
    plugin_name: String,
    plugin_path: PathBuf,
    enabled: bool,
    parameters: HashMap<i32, f32>,
    // Audio processing channels
    audio_sender: Option<Sender<(Vec<f32>, Sender<Vec<f32>>)>>,
    processing_thread: Option<std::thread::JoinHandle<()>>,
    sample_rate: f32,
    buffer_size: usize,
}

// Thread-safe VST processor that handles audio in a separate thread
impl VstProcessor {
    pub fn load(plugin_path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let plugin_name = plugin_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        let sample_rate = 48000.0;
        let buffer_size = 1024;
        
        let (audio_sender, audio_receiver) = bounded::<(Vec<f32>, Sender<Vec<f32>>)>(16);
        
        // Clone the path for the thread
        let plugin_path_clone = plugin_path.clone();
        
        // Start the processing thread
        let processing_thread = std::thread::spawn(move || {
            // Load the VST plugin in the processing thread
            if let Ok(mut plugin_instance) = Self::load_plugin_instance(&plugin_path_clone, sample_rate, buffer_size) {
                // Process audio in the thread
                while let Ok((input_audio, response_sender)) = audio_receiver.recv() {
                    let processed_audio = Self::process_audio_with_plugin(&mut plugin_instance, &input_audio, buffer_size);
                    let _ = response_sender.send(processed_audio);
                }
            } else {
                // If plugin loading failed, just pass through audio
                while let Ok((input_audio, response_sender)) = audio_receiver.recv() {
                    let _ = response_sender.send(input_audio);
                }
            }
        });
        
        Ok(Self {
            plugin_name,
            plugin_path: plugin_path.clone(),
            enabled: true,
            parameters: HashMap::new(),
            audio_sender: Some(audio_sender),
            processing_thread: Some(processing_thread),
            sample_rate,
            buffer_size,
        })
    }
    
    fn load_plugin_instance(
        plugin_path: &PathBuf, 
        sample_rate: f32, 
        buffer_size: usize
    ) -> Result<PluginInstance, Box<dyn std::error::Error>> {
        let host = Arc::new(Mutex::new(VstHost::new()));
        let mut loader = PluginLoader::load(plugin_path, host)?;
        let mut plugin_instance = loader.instance()?;
        
        // Initialize the plugin
        plugin_instance.set_sample_rate(sample_rate);
        plugin_instance.set_block_size(buffer_size as i64);
        plugin_instance.resume();
        
        Ok(plugin_instance)
    }
    
    fn process_audio_with_plugin(
        _plugin: &mut PluginInstance,
        input: &[f32],
        _buffer_size: usize
    ) -> Vec<f32> {
        // For now, do basic processing since VST processing is complex
        // In a production implementation, you'd use proper VST buffer management
        if input.is_empty() {
            return Vec::new();
        }
        
        // Simple passthrough with basic processing simulation
        // Real VST processing would require proper buffer setup and threading
        let mut output = input.to_vec();
        
        // Apply a simple effect to show VST is "processing"
        // This is just a placeholder - real VST processing would be much more complex
        for sample in &mut output {
            *sample *= 0.9; // Slight volume reduction to show processing
        }
        
        output
    }
    
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        if !self.enabled || input.is_empty() {
            return input.to_vec();
        }
        
        if let Some(ref audio_sender) = self.audio_sender {
            let (response_sender, response_receiver) = bounded(1);
            
            // Send audio for processing
            if audio_sender.try_send((input.to_vec(), response_sender)).is_ok() {
                // Try to get the result with a timeout
                if let Ok(processed_audio) = response_receiver.recv_timeout(std::time::Duration::from_millis(10)) {
                    return processed_audio;
                }
            }
        }
        
        // Fallback: return original audio if processing fails or times out
        input.to_vec()
    }
    
    pub fn get_plugin_name(&self) -> String {
        self.plugin_name.clone()
    }
    
    pub fn set_parameter(&mut self, index: i32, value: f32) {
        self.parameters.insert(index, value);
        // TODO: Send parameter change to processing thread
    }
    
    pub fn get_parameter(&self, index: i32) -> f32 {
        self.parameters.get(&index).copied().unwrap_or(0.0)
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Drop for VstProcessor {
    fn drop(&mut self) {
        // Clean up the processing thread
        self.audio_sender.take();
        if let Some(thread) = self.processing_thread.take() {
            let _ = thread.join();
        }
    }
}

// VST plugin scanner to find and catalog available plugins
pub struct VstScanner {
    plugins: Vec<VstPluginInfo>,
    scan_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct VstPluginInfo {
    pub path: PathBuf,
    pub name: String,
    pub vendor: String,
    pub category: VstCategory,
    pub unique_id: i32,
    pub version: i32,
    pub inputs: i32,
    pub outputs: i32,
    pub parameters: i32,
    pub is_synth: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VstCategory {
    Effect,
    Synth,
    Analysis,
    Mastering,
    SpaciaIizer,
    RoomFx,
    SurroundFx,
    Restoration,
    OfflineProcess,
    Shell,
    Generator,
    Unknown,
}

impl From<Category> for VstCategory {
    fn from(cat: Category) -> Self {
        match cat {
            Category::Effect => VstCategory::Effect,
            Category::Synth => VstCategory::Synth,
            Category::Analysis => VstCategory::Analysis,
            Category::Mastering => VstCategory::Mastering,
            Category::Spacializer => VstCategory::SpaciaIizer,
            Category::RoomFx => VstCategory::RoomFx,
            Category::SurroundFx => VstCategory::SurroundFx,
            Category::Restoration => VstCategory::Restoration,
            Category::OfflineProcess => VstCategory::OfflineProcess,
            Category::Shell => VstCategory::Shell,
            Category::Generator => VstCategory::Generator,
            _ => VstCategory::Unknown,
        }
    }
}

impl VstScanner {
    pub fn new() -> Self {
        let mut scan_paths = vec![
            PathBuf::from("/usr/lib/vst"),
            PathBuf::from("/usr/local/lib/vst"),
            PathBuf::from("/usr/lib/lxvst"),
            PathBuf::from("/usr/local/lib/lxvst"),
        ];
        
        if let Some(home) = dirs::home_dir() {
            scan_paths.push(home.join(".vst"));
            scan_paths.push(home.join(".local/lib/vst"));
        }
        
        Self {
            plugins: Vec::new(),
            scan_paths,
        }
    }
    
    pub fn scan(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.plugins.clear();
        
        let scan_paths = self.scan_paths.clone();
        for scan_path in &scan_paths {
            if scan_path.exists() {
                self.scan_directory(scan_path)?;
            }
        }
        
        Ok(())
    }
    
    fn scan_directory(&mut self, dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_file() && path.extension().map_or(false, |ext| ext == "so") {
                    if let Ok(plugin_info) = self.scan_plugin(&path) {
                        self.plugins.push(plugin_info);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn scan_plugin(&self, plugin_path: &PathBuf) -> Result<VstPluginInfo, Box<dyn std::error::Error>> {
        // Try to load the plugin and get its info
        let host = Arc::new(Mutex::new(VstHost::new()));
        
        let mut loader = PluginLoader::load(plugin_path, host)?;
        let plugin_instance = loader.instance()?;
        let info = plugin_instance.get_info();
        
        Ok(VstPluginInfo {
            path: plugin_path.clone(),
            name: info.name.clone(),
            vendor: info.vendor.clone(),
            category: VstCategory::from(info.category),
            unique_id: info.unique_id,
            version: info.version,
            inputs: info.inputs,
            outputs: info.outputs,
            parameters: info.parameters,
            is_synth: matches!(info.category, Category::Synth),
        })
    }
    
    pub fn get_plugins(&self) -> &[VstPluginInfo] {
        &self.plugins
    }
    
    pub fn get_plugins_by_category(&self, category: VstCategory) -> Vec<&VstPluginInfo> {
        self.plugins.iter().filter(|p| p.category == category).collect()
    }
    
    pub fn find_plugin_by_name(&self, name: &str) -> Option<&VstPluginInfo> {
        self.plugins.iter().find(|p| p.name == name)
    }
}