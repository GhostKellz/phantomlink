use crate::advanced_denoising::{
    AdvancedDenoisingConfig, DenoisingMode, SharedAdvancedDenoiser, create_advanced_denoiser,
};
use crate::audio_effects::{ChannelEffects, ChannelEffectsConfig};
use crate::ghostwave_integration::{
    GhostWaveIntegration, PhantomLinkProfile, detect_nvidia_driver,
};
use crate::gui::visualizer::{SpectrumAnalyzer, VUMeter};
use crate::phantomlink::AudioRouter;
use crate::rnnoise::Rnnoise;
use crate::vst_host::VstProcessor;
use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use crossbeam_channel::{Receiver, Sender};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const BUFFER_SIZE: usize = 1024;
const CHANNEL_COUNT: usize = 4;
const DEFAULT_SAMPLE_RATE: f32 = 48000.0;

/// Audio channel processor with volume, effects, and metering
pub struct ChannelProcessor {
    pub volume: f32,
    pub muted: bool,
    pub vst_processor: Option<VstProcessor>,
    pub gain: f32,
    pub pan: f32,
    #[allow(dead_code)] // Solo functionality for future mixer implementation
    pub solo: bool,
    #[allow(dead_code)] // Internal buffer for VST processing
    buffer: Vec<f32>,
    vu_meter: VUMeter,
    last_levels: [f32; 2], // Store last peak/rms levels
    /// Dynamics effects chain (Gate -> Compressor -> Limiter)
    pub effects: ChannelEffects,
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
            effects: ChannelEffects::new(DEFAULT_SAMPLE_RATE),
        }
    }

    /// Configure effects chain
    #[allow(dead_code)]
    pub fn configure_effects(&mut self, config: &ChannelEffectsConfig) {
        self.effects.apply_config(config);
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

        // Apply dynamics effects chain (Gate -> Compressor -> Limiter)
        self.effects.process(&mut output);

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
    pub fn process_advanced(
        &mut self,
        input: &[f32],
        advanced_denoiser: Option<&SharedAdvancedDenoiser>,
        dt: f32,
    ) -> (Vec<f32>, [f32; 2]) {
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
        if let Some(denoiser) = advanced_denoiser
            && let Ok(mut d) = denoiser.lock()
            && d.is_enabled()
            && let Ok(processed) = d.process_frame(&output)
        {
            output = processed;
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

    /// Process audio with GhostWave AI denoising (GhostWave -> VST chain order)
    /// This is the recommended pipeline for NVIDIA Broadcast-quality processing
    pub fn process_with_ghostwave(
        &mut self,
        input: &[f32],
        ghostwave: Option<&mut GhostWaveIntegration>,
        dt: f32,
    ) -> (Vec<f32>, [f32; 2]) {
        if self.muted {
            return (vec![0.0; input.len()], [0.0, 0.0]);
        }

        let mut output = input.to_vec();

        // 1. Apply input gain (pre-processing)
        let gain_linear = if self.gain >= 0.0 {
            1.0 + self.gain / 20.0
        } else {
            10.0_f32.powf(self.gain / 20.0)
        };

        for sample in &mut output {
            *sample *= gain_linear;
        }

        // 2. Apply GhostWave AI denoising (RTX-accelerated)
        //    This runs BEFORE VST to give clean audio to subsequent effects
        if let Some(gw) = ghostwave
            && gw.is_enabled()
            && let Err(e) = gw.process(&mut output)
        {
            log::trace!("GhostWave processing error: {}", e);
        }

        // 3. Apply VST processing chain (post-denoising)
        //    VST effects receive clean, denoised audio
        if let Some(ref mut vst) = self.vst_processor {
            output = vst.process(&output);
        }

        // 4. Apply output volume
        for sample in &mut output {
            *sample *= self.volume;
        }

        // 5. Apply stereo panning
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

/// Main audio processing engine with denoising and GhostWave integration
pub struct AudioEngine {
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
    channels: Arc<Mutex<Vec<ChannelProcessor>>>,
    rnnoise: Arc<Mutex<Rnnoise>>, // Keep for backward compatibility
    advanced_denoiser: Option<SharedAdvancedDenoiser>,
    spectrum_analyzer: Arc<Mutex<SpectrumAnalyzer>>,
    spectrum_data: Arc<Mutex<Vec<f32>>>,
    #[allow(dead_code)] // For future inter-thread audio routing
    audio_sender: Option<Sender<Vec<f32>>>,
    #[allow(dead_code)] // For future inter-thread audio routing
    audio_receiver: Option<Receiver<Vec<f32>>>,
    // GhostWave Integration
    #[allow(dead_code)] // GhostWave integration for RTX denoising
    ghostwave: Option<Arc<Mutex<GhostWaveIntegration>>>,
    #[allow(dead_code)] // GhostWave enable flag
    use_ghostwave: bool,
    #[allow(dead_code)] // Current GhostWave processing profile
    current_profile: PhantomLinkProfile,
    /// Configurable buffer size (samples per callback)
    buffer_size: usize,
    /// Audio routing matrix (channel → output with gain)
    router: Arc<Mutex<AudioRouter>>,
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

        // Initialize GhostWave integration
        let ghostwave = match GhostWaveIntegration::new() {
            Ok(gw) => {
                log::info!("GhostWave initialized successfully");
                Some(Arc::new(Mutex::new(gw)))
            }
            Err(e) => {
                log::warn!(
                    "Failed to initialize GhostWave: {}, falling back to traditional processing",
                    e
                );
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
            use_ghostwave: ghostwave.is_some(),
            ghostwave,
            current_profile: PhantomLinkProfile::Balanced,
            buffer_size: BUFFER_SIZE,
            router: Arc::new(Mutex::new(AudioRouter::new())),
        }
    }

    /// Update channel volume and mute state
    #[allow(dead_code)] // API for GUI channel control
    pub fn update_channel(&self, channel_idx: usize, volume: f32, muted: bool) {
        if let Ok(mut channels) = self.channels.lock()
            && let Some(channel) = channels.get_mut(channel_idx)
        {
            channel.volume = volume;
            channel.muted = muted;
        }
    }

    /// Enable or disable legacy RNNoise denoising
    #[allow(dead_code)] // API for noise suppression toggle
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
    #[allow(dead_code)] // API for denoising settings UI
    pub fn set_denoising_mode(&self, mode: DenoisingMode) -> Result<()> {
        if let Some(ref denoiser) = self.advanced_denoiser
            && let Ok(mut d) = denoiser.lock()
        {
            d.set_mode(mode)?;
        }
        Ok(())
    }

    /// Get current denoising mode
    #[allow(dead_code)] // API for denoising settings UI
    pub fn get_denoising_mode(&self) -> Option<DenoisingMode> {
        if let Some(ref denoiser) = self.advanced_denoiser
            && let Ok(d) = denoiser.lock()
        {
            return Some(d.get_mode());
        }
        None
    }

    /// Enable or disable advanced denoising
    pub fn set_advanced_denoising_enabled(&self, enabled: bool) {
        if let Some(ref denoiser) = self.advanced_denoiser
            && let Ok(mut d) = denoiser.lock()
        {
            d.set_enabled(enabled);
        }
    }

    /// Check if advanced denoising is currently active
    #[allow(dead_code)] // API for status display
    pub fn is_advanced_denoising_enabled(&self) -> bool {
        if let Some(ref denoiser) = self.advanced_denoiser
            && let Ok(d) = denoiser.lock()
        {
            return d.is_enabled();
        }
        false
    }

    /// Get denoising metrics for display
    #[allow(dead_code)] // API for metrics panel
    pub fn get_denoising_metrics(&self) -> Option<crate::advanced_denoising::DenoisingMetrics> {
        if let Some(ref denoiser) = self.advanced_denoiser
            && let Ok(d) = denoiser.lock()
        {
            return Some(d.get_metrics());
        }
        None
    }

    /// Get list of available denoising modes
    #[allow(dead_code)] // API for mode selector UI
    pub fn get_available_denoising_modes(&self) -> Vec<DenoisingMode> {
        if let Some(ref denoiser) = self.advanced_denoiser
            && denoiser.lock().is_ok()
        {
            // Return basic modes since we can't downcast the trait object
            return vec![
                DenoisingMode::Basic,
                DenoisingMode::Enhanced,
                DenoisingMode::Maximum,
            ];
        }
        vec![DenoisingMode::Basic] // Fallback to basic mode
    }

    /// Get a reference to the audio router for configuration
    #[allow(dead_code)] // Public API for route management from GUI
    pub fn get_router(&self) -> &Arc<Mutex<AudioRouter>> {
        &self.router
    }

    /// Configure a route from a channel to an output with gain
    #[allow(dead_code)] // Public API for route management from GUI
    pub fn add_route(&self, from: String, to: String, gain: f32) {
        if let Ok(mut router) = self.router.lock() {
            router.add_route(from, to, gain);
        }
    }

    /// Remove a route
    #[allow(dead_code)] // Public API for route management from GUI
    pub fn remove_route(&self, from: &str, to: &str) {
        if let Ok(mut router) = self.router.lock() {
            router.remove_route(from, to);
        }
    }

    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let input_device = host
            .default_input_device()
            .ok_or("No input device available")?;
        let output_device = host
            .default_output_device()
            .ok_or("No output device available")?;

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
        let ghostwave = self.ghostwave.clone();
        let use_ghostwave = self.use_ghostwave;
        let spectrum_analyzer: Arc<Mutex<SpectrumAnalyzer>> = Arc::clone(&self.spectrum_analyzer);
        let spectrum_data: Arc<Mutex<Vec<f32>>> = Arc::clone(&self.spectrum_data);
        let router = Arc::clone(&self.router);

        // Input stream: capture and process audio
        let input_stream = input_device.build_input_stream(
            &input_config,
            move |data: &[f32], _| {
                // Process input through all channels and mix
                let mut mixed_output = vec![0.0; data.len()];
                let mut total_levels = [0.0f32; 2];

                // Get GhostWave lock outside the channel loop for efficiency
                let mut gw_guard = if use_ghostwave {
                    ghostwave.as_ref().and_then(|gw| gw.lock().ok())
                } else {
                    None
                };

                if let Ok(mut channels) = channels.lock() {
                    // Collect per-channel processed audio for routing
                    let mut channel_outputs: Vec<Vec<f32>> = Vec::with_capacity(channels.len());

                    for channel in channels.iter_mut() {
                        // Priority: GhostWave (RTX) -> Advanced Denoiser -> Legacy RNNoise
                        let (processed, levels) = if let Some(ref mut gw) = gw_guard {
                            channel.process_with_ghostwave(data, Some(gw), 0.02)
                        } else if advanced_denoiser.is_some() {
                            channel.process_advanced(data, advanced_denoiser.as_ref(), 0.02)
                        } else if let Ok(rnnoise) = rnnoise.lock() {
                            channel.process(data, &rnnoise, 0.02)
                        } else {
                            channel.process_advanced(data, None, 0.02)
                        };

                        channel_outputs.push(processed);

                        // Accumulate levels for overall monitoring
                        total_levels[0] = total_levels[0].max(levels[0]); // Peak
                        total_levels[1] += levels[1] / CHANNEL_COUNT as f32; // RMS average
                    }

                    // Mix channels: use AudioRouter if routes are configured, otherwise average
                    let use_router = router.lock().is_ok_and(|r| !r.get_routes().is_empty());

                    if use_router {
                        if let Ok(r) = router.lock() {
                            let channel_map: std::collections::HashMap<String, usize> =
                                (0..channel_outputs.len())
                                    .map(|i| (format!("ch{}", i), i))
                                    .collect();
                            let output_names = vec!["master".to_string()];
                            let routed = r.apply_routing(&channel_outputs, &channel_map, &output_names);
                            if let Some(master) = routed.first() {
                                let copy_len = mixed_output.len().min(master.len());
                                mixed_output[..copy_len].copy_from_slice(&master[..copy_len]);
                            }
                        }
                    } else {
                        // Default: average all channels equally
                        for processed in &channel_outputs {
                            for (i, &sample) in processed.iter().enumerate() {
                                if i < mixed_output.len() {
                                    mixed_output[i] += sample / CHANNEL_COUNT as f32;
                                }
                            }
                        }
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

    /// Check if audio streams are running
    #[allow(dead_code)] // API for status display
    pub fn is_running(&self) -> bool {
        self.input_stream.is_some() && self.output_stream.is_some()
    }

    // GhostWave-specific methods

    /// Enable or disable GhostWave RTX denoising
    #[allow(dead_code)] // API for GhostWave settings
    pub fn set_ghostwave_enabled(&mut self, enabled: bool) {
        self.use_ghostwave = enabled && self.ghostwave.is_some();
        if let Some(ref gw) = self.ghostwave
            && let Ok(mut g) = gw.lock()
        {
            g.set_enabled(enabled);
        }
    }

    /// Check if GhostWave is enabled
    #[allow(dead_code)] // API for status display
    pub fn is_ghostwave_enabled(&self) -> bool {
        self.use_ghostwave
    }

    /// Set GhostWave processing profile
    #[allow(dead_code)] // API for GhostWave settings
    pub fn set_ghostwave_profile(&mut self, profile: PhantomLinkProfile) {
        self.current_profile = profile;
        if let Some(ref gw) = self.ghostwave
            && let Ok(mut g) = gw.lock()
        {
            if let Err(e) = g.set_profile(profile) {
                log::warn!("Failed to set GhostWave profile: {}", e);
            } else {
                log::info!("Applied GhostWave profile: {:?}", profile);
            }
        }
    }

    /// Get current GhostWave processing latency
    #[allow(dead_code)] // API for latency monitoring
    pub fn get_ghostwave_latency(&self) -> f32 {
        if let Some(ref gw) = self.ghostwave
            && let Ok(g) = gw.lock()
        {
            return g.get_metrics().latency_ms;
        }
        0.0
    }

    /// Check if RTX acceleration is active
    #[allow(dead_code)] // API for status display
    pub fn is_rtx_active(&self) -> bool {
        if let Some(ref gw) = self.ghostwave
            && let Ok(g) = gw.lock()
        {
            return g.is_rtx_active();
        }
        false
    }

    /// Get NVIDIA GPU name for display
    #[allow(dead_code)] // API for hardware info display
    pub fn get_nvidia_gpu_name(&self) -> String {
        if let Some(ref gw) = self.ghostwave
            && let Ok(g) = gw.lock()
        {
            return g.get_rtx_status().gpu_name.clone();
        }
        detect_nvidia_driver().gpu_name
    }

    /// Get reference to GhostWave integration for GUI access
    #[allow(dead_code)] // API for direct GhostWave access from GUI
    pub fn get_ghostwave(&self) -> Option<&Arc<Mutex<GhostWaveIntegration>>> {
        self.ghostwave.as_ref()
    }

    /// Get GhostWave metrics for telemetry display
    pub fn get_ghostwave_metrics(&self) -> Option<crate::ghostwave_integration::ProcessingMetrics> {
        if let Some(ref gw) = self.ghostwave
            && let Ok(g) = gw.lock()
        {
            return Some(g.get_metrics().clone());
        }
        None
    }

    /// Get GhostWave GPU fallback status
    pub fn get_ghostwave_fallback_status(
        &self,
    ) -> Option<crate::ghostwave_integration::GpuFallbackStatus> {
        if let Some(ref gw) = self.ghostwave
            && let Ok(g) = gw.lock()
        {
            return Some(g.get_gpu_fallback().clone());
        }
        None
    }

    pub fn get_channel_levels(&self, channel_idx: usize) -> Option<[f32; 2]> {
        if let Ok(channels) = self.channels.lock()
            && let Some(channel) = channels.get(channel_idx)
        {
            return Some(channel.last_levels); // Return actual stored levels
        }
        None
    }

    /// Get channel state (volume, muted, gain, pan)
    pub fn get_channel_state(&self, channel_idx: usize) -> Option<(f32, bool, f32, f32)> {
        if let Ok(channels) = self.channels.lock()
            && let Some(channel) = channels.get(channel_idx)
        {
            return Some((channel.volume, channel.muted, channel.gain, channel.pan));
        }
        None
    }

    /// Set VST plugin for a channel
    #[allow(dead_code)] // API for VST plugin management
    pub fn set_channel_vst(&self, channel_idx: usize, vst_processor: Option<VstProcessor>) {
        if let Ok(mut channels) = self.channels.lock()
            && let Some(channel) = channels.get_mut(channel_idx)
        {
            channel.vst_processor = vst_processor;
        }
    }

    /// Get shared spectrum data for visualization
    #[allow(dead_code)] // API for spectrum analyzer panel
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
                Some(
                    (0..64)
                        .map(|i| (i as f32 * 0.01).sin().abs() * 0.5)
                        .collect(),
                )
            }
        } else {
            None
        }
    }

    /// Update channel with full parameter set
    #[allow(dead_code)] // API for advanced channel control
    pub fn update_channel_advanced(
        &self,
        channel_idx: usize,
        volume: f32,
        muted: bool,
        gain: f32,
        pan: f32,
    ) {
        if let Ok(mut channels) = self.channels.lock()
            && let Some(channel) = channels.get_mut(channel_idx)
        {
            channel.volume = volume;
            channel.muted = muted;
            channel.gain = gain;
            channel.pan = pan;
        }
    }

    /// Get current buffer size
    #[allow(dead_code)]
    pub fn get_buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// Set buffer size (requires restart to take effect)
    /// Returns true if the size changed
    pub fn set_buffer_size(&mut self, size: usize) -> bool {
        if self.buffer_size != size {
            self.buffer_size = size;
            log::info!("Buffer size set to {} samples", size);
            true
        } else {
            false
        }
    }

    /// Restart the audio engine with current settings
    /// Use this after changing buffer size
    pub fn restart(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let was_running = self.is_running();
        if was_running {
            self.stop();
        }
        if was_running {
            self.start()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_processor_defaults() {
        let proc = ChannelProcessor::new();
        assert_eq!(proc.volume, 0.8);
        assert!(!proc.muted);
        assert_eq!(proc.gain, 0.0);
        assert_eq!(proc.pan, 0.0);
        assert!(!proc.solo);
    }

    #[test]
    fn test_channel_processor_muted_returns_silence() {
        let mut proc = ChannelProcessor::new();
        proc.muted = true;
        let rnnoise = Rnnoise::new();
        let input = vec![0.5; 480];
        let (output, levels) = proc.process(&input, &rnnoise, 0.016);
        assert!(output.iter().all(|&s| s == 0.0));
        assert_eq!(levels, [0.0, 0.0]);
    }

    #[test]
    fn test_channel_processor_stereo_doubling() {
        let mut proc = ChannelProcessor::new();
        proc.gain = 0.0;
        proc.volume = 1.0;
        proc.pan = 0.0;
        let rnnoise = Rnnoise::new();
        let input = vec![0.5; 10];
        let (output, _) = proc.process(&input, &rnnoise, 0.016);
        // Mono input → stereo output (2x samples)
        assert_eq!(output.len(), input.len() * 2);
    }

    #[test]
    fn test_channel_processor_panning_left() {
        let mut proc = ChannelProcessor::new();
        proc.effects.limiter.set_enabled(false); // Disable limiter for math test
        proc.gain = 0.0;
        proc.volume = 1.0;
        proc.pan = -1.0; // Full left
        let rnnoise = Rnnoise::new();
        let input = vec![1.0; 1];
        let (output, _) = proc.process(&input, &rnnoise, 0.016);
        // Left channel should be full, right should be zero
        assert_eq!(output.len(), 2);
        assert!((output[0] - 1.0).abs() < 0.001);
        assert!((output[1]).abs() < 0.001);
    }

    #[test]
    fn test_channel_processor_panning_right() {
        let mut proc = ChannelProcessor::new();
        proc.effects.limiter.set_enabled(false); // Disable limiter for math test
        proc.gain = 0.0;
        proc.volume = 1.0;
        proc.pan = 1.0; // Full right
        let rnnoise = Rnnoise::new();
        let input = vec![1.0; 1];
        let (output, _) = proc.process(&input, &rnnoise, 0.016);
        assert_eq!(output.len(), 2);
        assert!((output[0]).abs() < 0.001);
        assert!((output[1] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_channel_processor_gain_positive() {
        let mut proc = ChannelProcessor::new();
        proc.effects.limiter.set_enabled(false); // Disable limiter for math test
        proc.gain = 20.0; // +20dB → gain_linear = 2.0
        proc.volume = 1.0;
        proc.pan = 0.0;
        let rnnoise = Rnnoise::new();
        let input = vec![0.25; 1];
        let (output, _) = proc.process(&input, &rnnoise, 0.016);
        // With +20dB gain_linear = 1.0 + 20.0/20.0 = 2.0
        // So output should be 0.25 * 2.0 = 0.5 per channel
        assert!((output[0] - 0.5).abs() < 0.01);
        assert!((output[1] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_channel_processor_volume_scaling() {
        let mut proc = ChannelProcessor::new();
        proc.effects.limiter.set_enabled(false); // Disable limiter for math test
        proc.gain = 0.0;
        proc.volume = 0.5;
        proc.pan = 0.0;
        let rnnoise = Rnnoise::new();
        let input = vec![1.0; 1];
        let (output, _) = proc.process(&input, &rnnoise, 0.016);
        assert!((output[0] - 0.5).abs() < 0.01);
        assert!((output[1] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_audio_engine_defaults() {
        let engine = AudioEngine::new();
        assert_eq!(engine.get_buffer_size(), BUFFER_SIZE);
        assert!(!engine.is_running());
    }

    #[test]
    fn test_audio_engine_buffer_size_change() {
        let mut engine = AudioEngine::new();
        assert!(engine.set_buffer_size(512));
        assert_eq!(engine.get_buffer_size(), 512);
        assert!(!engine.set_buffer_size(512)); // Same size, no change
    }

    #[test]
    fn test_audio_engine_channel_update() {
        let mut engine = AudioEngine::new();
        engine.update_channel(0, 0.5, true);
        let state = engine.get_channel_state(0);
        assert!(state.is_some());
        let (vol, muted, _gain, _pan) = state.unwrap();
        assert_eq!(vol, 0.5);
        assert!(muted);
    }

    #[test]
    fn test_audio_engine_channel_out_of_bounds() {
        let engine = AudioEngine::new();
        assert!(engine.get_channel_state(99).is_none());
        assert!(engine.get_channel_levels(99).is_none());
    }

    #[test]
    fn test_audio_engine_ghostwave_toggle() {
        let mut engine = AudioEngine::new();
        // GhostWave is enabled by default when integration initializes successfully
        assert!(engine.is_ghostwave_enabled());
        engine.set_ghostwave_enabled(false);
        assert!(!engine.is_ghostwave_enabled());
        engine.set_ghostwave_enabled(true);
        assert!(engine.is_ghostwave_enabled());
    }

    #[test]
    fn test_audio_engine_advanced_denoising_toggle() {
        let mut engine = AudioEngine::new();
        // Advanced denoiser is enabled by default on successful initialization
        assert!(engine.is_advanced_denoising_enabled());
        engine.set_advanced_denoising_enabled(false);
        assert!(!engine.is_advanced_denoising_enabled());
    }

    #[test]
    fn test_audio_router_basic_routing() {
        use crate::phantomlink::AudioRouter;
        use std::collections::HashMap;

        let mut router = AudioRouter::new();
        router.add_route("ch0".to_string(), "master".to_string(), 1.0);

        let input = vec![vec![0.5; 4]]; // 1 channel, 4 samples
        let channel_map: HashMap<String, usize> = [("ch0".to_string(), 0)].into_iter().collect();
        let outputs = router.apply_routing(&input, &channel_map, &["master".to_string()]);

        assert_eq!(outputs.len(), 1);
        assert!(outputs[0].iter().all(|&s| (s - 0.5).abs() < 0.001));
    }

    #[test]
    fn test_audio_router_gain_scaling() {
        use crate::phantomlink::AudioRouter;
        use std::collections::HashMap;

        let mut router = AudioRouter::new();
        router.add_route("ch0".to_string(), "master".to_string(), 0.5);

        let input = vec![vec![1.0; 4]];
        let channel_map: HashMap<String, usize> = [("ch0".to_string(), 0)].into_iter().collect();
        let outputs = router.apply_routing(&input, &channel_map, &["master".to_string()]);

        assert!(outputs[0].iter().all(|&s| (s - 0.5).abs() < 0.001));
    }

    #[test]
    fn test_audio_router_multiple_routes_sum() {
        use crate::phantomlink::AudioRouter;
        use std::collections::HashMap;

        let mut router = AudioRouter::new();
        router.add_route("ch0".to_string(), "master".to_string(), 1.0);
        router.add_route("ch1".to_string(), "master".to_string(), 1.0);

        let input = vec![vec![0.3; 4], vec![0.2; 4]];
        let channel_map: HashMap<String, usize> =
            [("ch0".to_string(), 0), ("ch1".to_string(), 1)].into_iter().collect();
        let outputs = router.apply_routing(&input, &channel_map, &["master".to_string()]);

        // Both channels should be summed: 0.3 + 0.2 = 0.5
        assert!(outputs[0].iter().all(|&s| (s - 0.5).abs() < 0.001));
    }

    #[test]
    fn test_audio_router_disabled_route_skipped() {
        use crate::phantomlink::AudioRouter;
        use std::collections::HashMap;

        let mut router = AudioRouter::new();
        router.add_route("ch0".to_string(), "master".to_string(), 1.0);
        router.set_route_enabled("ch0", "master", false);

        let input = vec![vec![0.5; 4]];
        let channel_map: HashMap<String, usize> = [("ch0".to_string(), 0)].into_iter().collect();
        let outputs = router.apply_routing(&input, &channel_map, &["master".to_string()]);

        // Disabled route produces silence
        assert!(outputs[0].iter().all(|&s| s.abs() < 0.001));
    }

    #[test]
    fn test_audio_router_empty_returns_silence() {
        use crate::phantomlink::AudioRouter;
        use std::collections::HashMap;

        let router = AudioRouter::new();
        let input = vec![vec![0.5; 4]];
        let channel_map: HashMap<String, usize> = [("ch0".to_string(), 0)].into_iter().collect();
        let outputs = router.apply_routing(&input, &channel_map, &["master".to_string()]);

        // No routes means silence
        assert!(outputs[0].iter().all(|&s| s.abs() < 0.001));
    }

    #[test]
    fn test_audio_router_empty_input() {
        use crate::phantomlink::AudioRouter;
        use std::collections::HashMap;

        let mut router = AudioRouter::new();
        router.add_route("ch0".to_string(), "master".to_string(), 1.0);

        let input: Vec<Vec<f32>> = vec![];
        let channel_map: HashMap<String, usize> = HashMap::new();
        let outputs = router.apply_routing(&input, &channel_map, &["master".to_string()]);

        assert!(outputs.is_empty());
    }

    #[test]
    fn test_audio_engine_router_integration() {
        let engine = AudioEngine::new();
        // Router should be accessible and initially empty
        let router = engine.get_router();
        let guard = router.lock().unwrap();
        assert!(guard.get_routes().is_empty());
    }

    #[test]
    fn test_audio_engine_add_remove_routes() {
        let engine = AudioEngine::new();
        engine.add_route("ch0".to_string(), "master".to_string(), 1.0);

        {
            let guard = engine.get_router().lock().unwrap();
            assert_eq!(guard.get_routes().len(), 1);
        }

        engine.remove_route("ch0", "master");

        {
            let guard = engine.get_router().lock().unwrap();
            assert!(guard.get_routes().is_empty());
        }
    }

    #[test]
    fn test_channel_processor_effects_chain_applied() {
        let mut proc = ChannelProcessor::new();
        proc.gain = 0.0;
        proc.volume = 1.0;
        proc.pan = 0.0;
        // Enable limiter (it's part of effects chain)
        proc.effects.limiter.set_enabled(true);

        let rnnoise = Rnnoise::new();
        // Input a very loud signal that would clip
        let input = vec![2.0; 10];
        let (output, _) = proc.process(&input, &rnnoise, 0.016);

        // Limiter should prevent any sample from exceeding ~1.0
        for &sample in &output {
            assert!(sample.abs() <= 1.5, "Limiter should control peaks, got {}", sample);
        }
    }

    #[test]
    fn test_audio_engine_spectrum_data() {
        let engine = AudioEngine::new();
        // Spectrum data should return something (real or fake demo data)
        let spectrum = engine.get_spectrum_data_vec();
        assert!(spectrum.is_some());
        let data = spectrum.unwrap();
        assert!(!data.is_empty());
    }
}
