use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};

pub struct AudioEngine {
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            input_stream: None,
            output_stream: None,
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

        // Input stream: capture audio
        let input_stream = input_device.build_input_stream(
            &input_config,
            move |data: &[f32], _| {
                // TODO: Process input audio (RNNoise, VST, mixing)
                // Send to output or channel strips
            },
            move |err| {
                eprintln!("Input stream error: {}", err);
            },
            None, // Option<Duration>
        ).expect("Failed to build input stream");

        // Output stream: play audio
        let output_stream = output_device.build_output_stream(
            &output_config,
            move |data: &mut [f32], _| {
                // TODO: Fill output buffer with mixed/processed audio
                for sample in data.iter_mut() {
                    *sample = 0.0; // Silence for now
                }
            },
            move |err| {
                eprintln!("Output stream error: {}", err);
            },
            None, // Option<Duration>
        ).expect("Failed to build output stream");

        input_stream.play().expect("Failed to start input stream");
        output_stream.play().expect("Failed to start output stream");

        self.input_stream = Some(input_stream);
        self.output_stream = Some(output_stream);
    }

    pub fn process_with_vst(&self, _channel: usize, _input: &[f32]) -> Vec<f32> {
        // Process audio through VST plugin for the given channel (stub)
        _input.to_vec()
    }
}