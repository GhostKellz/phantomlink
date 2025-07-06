use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::rnnoise::Rnnoise;
use crate::vst_host::VstProcessor;
use crate::gui::visualizer::{SpectrumAnalyzer, VUMeter};
use crate::advanced_denoising::{
    AdvancedDenoisingSystem, AdvancedDenoisingConfig, DenoisingMode, 
    AdvancedDenoiser, SharedAdvancedDenoiser, create_advanced_denoiser
};
use crossbeam_channel::{Receiver, Sender};
use std::time::Instant;
use anyhow::Result;

const BUFFER_SIZE: usize = 1024;
const CHANNEL_COUNT: usize = 4;

pub struct ChannelProcessor {
    pub volume: f32,
    pub muted: bool,
    pub vst_processor: Option<VstProcessor>,
    pub gain: f32,
    pub pan: f32,
    pub solo: bool,
    buffer: Vec<f32>,
    vu_meter: VUMeter,
    last_levels: [f32; 2], // Store last peak/rms levels
}

impl ChannelProcessor {
    pub fn new() -> Self {
        Self {
            volume: 0.8,
            muted: false,
            vst_processor: None,
            gain: 0.0,
            pan: 0.0,
            solo: false,
            buffer: vec![0.0; BUFFER_SIZE],
            vu_meter: VUMeter::new(128),
            last_levels: [0.0, 0.0],
        }
    }

    pub fn process(&mut self, input: &[f32], rnnoise: &Rnnoise, dt: f32) -> (Vec<f32>, [f32; 2]) {
        if self.muted {
            return (vec![0.0; input.len()], [0.0, 0.0]);
        }

        let mut output = input.to_vec();
        
        // Apply gain
        let gain_linear = if self.gain >= 0.0 {
            1.0 + self.gain / 20.0
        } else {
            10.0_f32.powf(self.gain / 20.0)
        };
        
        for sample in &mut output {
            *sample *= gain_linear;
        }
        
        // Apply noise reduction if enabled (fallback to legacy RNNoise)
        if rnnoise.is_enabled() {
            output = rnnoise.process(&output);
        }
        
        // Apply VST processing
        if let Some(ref mut vst) = self.vst_processor {
            output = vst.process(&output);
        }
        
        // Apply volume
        for sample in &mut output {
            *sample *= self.volume;
        }
        
        // Apply panning (assuming stereo output)
        let mut stereo_output = Vec::with_capacity(output.len() * 2);
        for &sample in &output {
            let left_gain = if self.pan <= 0.0 { 1.0 } else { 1.0 - self.pan };
            let right_gain = if self.pan >= 0.0 { 1.0 } else { 1.0 + self.pan };
            
            stereo_output.push(sample * left_gain);
            stereo_output.push(sample * right_gain);
        }
        
        // Update VU meter and get levels
        let (peak, rms) = self.vu_meter.process(&stereo_output, dt);
        let levels = [peak, rms];
        
        // Store levels for GUI access
        self.last_levels = levels;
        
        (stereo_output, levels)
    }
    
    // New method for processing with advanced denoiser
    pub fn process_advanced(&mut self, input: &[f32], advanced_denoiser: Option<&SharedAdvancedDenoiser>, dt: f32) -> (Vec<f32>, [f32; 2]) {
        if self.muted {
            return (vec![0.0; input.len()], [0.0, 0.0]);
        }

        let mut output = input.to_vec();
        
        // Apply gain
        let gain_linear = if self.gain >= 0.0 {
            1.0 + self.gain / 20.0
        } else {
            10.0_f32.powf(self.gain / 20.0)
        };
        
        for sample in &mut output {
            *sample *= gain_linear;
        }
        
        // Apply advanced noise reduction if available
        if let Some(denoiser) = advanced_denoiser {
            if let Ok(mut d) = denoiser.lock() {
                if d.is_enabled() {
                    if let Ok(processed) = d.process_frame(&output) {
                        output = processed;
                    }
                }
            }
        }
        
        // Apply VST processing
        if let Some(ref mut vst) = self.vst_processor {
            output = vst.process(&output);
        }
        
        // Apply volume
        for sample in &mut output {
            *sample *= self.volume;
        }
        
        // Apply panning (assuming stereo output)
        let mut stereo_output = Vec::with_capacity(output.len() * 2);
        for &sample in &output {
            let left_gain = if self.pan <= 0.0 { 1.0 } else { 1.0 - self.pan };
            let right_gain = if self.pan >= 0.0 { 1.0 } else { 1.0 + self.pan };
            
            stereo_output.push(sample * left_gain);
            stereo_output.push(sample * right_gain);
        }
        
        // Update VU meter and get levels
        let (peak, rms) = self.vu_meter.process(&stereo_output, dt);
        let levels = [peak, rms];
        
        // Store levels for GUI access
        self.last_levels = levels;
        
        (stereo_output, levels)
    }
}

pub struct AudioEngine {
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
    channels: Arc<Mutex<Vec<ChannelProcessor>>>,
    rnnoise: Arc<Mutex<Rnnoise>>, // Keep for backward compatibility
    advanced_denoiser: Option<SharedAdvancedDenoiser>,
    spectrum_analyzer: Arc<Mutex<SpectrumAnalyzer>>,
    spectrum_data: Arc<Mutex<Vec<f32>>>,
    audio_sender: Option<Sender<Vec<f32>>>,
    audio_receiver: Option<Receiver<Vec<f32>>>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let mut channels_vec = Vec::with_capacity(CHANNEL_COUNT);
        for _ in 0..CHANNEL_COUNT {
            channels_vec.push(ChannelProcessor::new());
        }
        let channels = Arc::new(Mutex::new(channels_vec));
        let rnnoise = Arc::new(Mutex::new(Rnnoise::new()));
        let spectrum_analyzer = Arc::new(Mutex::new(SpectrumAnalyzer::new(48000.0)));
        let spectrum_data = Arc::new(Mutex::new(vec![0.0; 512]));
        
        let (audio_sender, audio_receiver) = crossbeam_channel::bounded(1024);
        
        // Initialize advanced denoising system
        let advanced_denoiser = match create_advanced_denoiser(AdvancedDenoisingConfig::default()) {
            Ok(denoiser) => {
                if let Ok(mut d) = denoiser.lock() {
                    d.set_enabled(true);
                }
                Some(denoiser)
            }
            Err(e) => {
                eprintln!("Failed to initialize advanced denoiser: {}", e);
                None
            }
        };
        
        Self {
            input_stream: None,
            output_stream: None,
            channels,
            rnnoise,
            advanced_denoiser,
            spectrum_analyzer,
            spectrum_data,
            audio_sender: Some(audio_sender),
            audio_receiver: Some(audio_receiver),
        }
    }
    
    pub fn update_channel(&self, channel_idx: usize, volume: f32, muted: bool) {
        if let Ok(mut channels) = self.channels.lock() {
            if let Some(channel) = channels.get_mut(channel_idx) {
                channel.volume = volume;
                channel.muted = muted;
            }
        }
    }
    
    pub fn set_rnnoise_enabled(&self, enabled: bool) {
        if let Ok(mut rnnoise) = self.rnnoise.lock() {
            if enabled {
                rnnoise.enable();
            } else {
                rnnoise.disable();
            }
        }
    }

    // Advanced denoising methods
    pub fn set_denoising_mode(&self, mode: DenoisingMode) -> Result<()> {
        if let Some(ref denoiser) = self.advanced_denoiser {
            if let Ok(mut d) = denoiser.lock() {
                d.set_mode(mode)?;
            }
        }
        Ok(())
    }
    
    pub fn get_denoising_mode(&self) -> Option<DenoisingMode> {
        if let Some(ref denoiser) = self.advanced_denoiser {
            if let Ok(d) = denoiser.lock() {
                return Some(d.get_mode());
            }
        }
        None
    }
    
    pub fn set_advanced_denoising_enabled(&self, enabled: bool) {
        if let Some(ref denoiser) = self.advanced_denoiser {
            if let Ok(mut d) = denoiser.lock() {
                d.set_enabled(enabled);
            }
        }
    }
    
    pub fn is_advanced_denoising_enabled(&self) -> bool {
        if let Some(ref denoiser) = self.advanced_denoiser {
            if let Ok(d) = denoiser.lock() {
                return d.is_enabled();
            }
        }
        false
    }
    
    pub fn get_denoising_metrics(&self) -> Option<crate::advanced_denoising::DenoisingMetrics> {
        if let Some(ref denoiser) = self.advanced_denoiser {
            if let Ok(d) = denoiser.lock() {
                return Some(d.get_metrics());
            }
        }
        None
    }
    
    pub fn get_available_denoising_modes(&self) -> Vec<DenoisingMode> {
        if let Some(ref denoiser) = self.advanced_denoiser {
            if let Ok(d) = denoiser.lock() {
                // Return basic modes since we can't downcast the trait object
                return vec![
                    DenoisingMode::Basic,
                    DenoisingMode::Enhanced,
                    DenoisingMode::Maximum,
                ];
            }
        }
        vec![DenoisingMode::Basic] // Fallback to basic mode
    }
    
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let input_device = host.default_input_device().ok_or("No input device available")?;
        let output_device = host.default_output_device().ok_or("No output device available")?;
        
        let input_config = input_device.default_input_config()?;
        let output_config = output_device.default_output_config()?;
        let input_config: StreamConfig = input_config.into();
        let output_config: StreamConfig = output_config.into();

        println!("Input config: {:?}", input_config);
        println!("Output config: {:?}", output_config);

        // Create shared audio buffer for routing between input and output
        let audio_buffer = Arc::new(Mutex::new(VecDeque::<f32>::with_capacity(BUFFER_SIZE * 4)));
        let audio_buffer_out = Arc::clone(&audio_buffer);
        
        let channels = Arc::clone(&self.channels);
        let rnnoise = Arc::clone(&self.rnnoise);
        let advanced_denoiser = self.advanced_denoiser.clone();
        let spectrum_analyzer: Arc<Mutex<SpectrumAnalyzer>> = Arc::clone(&self.spectrum_analyzer);
        let spectrum_data: Arc<Mutex<Vec<f32>>> = Arc::clone(&self.spectrum_data);
        
        let start_time = Instant::now();
        
        // Input stream: capture and process audio
        let input_stream = input_device.build_input_stream(
            &input_config,
            move |data: &[f32], _| {
                let dt = start_time.elapsed().as_secs_f32() % 1.0; // Frame time for VU meters
                
                // Process input through all channels and mix
                let mut mixed_output = vec![0.0; data.len()];
                let mut total_levels = [0.0f32; 2];
                
                if let Ok(mut channels) = channels.lock() {
                    for channel in channels.iter_mut() {
                        // Use advanced denoiser if available, otherwise fall back to legacy RNNoise
                        let (processed, levels) = if advanced_denoiser.is_some() {
                            channel.process_advanced(data, advanced_denoiser.as_ref(), 0.02)
                        } else if let Ok(rnnoise) = rnnoise.lock() {
                            channel.process(data, &rnnoise, 0.02)
                        } else {
                            // Fallback: process without denoising
                            channel.process_advanced(data, None, 0.02)
                        };
                        
                        // Mix the processed audio from this channel
                        for (i, &sample) in processed.iter().enumerate() {
                            if i < mixed_output.len() {
                                mixed_output[i] += sample / CHANNEL_COUNT as f32; // Average mix
                            }
                        }
                        
                        // Accumulate levels for overall monitoring
                        total_levels[0] = total_levels[0].max(levels[0]); // Peak
                        total_levels[1] += levels[1] / CHANNEL_COUNT as f32; // RMS average
                    }
                }
                
                // Update spectrum analyzer
                if let Ok(mut analyzer) = spectrum_analyzer.lock() {
                    let spectrum = analyzer.process(&mixed_output);
                    if let Ok(mut spectrum_out) = spectrum_data.lock() {
                        let copy_len = spectrum_out.len().min(spectrum.len());
                        spectrum_out[..copy_len].copy_from_slice(&spectrum[..copy_len]);
                    }
                }
                
                // Send processed audio to output buffer
                if let Ok(mut buffer) = audio_buffer.lock() {
                    for &sample in &mixed_output {
                        if buffer.len() < BUFFER_SIZE * 4 {
                            buffer.push_back(sample);
                        } else {
                            // Buffer is full, drop oldest samples
                            buffer.pop_front();
                            buffer.push_back(sample);
                        }
                    }
                }
            },
            move |err| {
                eprintln!("Input stream error: {}", err);
            },
            None,
        )?;

        // Output stream: play processed audio
        let output_stream = output_device.build_output_stream(
            &output_config,
            move |data: &mut [f32], _| {
                // Fill output buffer from processed audio buffer
                if let Ok(mut buffer) = audio_buffer_out.lock() {
                    for sample in data.iter_mut() {
                        *sample = buffer.pop_front().unwrap_or(0.0); // Silence if buffer empty
                    }
                } else {
                    // Fallback: output silence if we can't access buffer
                    for sample in data.iter_mut() {
                        *sample = 0.0;
                    }
                }
            },
            move |err| {
                eprintln!("Output stream error: {}", err);
            },
            None,
        )?;

        // Start the streams
        input_stream.play()?;
        output_stream.play()?;

        // Store streams to keep them alive
        self.input_stream = Some(input_stream);
        self.output_stream = Some(output_stream);

        println!("Audio engine started successfully!");
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(input_stream) = self.input_stream.take() {
            let _ = input_stream.pause();
        }
        if let Some(output_stream) = self.output_stream.take() {
            let _ = output_stream.pause();
        }
        println!("Audio engine stopped");
    }

    pub fn is_running(&self) -> bool {
        self.input_stream.is_some() && self.output_stream.is_some()
    }

    pub fn get_channel_levels(&self, channel_idx: usize) -> Option<[f32; 2]> {
        if let Ok(channels) = self.channels.lock() {
            if let Some(channel) = channels.get(channel_idx) {
                return Some(channel.last_levels); // Return actual stored levels
            }
        }
        None
    }

    pub fn set_channel_vst(&self, channel_idx: usize, vst_processor: Option<VstProcessor>) {
        if let Ok(mut channels) = self.channels.lock() {
            if let Some(channel) = channels.get_mut(channel_idx) {
                channel.vst_processor = vst_processor;
            }
        }
    }
    
    pub fn get_spectrum_data(&self) -> Arc<Mutex<Vec<f32>>> {
        self.spectrum_data.clone()
    }
    
    /// Get spectrum data as a simple Vec for GUI display
    pub fn get_spectrum_data_vec(&self) -> Option<Vec<f32>> {
        if let Ok(data) = self.spectrum_data.lock() {
            if !data.is_empty() {
                Some(data.clone())
            } else {
                // Return fake data for demonstration
                Some((0..64).map(|i| (i as f32 * 0.01).sin().abs() * 0.5).collect())
            }
        } else {
            None
        }
    }
    
    pub fn update_channel_advanced(&self, channel_idx: usize, volume: f32, muted: bool, gain: f32, pan: f32) {
        if let Ok(mut channels) = self.channels.lock() {
            if let Some(channel) = channels.get_mut(channel_idx) {
                channel.volume = volume;
                channel.muted = muted;
                channel.gain = gain;
                channel.pan = pan;
            }
        }
    }
}