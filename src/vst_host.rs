//! VST2 Plugin hosting for audio effects and instruments.
//!
//! Provides full VST2 plugin hosting with:
//! - Plugin loading and initialization
//! - Real-time parameter control via message passing
//! - Thread-safe audio processing
//! - Plugin scanning and cataloging

#![allow(dead_code)] // Complete VST hosting API for plugin management

use vst::host::{Host, PluginLoader, PluginInstance};
use vst::plugin::{Plugin, Category, PluginParameters};
use vst::api::Events;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::collections::HashMap;
use crossbeam_channel::{Sender, Receiver, bounded, TrySendError};

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

/// Message types for VST processor thread communication
#[derive(Debug)]
enum VstMessage {
    /// Process audio buffer and return result
    ProcessAudio {
        input: Vec<f32>,
        response: Sender<Vec<f32>>,
    },
    /// Set a parameter value
    SetParameter {
        index: i32,
        value: f32,
    },
    /// Get parameter info (name, display value)
    GetParameterInfo {
        index: i32,
        response: Sender<Option<ParameterInfo>>,
    },
    /// Request all parameter values
    GetAllParameters {
        response: Sender<HashMap<i32, f32>>,
    },
    /// Set enabled state
    SetEnabled {
        enabled: bool,
    },
    /// Shutdown the processor
    Shutdown,
}

/// Parameter information returned from VST plugin
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub index: i32,
    pub name: String,
    pub label: String,
    pub value: f32,
    pub display: String,
}

pub struct VstProcessor {
    plugin_name: String,
    plugin_path: PathBuf,
    enabled: bool,
    parameters: HashMap<i32, f32>,
    parameter_count: i32,
    // Message channel to processing thread
    message_sender: Option<Sender<VstMessage>>,
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

        // Create message channel with reasonable buffer
        let (message_sender, message_receiver) = bounded::<VstMessage>(64);

        // Clone the path for the thread
        let plugin_path_clone = plugin_path.clone();

        // Get parameter count by loading plugin temporarily
        let parameter_count = Self::query_parameter_count(&plugin_path_clone)?;

        // Start the processing thread
        let processing_thread = std::thread::spawn(move || {
            Self::processor_thread_main(plugin_path_clone, sample_rate, buffer_size, message_receiver);
        });

        // Initialize default parameter values
        let mut parameters = HashMap::new();
        for i in 0..parameter_count {
            parameters.insert(i, 0.5); // Default to 50%
        }

        Ok(Self {
            plugin_name,
            plugin_path: plugin_path.clone(),
            enabled: true,
            parameters,
            parameter_count,
            message_sender: Some(message_sender),
            processing_thread: Some(processing_thread),
            sample_rate,
            buffer_size,
        })
    }

    /// Get parameter count from plugin without keeping it loaded
    fn query_parameter_count(plugin_path: &PathBuf) -> Result<i32, Box<dyn std::error::Error>> {
        let host = Arc::new(Mutex::new(VstHost::new()));
        let mut loader = PluginLoader::load(plugin_path, host)?;
        let plugin_instance = loader.instance()?;
        let info = plugin_instance.get_info();
        Ok(info.parameters)
    }

    /// Main loop for the processor thread
    fn processor_thread_main(
        plugin_path: PathBuf,
        sample_rate: f32,
        buffer_size: usize,
        message_receiver: Receiver<VstMessage>,
    ) {
        // Load the VST plugin in the processing thread
        let plugin_result = Self::load_plugin_instance(&plugin_path, sample_rate, buffer_size);

        match plugin_result {
            Ok(mut plugin_instance) => {
                let mut enabled = true;
                let info = plugin_instance.get_info();
                let param_count = info.parameters;

                // Get parameter object for accessing/setting parameters
                let params = plugin_instance.get_parameter_object();

                // Process messages in the thread
                while let Ok(message) = message_receiver.recv() {
                    match message {
                        VstMessage::ProcessAudio { input, response } => {
                            let output = if enabled {
                                Self::process_audio_with_plugin(&mut plugin_instance, &input, buffer_size)
                            } else {
                                input
                            };
                            let _ = response.send(output);
                        }
                        VstMessage::SetParameter { index, value } => {
                            // Clamp value to 0.0-1.0 range (VST standard)
                            let clamped = value.clamp(0.0, 1.0);
                            params.set_parameter(index, clamped);
                        }
                        VstMessage::GetParameterInfo { index, response } => {
                            if index >= 0 && index < param_count {
                                let info = ParameterInfo {
                                    index,
                                    name: params.get_parameter_name(index),
                                    label: params.get_parameter_label(index),
                                    value: params.get_parameter(index),
                                    display: params.get_parameter_text(index),
                                };
                                let _ = response.send(Some(info));
                            } else {
                                let _ = response.send(None);
                            }
                        }
                        VstMessage::GetAllParameters { response } => {
                            let mut param_values = HashMap::new();
                            for i in 0..param_count {
                                param_values.insert(i, params.get_parameter(i));
                            }
                            let _ = response.send(param_values);
                        }
                        VstMessage::SetEnabled { enabled: e } => {
                            enabled = e;
                        }
                        VstMessage::Shutdown => {
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to load VST plugin: {}", e);
                // If plugin loading failed, just handle audio passthrough
                while let Ok(message) = message_receiver.recv() {
                    match message {
                        VstMessage::ProcessAudio { input, response } => {
                            let _ = response.send(input);
                        }
                        VstMessage::Shutdown => break,
                        _ => {}
                    }
                }
            }
        }
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
        plugin: &mut PluginInstance,
        input: &[f32],
        buffer_size: usize
    ) -> Vec<f32> {
        if input.is_empty() {
            return Vec::new();
        }

        // Get plugin channel configuration
        let info = plugin.get_info();
        let num_inputs = info.inputs.max(1) as usize;
        let num_outputs = info.outputs.max(1) as usize;

        // Determine actual frame count
        let frame_count = input.len() / num_inputs.max(1);

        // Create input buffers (deinterleaved)
        let mut input_buffers: Vec<Vec<f32>> = (0..num_inputs)
            .map(|ch| {
                (0..frame_count)
                    .map(|frame| {
                        let idx = frame * num_inputs + ch;
                        if idx < input.len() {
                            input[idx]
                        } else {
                            0.0
                        }
                    })
                    .collect()
            })
            .collect();

        // Create output buffers
        let mut output_buffers: Vec<Vec<f32>> = (0..num_outputs)
            .map(|_| vec![0.0; frame_count])
            .collect();

        // Convert to raw pointers for VST AudioBuffer API
        let input_ptrs: Vec<*const f32> = input_buffers.iter().map(|b| b.as_ptr()).collect();
        let mut output_ptrs: Vec<*mut f32> = output_buffers.iter_mut().map(|b| b.as_mut_ptr()).collect();

        // Create AudioBuffer from raw pointers and process
        // Safety: pointers are valid for the duration of this function call
        unsafe {
            let mut audio_buffer = vst::buffer::AudioBuffer::from_raw(
                num_inputs,
                num_outputs,
                input_ptrs.as_ptr(),
                output_ptrs.as_mut_ptr(),
                frame_count,
            );
            plugin.process(&mut audio_buffer);
        }

        // Interleave output back to single buffer
        let mut output = Vec::with_capacity(frame_count * num_outputs);
        for frame in 0..frame_count {
            for ch in 0..num_outputs {
                output.push(output_buffers[ch][frame]);
            }
        }

        output
    }

    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        if !self.enabled || input.is_empty() {
            return input.to_vec();
        }

        if let Some(ref sender) = self.message_sender {
            let (response_sender, response_receiver) = bounded(1);

            let message = VstMessage::ProcessAudio {
                input: input.to_vec(),
                response: response_sender,
            };

            // Send audio for processing
            if sender.try_send(message).is_ok() {
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

    /// Set a parameter value (0.0 - 1.0 range)
    /// This properly sends the parameter change to the VST processing thread
    pub fn set_parameter(&mut self, index: i32, value: f32) {
        let clamped = value.clamp(0.0, 1.0);
        self.parameters.insert(index, clamped);

        // Send parameter change to processing thread
        if let Some(ref sender) = self.message_sender {
            let _ = sender.try_send(VstMessage::SetParameter {
                index,
                value: clamped,
            });
        }
    }

    /// Get cached parameter value
    pub fn get_parameter(&self, index: i32) -> f32 {
        self.parameters.get(&index).copied().unwrap_or(0.0)
    }

    /// Get parameter info from the actual VST plugin (blocking)
    pub fn get_parameter_info(&self, index: i32) -> Option<ParameterInfo> {
        if let Some(ref sender) = self.message_sender {
            let (response_sender, response_receiver) = bounded(1);

            let message = VstMessage::GetParameterInfo {
                index,
                response: response_sender,
            };

            if sender.try_send(message).is_ok() {
                if let Ok(info) = response_receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                    return info;
                }
            }
        }
        None
    }

    /// Get all parameter values from the VST plugin (blocking)
    pub fn get_all_parameters(&self) -> HashMap<i32, f32> {
        if let Some(ref sender) = self.message_sender {
            let (response_sender, response_receiver) = bounded(1);

            let message = VstMessage::GetAllParameters {
                response: response_sender,
            };

            if sender.try_send(message).is_ok() {
                if let Ok(params) = response_receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                    return params;
                }
            }
        }
        self.parameters.clone()
    }

    /// Sync local parameter cache with actual VST values
    pub fn sync_parameters(&mut self) {
        let params = self.get_all_parameters();
        self.parameters = params;
    }

    /// Get number of parameters
    pub fn get_parameter_count(&self) -> i32 {
        self.parameter_count
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        // Notify processing thread
        if let Some(ref sender) = self.message_sender {
            let _ = sender.try_send(VstMessage::SetEnabled { enabled });
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the sample rate
    pub fn get_sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Get the buffer size
    pub fn get_buffer_size(&self) -> usize {
        self.buffer_size
    }
}

impl Drop for VstProcessor {
    fn drop(&mut self) {
        // Send shutdown message
        if let Some(ref sender) = self.message_sender {
            let _ = sender.send(VstMessage::Shutdown);
        }

        // Clean up the processing thread
        self.message_sender.take();
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