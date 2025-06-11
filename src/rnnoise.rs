pub struct Rnnoise {
    enabled: bool,
}

impl Rnnoise {
    pub fn new() -> Self {
        Self { enabled: false }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    // Placeholder for actual RNNoise processing
    pub fn process(&self, _input: &[f32]) -> Vec<f32> {
        // In a real implementation, call into rnnoise here
        _input.to_vec()
    }
}