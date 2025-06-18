use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
// Removed crossbeam_channel import
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::rnnoise::Rnnoise;
use crate::vst_host::VstProcessor;
use crate::gui::visualizer::{SpectrumAnalyzer, VUMeter};
use crossbeam_channel::{Receiver, Sender};

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
        
        // Apply noise reduction if enabled
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
        
        (stereo_output, levels)
    }
}

pub struct AudioEngine {
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
    channels: Arc<Mutex<Vec<ChannelProcessor>>>,
    rnnoise: Arc<Mutex<Rnnoise>>,
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
        let spectrum_analyzer = Arc::new(Mutex::new(SpectrumAnalyzer::new(1024, 48000.0)));
        let spectrum_data = Arc::new(Mutex::new(vec![0.0; 512]));
        
        let (audio_sender, audio_receiver) = crossbeam_channel::bounded(1024);
        
        Self {
            input_stream: None,
            output_stream: None,
            channels,
            rnnoise,
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

    pub fn start(&mut self) {
        let host = cpal::default_host();
        let input_device = host.default_input_device().expect("No input device available");
        let output_device = host.default_output_device().expect("No output device available");
        let input_config = input_device.default_input_config().unwrap();
        let output_config = output_device.default_output_config().unwrap();
        let input_config: StreamConfig = input_config.into();
        let output_config: StreamConfig = output_config.into();

        // Create simple buffer for audio routing
        let audio_buffer = std::sync::Arc::new(std::sync::Mutex::new(VecDeque::<f32>::with_capacity(BUFFER_SIZE * 4)));
        let audio_buffer_out = Arc::clone(&audio_buffer);
        
        let channels = Arc::clone(&self.channels);
        let rnnoise = Arc::clone(&self.rnnoise);
        
        // Input stream: capture and process audio
        let input_stream = input_device.build_input_stream(
            &input_config,
            move |data: &[f32], _| {
                // Process input through all channels and mix
                let mut mixed_output = vec![0.0; data.len()];
                
                if let (Ok(mut channels), Ok(rnnoise)) = (channels.lock(), rnnoise.lock()) {
                    for channel in channels.iter_mut() {
                        let (processed, _levels) = channel.process(data, &rnnoise, 0.02); // Assume ~20ms frame
                        for (i, &sample) in processed.iter().enumerate() {
                            if i < mixed_output.len() {
                                mixed_output[i] += sample / CHANNEL_COUNT as f32; // Mix channels
                            }
                        }
                    }
                }
                
                // Send processed audio to buffer
                if let Ok(mut buffer) = audio_buffer.lock() {
                    for &sample in &mixed_output {
                        if buffer.len() < BUFFER_SIZE * 4 {
                            buffer.push_back(sample);
                        }
                    }
                }
            },
            move |err| {
                eprintln!("Input stream error: {}", err);
            },
            None,
        ).expect("Failed to build input stream");

        // Output stream: play processed audio
        let output_stream = output_device.build_output_stream(
            &output_config,
            move |data: &mut [f32], _| {
                // Fill output buffer from audio buffer
                if let Ok(mut buffer) = audio_buffer_out.lock() {
                    for sample in data.iter_mut() {
                        *sample = buffer.pop_front().unwrap_or(0.0);
                    }
                }
            },
            move |err| {
                eprintln!("Output stream error: {}", err);
            },
            None,
        ).expect("Failed to build output stream");

        input_stream.play().expect("Failed to start input stream");
        output_stream.play().expect("Failed to start output stream");

        self.input_stream = Some(input_stream);
        self.output_stream = Some(output_stream);
    }

    pub fn set_channel_vst(&self, channel_idx: usize, vst_processor: Option<VstProcessor>) {
        if let Ok(mut channels) = self.channels.lock() {
            if let Some(channel) = channels.get_mut(channel_idx) {
                channel.vst_processor = vst_processor;
            }
        }
    }
    
    pub fn get_spectrum_data(&self) -> Arc<Mutex<Vec<f32>>> {
        Arc::clone(&self.spectrum_data)
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