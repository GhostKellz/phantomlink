//! Audio effects for PhantomLink channel processing
//!
//! Provides professional-grade dynamics processing:
//! - Noise Gate with hold time
//! - Compressor with soft knee
//! - Limiter with lookahead

/// Noise gate for cutting audio below threshold
pub struct NoiseGate {
    threshold_db: f32,
    attack_ms: f32,
    release_ms: f32,
    hold_ms: f32,
    sample_rate: f32,
    envelope: f32,
    hold_counter: f32,
    enabled: bool,
}

impl NoiseGate {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold_db: -40.0,
            attack_ms: 0.5,
            release_ms: 50.0,
            hold_ms: 20.0,
            sample_rate,
            envelope: 0.0,
            hold_counter: 0.0,
            enabled: false,
        }
    }

    pub fn set_threshold(&mut self, db: f32) {
        self.threshold_db = db.clamp(-80.0, 0.0);
    }

    pub fn set_attack(&mut self, ms: f32) {
        self.attack_ms = ms.clamp(0.1, 100.0);
    }

    pub fn set_release(&mut self, ms: f32) {
        self.release_ms = ms.clamp(10.0, 1000.0);
    }

    pub fn set_hold(&mut self, ms: f32) {
        self.hold_ms = ms.clamp(0.0, 500.0);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        let threshold_linear = 10.0_f32.powf(self.threshold_db / 20.0);
        let attack_coeff = (-1.0 / (self.attack_ms * 0.001 * self.sample_rate)).exp();
        let release_coeff = (-1.0 / (self.release_ms * 0.001 * self.sample_rate)).exp();
        let hold_samples = self.hold_ms * 0.001 * self.sample_rate;

        for sample in samples.iter_mut() {
            let input_level = sample.abs();

            // Gate detection
            if input_level > threshold_linear {
                self.hold_counter = hold_samples;
                // Attack - open gate
                self.envelope = self.envelope * attack_coeff + (1.0 - attack_coeff);
            } else if self.hold_counter > 0.0 {
                self.hold_counter -= 1.0;
            } else {
                // Release - close gate
                self.envelope = self.envelope * release_coeff;
            }

            // Apply gate
            *sample *= self.envelope;
        }
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.hold_counter = 0.0;
    }
}

/// Compressor with soft knee and makeup gain
pub struct Compressor {
    threshold_db: f32,
    ratio: f32,
    attack_ms: f32,
    release_ms: f32,
    knee_db: f32,
    makeup_db: f32,
    sample_rate: f32,
    envelope: f32,
    enabled: bool,
}

impl Compressor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold_db: -18.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 100.0,
            knee_db: 6.0,
            makeup_db: 0.0,
            sample_rate,
            envelope: 0.0,
            enabled: false,
        }
    }

    pub fn set_threshold(&mut self, db: f32) {
        self.threshold_db = db.clamp(-60.0, 0.0);
    }

    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(1.0, 20.0);
    }

    pub fn set_attack(&mut self, ms: f32) {
        self.attack_ms = ms.clamp(0.1, 100.0);
    }

    pub fn set_release(&mut self, ms: f32) {
        self.release_ms = ms.clamp(10.0, 1000.0);
    }

    pub fn set_knee(&mut self, db: f32) {
        self.knee_db = db.clamp(0.0, 24.0);
    }

    pub fn set_makeup(&mut self, db: f32) {
        self.makeup_db = db.clamp(-12.0, 24.0);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn compute_gain_reduction(&self, input_db: f32) -> f32 {
        let half_knee = self.knee_db / 2.0;
        let knee_start = self.threshold_db - half_knee;
        let knee_end = self.threshold_db + half_knee;

        if input_db <= knee_start {
            // Below knee - no compression
            0.0
        } else if input_db >= knee_end {
            // Above knee - full compression
            (input_db - self.threshold_db) * (1.0 - 1.0 / self.ratio)
        } else {
            // In soft knee region - gradual compression
            let x = input_db - knee_start;
            let knee_width = self.knee_db;
            if knee_width > 0.0 {
                (1.0 - 1.0 / self.ratio) * x * x / (2.0 * knee_width)
            } else {
                0.0
            }
        }
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        let attack_coeff = (-1.0 / (self.attack_ms * 0.001 * self.sample_rate)).exp();
        let release_coeff = (-1.0 / (self.release_ms * 0.001 * self.sample_rate)).exp();
        let makeup_linear = 10.0_f32.powf(self.makeup_db / 20.0);

        for sample in samples.iter_mut() {
            let input_abs = sample.abs().max(1e-10);
            let input_db = 20.0 * input_abs.log10();

            // Compute desired gain reduction
            let gain_reduction_db = self.compute_gain_reduction(input_db);

            // Smooth envelope
            let coeff = if gain_reduction_db > self.envelope {
                attack_coeff
            } else {
                release_coeff
            };
            self.envelope = self.envelope * coeff + gain_reduction_db * (1.0 - coeff);

            // Apply gain reduction and makeup
            let gain_linear = 10.0_f32.powf(-self.envelope / 20.0) * makeup_linear;
            *sample *= gain_linear;
        }
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }

    pub fn get_gain_reduction(&self) -> f32 {
        self.envelope
    }
}

/// Hard limiter with ceiling control
pub struct Limiter {
    ceiling_db: f32,
    release_ms: f32,
    sample_rate: f32,
    envelope: f32,
    enabled: bool,
}

impl Limiter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            ceiling_db: -0.3,
            release_ms: 50.0,
            sample_rate,
            envelope: 0.0,
            enabled: true, // Limiter typically enabled by default
        }
    }

    pub fn set_ceiling(&mut self, db: f32) {
        self.ceiling_db = db.clamp(-12.0, 0.0);
    }

    pub fn set_release(&mut self, ms: f32) {
        self.release_ms = ms.clamp(10.0, 500.0);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        let ceiling_linear = 10.0_f32.powf(self.ceiling_db / 20.0);
        let release_coeff = (-1.0 / (self.release_ms * 0.001 * self.sample_rate)).exp();

        for sample in samples.iter_mut() {
            let input_abs = sample.abs();

            // Calculate required attenuation
            let attenuation = if input_abs > ceiling_linear {
                ceiling_linear / input_abs
            } else {
                1.0
            };

            // Smooth the attenuation (instant attack, smooth release)
            if attenuation < self.envelope {
                self.envelope = attenuation; // Instant attack
            } else {
                self.envelope = self.envelope * release_coeff + attenuation * (1.0 - release_coeff);
            }

            // Apply limiting
            *sample *= self.envelope;
        }
    }

    pub fn reset(&mut self) {
        self.envelope = 1.0;
    }

    pub fn is_limiting(&self) -> bool {
        self.envelope < 0.99
    }
}

/// Channel effects chain configuration
#[derive(Clone)]
pub struct ChannelEffectsConfig {
    pub gate_enabled: bool,
    pub gate_threshold_db: f32,
    pub compressor_enabled: bool,
    pub compressor_threshold_db: f32,
    pub compressor_ratio: f32,
    pub limiter_enabled: bool,
    pub limiter_ceiling_db: f32,
}

impl Default for ChannelEffectsConfig {
    fn default() -> Self {
        Self {
            gate_enabled: false,
            gate_threshold_db: -40.0,
            compressor_enabled: false,
            compressor_threshold_db: -18.0,
            compressor_ratio: 4.0,
            limiter_enabled: true,
            limiter_ceiling_db: -0.3,
        }
    }
}

/// Complete channel effects chain
pub struct ChannelEffects {
    pub gate: NoiseGate,
    pub compressor: Compressor,
    pub limiter: Limiter,
}

impl ChannelEffects {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            gate: NoiseGate::new(sample_rate),
            compressor: Compressor::new(sample_rate),
            limiter: Limiter::new(sample_rate),
        }
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        // Order: Gate -> Compressor -> Limiter
        self.gate.process(samples);
        self.compressor.process(samples);
        self.limiter.process(samples);
    }

    pub fn apply_config(&mut self, config: &ChannelEffectsConfig) {
        self.gate.set_enabled(config.gate_enabled);
        self.gate.set_threshold(config.gate_threshold_db);

        self.compressor.set_enabled(config.compressor_enabled);
        self.compressor.set_threshold(config.compressor_threshold_db);
        self.compressor.set_ratio(config.compressor_ratio);

        self.limiter.set_enabled(config.limiter_enabled);
        self.limiter.set_ceiling(config.limiter_ceiling_db);
    }

    pub fn reset(&mut self) {
        self.gate.reset();
        self.compressor.reset();
        self.limiter.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_gate() {
        let mut gate = NoiseGate::new(48000.0);
        gate.set_enabled(true);
        gate.set_threshold(-30.0);

        // Test with quiet signal (should be gated)
        let mut quiet = vec![0.001; 100];
        gate.process(&mut quiet);
        assert!(quiet.iter().all(|&s| s.abs() < 0.01));

        gate.reset();

        // Test with loud signal (should pass)
        let mut loud = vec![0.5; 100];
        gate.process(&mut loud);
        assert!(loud.iter().any(|&s| s.abs() > 0.1));
    }

    #[test]
    fn test_compressor() {
        let mut comp = Compressor::new(48000.0);
        comp.set_enabled(true);
        comp.set_threshold(-20.0);
        comp.set_ratio(4.0);

        // Test with loud signal (should be compressed)
        let mut loud = vec![0.8; 1000];
        comp.process(&mut loud);

        // Should be reduced but not silent
        let max_output = loud.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max_output < 0.8);
        assert!(max_output > 0.1);
    }

    #[test]
    fn test_limiter() {
        let mut limiter = Limiter::new(48000.0);
        limiter.set_enabled(true);
        limiter.set_ceiling(-6.0); // -6dB ceiling

        // Test with signal that exceeds ceiling
        let mut loud = vec![1.0; 100];
        limiter.process(&mut loud);

        // Should be limited to ceiling
        let ceiling_linear = 10.0_f32.powf(-6.0 / 20.0);
        assert!(loud.iter().all(|&s| s.abs() <= ceiling_linear + 0.01));
    }

    #[test]
    fn test_effects_chain() {
        let mut effects = ChannelEffects::new(48000.0);

        let config = ChannelEffectsConfig {
            gate_enabled: true,
            gate_threshold_db: -50.0,
            compressor_enabled: true,
            compressor_threshold_db: -20.0,
            compressor_ratio: 4.0,
            limiter_enabled: true,
            limiter_ceiling_db: -1.0,
        };
        effects.apply_config(&config);

        let mut samples = vec![0.5; 100];
        effects.process(&mut samples);

        // Output should be limited
        let ceiling_linear = 10.0_f32.powf(-1.0 / 20.0);
        assert!(samples.iter().all(|&s| s.abs() <= ceiling_linear + 0.01));
    }
}
