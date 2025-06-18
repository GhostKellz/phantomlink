use nnnoiseless::DenoiseState;
use std::sync::Mutex;

pub struct Rnnoise {
    enabled: bool,
    denoiser: Mutex<Option<DenoiseState<'static>>>,
}

impl Rnnoise {
    pub fn new() -> Self {
        Self { 
            enabled: false,
            denoiser: Mutex::new(None),
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
        // Initialize denoiser when enabled
        if let Ok(mut denoiser) = self.denoiser.lock() {
            *denoiser = Some(*DenoiseState::new());
        }
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        // Clean up denoiser when disabled
        if let Ok(mut denoiser) = self.denoiser.lock() {
            *denoiser = None;
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn process(&self, input: &[f32]) -> Vec<f32> {
        if !self.enabled {
            return input.to_vec();
        }
        
        if let Ok(mut denoiser_guard) = self.denoiser.lock() {
            if let Some(ref mut denoiser) = *denoiser_guard {
                let mut output = vec![0.0f32; input.len()];
                
                // Process in chunks of 480 samples (RNNoise frame size)
                const FRAME_SIZE: usize = 480;
                
                for (input_chunk, output_chunk) in input.chunks(FRAME_SIZE).zip(output.chunks_mut(FRAME_SIZE)) {
                    if input_chunk.len() == FRAME_SIZE {
                        // Convert f32 to the format expected by nnnoiseless
                        let mut input_frame = [0.0f32; FRAME_SIZE];
                        let mut output_frame = [0.0f32; FRAME_SIZE];
                        input_frame[..input_chunk.len()].copy_from_slice(input_chunk);
                        
                        // Apply denoising
                        let _vad_prob = denoiser.process_frame(&mut output_frame, &input_frame);
                        
                        // Copy back to output
                        output_chunk.copy_from_slice(&output_frame[..output_chunk.len()]);
                    } else {
                        // Handle partial frames
                        output_chunk.copy_from_slice(input_chunk);
                    }
                }
                
                return output;
            }
        }
        
        // Fallback if denoiser is not available
        input.to_vec()
    }
}