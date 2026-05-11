//! JACK Audio Connection Kit integration for professional audio routing.
//!
//! Provides low-latency audio I/O through JACK when available,
//! with automatic fallback to ALSA/PipeWire when JACK is not running.

use crossbeam_channel::{Receiver, Sender, bounded};
use jack::{AudioIn, AudioOut, Client, ClientOptions, Control, Port, ProcessHandler, ProcessScope};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

/// Audio data for inter-thread communication
pub struct AudioData {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
}

/// Shared state between JACK process thread and main thread
pub struct JackSharedState {
    /// Input samples received from JACK
    input_tx: Sender<AudioData>,
    /// Output samples to send to JACK
    output_rx: Receiver<AudioData>,
    /// Current peak levels [left_in, right_in, left_out, right_out]
    pub peak_levels: [Arc<AtomicU32>; 4],
    /// Processing enabled flag
    pub enabled: Arc<AtomicBool>,
    /// XRun count
    pub xruns: Arc<AtomicU32>,
}

struct JackHandler {
    input_left: Port<AudioIn>,
    input_right: Port<AudioIn>,
    output_left: Port<AudioOut>,
    output_right: Port<AudioOut>,
    state: Arc<JackSharedState>,
    /// Fallback buffer when no output is ready
    silence: Vec<f32>,
}

impl ProcessHandler for JackHandler {
    fn process(&mut self, _client: &Client, scope: &ProcessScope) -> Control {
        let in_left = self.input_left.as_slice(scope);
        let in_right = self.input_right.as_slice(scope);
        let out_left = self.output_left.as_mut_slice(scope);
        let out_right = self.output_right.as_mut_slice(scope);

        // Calculate input peak levels
        let peak_left_in = in_left.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        let peak_right_in = in_right.iter().map(|s| s.abs()).fold(0.0f32, f32::max);

        self.state.peak_levels[0].store(peak_left_in.to_bits(), Ordering::Relaxed);
        self.state.peak_levels[1].store(peak_right_in.to_bits(), Ordering::Relaxed);

        // Send input to main thread (non-blocking)
        let _ = self.state.input_tx.try_send(AudioData {
            left: in_left.to_vec(),
            right: in_right.to_vec(),
        });

        // Check if processing is enabled
        if !self.state.enabled.load(Ordering::Relaxed) {
            // Passthrough mode - copy input directly to output
            out_left.copy_from_slice(in_left);
            out_right.copy_from_slice(in_right);
        } else {
            // Try to get processed output from main thread
            if let Ok(data) = self.state.output_rx.try_recv() {
                let copy_len = out_left.len().min(data.left.len());
                out_left[..copy_len].copy_from_slice(&data.left[..copy_len]);
                out_right[..copy_len].copy_from_slice(&data.right[..copy_len]);

                // Calculate output peak levels
                let peak_left_out = data.left.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
                let peak_right_out = data.right.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
                self.state.peak_levels[2].store(peak_left_out.to_bits(), Ordering::Relaxed);
                self.state.peak_levels[3].store(peak_right_out.to_bits(), Ordering::Relaxed);
            } else {
                // No processed data available - output silence to avoid glitches
                self.silence.resize(out_left.len(), 0.0);
                out_left.copy_from_slice(&self.silence);
                out_right.copy_from_slice(&self.silence);
            }
        }

        Control::Continue
    }
}

/// JACK audio client for PhantomLink
pub struct JackClient {
    client: Option<jack::AsyncClient<Notifications, JackHandler>>,
    /// Receiver for input audio from JACK
    input_rx: Option<Receiver<AudioData>>,
    /// Sender for output audio to JACK
    output_tx: Option<Sender<AudioData>>,
    /// Shared state
    state: Option<Arc<JackSharedState>>,
    /// JACK client name (for port connections)
    client_name: String,
    /// Sample rate
    sample_rate: usize,
    /// Buffer size
    buffer_size: u32,
    /// Whether JACK is available
    available: bool,
}

/// JACK notification handler
struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn xrun(&mut self, _client: &Client) -> Control {
        log::warn!("JACK xrun detected");
        Control::Continue
    }

    fn sample_rate(&mut self, _client: &Client, srate: jack::Frames) -> Control {
        log::info!("JACK sample rate changed to {}", srate);
        Control::Continue
    }
}

impl JackClient {
    /// Create a new JACK client, or return an inactive client if JACK is not available
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        match Client::new("PhantomLink", ClientOptions::NO_START_SERVER) {
            Ok((client, status)) => {
                log::info!("JACK client created: {:?}", status);

                let sample_rate = client.sample_rate() as usize;
                let buffer_size = client.buffer_size();
                let client_name = client.name().to_string();

                log::info!(
                    "JACK: {}Hz, {} samples/buffer, client name: {}",
                    sample_rate,
                    buffer_size,
                    client_name
                );

                // Create ports
                let input_left = client.register_port("input_L", AudioIn::default())?;
                let input_right = client.register_port("input_R", AudioIn::default())?;
                let output_left = client.register_port("output_L", AudioOut::default())?;
                let output_right = client.register_port("output_R", AudioOut::default())?;

                // Create channels for audio data exchange (with small buffer to prevent blocking)
                let (input_tx, input_rx) = bounded(4);
                let (output_tx, output_rx) = bounded(4);

                // Create shared state
                let state = Arc::new(JackSharedState {
                    input_tx,
                    output_rx,
                    peak_levels: [
                        Arc::new(AtomicU32::new(0)),
                        Arc::new(AtomicU32::new(0)),
                        Arc::new(AtomicU32::new(0)),
                        Arc::new(AtomicU32::new(0)),
                    ],
                    enabled: Arc::new(AtomicBool::new(true)),
                    xruns: Arc::new(AtomicU32::new(0)),
                });

                let handler = JackHandler {
                    input_left,
                    input_right,
                    output_left,
                    output_right,
                    state: Arc::clone(&state),
                    silence: Vec::new(),
                };

                // Activate client
                let async_client = client.activate_async(Notifications, handler)?;
                log::info!("JACK client activated successfully");

                Ok(Self {
                    client: Some(async_client),
                    input_rx: Some(input_rx),
                    output_tx: Some(output_tx),
                    state: Some(state),
                    client_name,
                    sample_rate,
                    buffer_size,
                    available: true,
                })
            }
            Err(e) => {
                log::info!("JACK not available: {}. Using PipeWire/ALSA fallback.", e);
                Ok(Self {
                    client: None,
                    input_rx: None,
                    output_tx: None,
                    state: None,
                    client_name: String::new(),
                    sample_rate: 48000,
                    buffer_size: 256,
                    available: false,
                })
            }
        }
    }

    /// Check if JACK is available
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Get the sample rate
    pub fn get_sample_rate(&self) -> usize {
        self.sample_rate
    }

    /// Get the buffer size
    pub fn get_buffer_size(&self) -> u32 {
        self.buffer_size
    }

    /// Get the JACK client name
    pub fn get_client_name(&self) -> &str {
        &self.client_name
    }

    /// Enable or disable audio processing (when disabled, input passes through directly)
    pub fn set_processing_enabled(&self, enabled: bool) {
        if let Some(ref state) = self.state {
            state.enabled.store(enabled, Ordering::Relaxed);
        }
    }

    /// Get current peak levels [left_in, right_in, left_out, right_out]
    pub fn get_peak_levels(&self) -> [f32; 4] {
        if let Some(ref state) = self.state {
            [
                f32::from_bits(state.peak_levels[0].load(Ordering::Relaxed)),
                f32::from_bits(state.peak_levels[1].load(Ordering::Relaxed)),
                f32::from_bits(state.peak_levels[2].load(Ordering::Relaxed)),
                f32::from_bits(state.peak_levels[3].load(Ordering::Relaxed)),
            ]
        } else {
            [0.0; 4]
        }
    }

    /// Receive input audio from JACK (non-blocking)
    pub fn receive_input(&self) -> Option<AudioData> {
        self.input_rx.as_ref().and_then(|rx| rx.try_recv().ok())
    }

    /// Send processed audio to JACK (non-blocking)
    pub fn send_output(&self, data: AudioData) -> bool {
        if let Some(ref tx) = self.output_tx {
            tx.try_send(data).is_ok()
        } else {
            false
        }
    }

    /// Connect PhantomLink input ports to system capture ports
    pub fn connect_inputs_to_system(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref client) = self.client {
            let client_ref = client.as_client();

            // Find system capture ports
            let capture_ports =
                client_ref.ports(Some("system:capture.*"), None, jack::PortFlags::IS_OUTPUT);

            // Connect first two capture ports to our inputs
            if capture_ports.len() >= 2 {
                let our_input_l = format!("{}:input_L", self.client_name);
                let our_input_r = format!("{}:input_R", self.client_name);

                client_ref.connect_ports_by_name(&capture_ports[0], &our_input_l)?;
                client_ref.connect_ports_by_name(&capture_ports[1], &our_input_r)?;

                log::info!("Connected {} -> {}", capture_ports[0], our_input_l);
                log::info!("Connected {} -> {}", capture_ports[1], our_input_r);
            }
        }
        Ok(())
    }

    /// Connect PhantomLink output ports to system playback ports
    pub fn connect_outputs_to_system(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref client) = self.client {
            let client_ref = client.as_client();

            // Find system playback ports
            let playback_ports =
                client_ref.ports(Some("system:playback.*"), None, jack::PortFlags::IS_INPUT);

            // Connect our outputs to first two playback ports
            if playback_ports.len() >= 2 {
                let our_output_l = format!("{}:output_L", self.client_name);
                let our_output_r = format!("{}:output_R", self.client_name);

                client_ref.connect_ports_by_name(&our_output_l, &playback_ports[0])?;
                client_ref.connect_ports_by_name(&our_output_r, &playback_ports[1])?;

                log::info!("Connected {} -> {}", our_output_l, playback_ports[0]);
                log::info!("Connected {} -> {}", our_output_r, playback_ports[1]);
            }
        }
        Ok(())
    }

    /// Connect to default system ports (both input and output)
    pub fn connect_default_ports(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.connect_inputs_to_system()?;
        self.connect_outputs_to_system()?;
        Ok(())
    }

    /// List all available JACK ports
    pub fn list_ports(&self) -> Vec<String> {
        if let Some(ref client) = self.client {
            client
                .as_client()
                .ports(None, None, jack::PortFlags::empty())
        } else {
            Vec::new()
        }
    }

    /// List capture (input) ports
    pub fn list_capture_ports(&self) -> Vec<String> {
        if let Some(ref client) = self.client {
            client
                .as_client()
                .ports(None, None, jack::PortFlags::IS_OUTPUT)
        } else {
            Vec::new()
        }
    }

    /// List playback (output) ports
    pub fn list_playback_ports(&self) -> Vec<String> {
        if let Some(ref client) = self.client {
            client
                .as_client()
                .ports(None, None, jack::PortFlags::IS_INPUT)
        } else {
            Vec::new()
        }
    }

    /// Connect a source port to a destination port by name
    pub fn connect_ports(
        &self,
        source: &str,
        dest: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref client) = self.client {
            client.as_client().connect_ports_by_name(source, dest)?;
            log::info!("JACK: Connected {} -> {}", source, dest);
        }
        Ok(())
    }

    /// Disconnect ports by name
    pub fn disconnect_ports(
        &self,
        source: &str,
        dest: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref client) = self.client {
            client.as_client().disconnect_ports_by_name(source, dest)?;
            log::info!("JACK: Disconnected {} from {}", source, dest);
        }
        Ok(())
    }
}

impl Drop for JackClient {
    fn drop(&mut self) {
        if self.available {
            log::info!("JACK client shutting down");
        }
    }
}

/// JACK router for managing port connections
pub struct JackRouter {
    connections: Vec<(String, String)>,
}

impl JackRouter {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
        }
    }

    pub fn add_connection(&mut self, from: String, to: String) {
        self.connections.push((from, to));
    }

    pub fn remove_connection(&mut self, from: &str, to: &str) {
        self.connections.retain(|(f, t)| f != from || t != to);
    }

    pub fn get_connections(&self) -> &[(String, String)] {
        &self.connections
    }

    pub fn apply(&self, client: &JackClient) -> Result<(), Box<dyn std::error::Error>> {
        for (from, to) in &self.connections {
            client.connect_ports(from, to)?;
        }
        Ok(())
    }
}

impl Default for JackRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jack_client_creation() {
        // This test will pass whether JACK is available or not
        let client = JackClient::new();
        assert!(client.is_ok());

        let client = client.unwrap();
        // Sample rate should be valid
        assert!(client.get_sample_rate() > 0);
        // Buffer size should be reasonable
        assert!(client.get_buffer_size() >= 32);
    }

    #[test]
    fn test_jack_router() {
        let mut router = JackRouter::new();
        router.add_connection(
            "system:capture_1".to_string(),
            "PhantomLink:input_L".to_string(),
        );
        router.add_connection(
            "system:capture_2".to_string(),
            "PhantomLink:input_R".to_string(),
        );

        assert_eq!(router.get_connections().len(), 2);

        router.remove_connection("system:capture_1", "PhantomLink:input_L");
        assert_eq!(router.get_connections().len(), 1);
    }
}
