use realfft::{RealFftPlanner, RealToComplex};
use num_complex::Complex;
use std::sync::{Arc, Mutex};
use eframe::egui;

pub struct SpectrumAnalyzer {
    fft: Arc<dyn RealToComplex<f32>>,
    buffer: Vec<f32>,
    spectrum: Vec<f32>,
    window: Vec<f32>,
    sample_rate: f32,
}

impl SpectrumAnalyzer {
    pub fn new(sample_rate: f32) -> Self {
        Self::new_with_size(1024, sample_rate)
    }
    
    pub fn new_with_size(fft_size: usize, sample_rate: f32) -> Self {
        let mut planner = RealFftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(fft_size);
        
        // Hann window for better frequency resolution
        let window: Vec<f32> = (0..fft_size)
            .map(|i| {
                let n = i as f32 / (fft_size - 1) as f32;
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * n).cos())
            })
            .collect();
        
        Self {
            fft,
            buffer: vec![0.0; fft_size],
            spectrum: vec![0.0; fft_size / 2 + 1],
            window,
            sample_rate,
        }
    }
    
    pub fn process(&mut self, input: &[f32]) -> &[f32] {
        if input.len() != self.buffer.len() {
            return &self.spectrum;
        }
        
        // Apply window function
        for (i, &sample) in input.iter().enumerate() {
            self.buffer[i] = sample * self.window[i];
        }
        
        // Perform FFT
        let mut spectrum_complex = vec![Complex::new(0.0, 0.0); self.spectrum.len()];
        self.fft.process(&mut self.buffer, &mut spectrum_complex).unwrap();
        
        // Convert to magnitude and apply logarithmic scaling
        for (i, complex) in spectrum_complex.iter().enumerate() {
            let magnitude = complex.norm();
            let db = 20.0 * magnitude.log10().max(-60.0); // Minimum -60dB
            self.spectrum[i] = ((db + 60.0) / 60.0).max(0.0); // Normalize to 0-1
        }
        
        &self.spectrum
    }
    
    pub fn get_frequency_bins(&self) -> Vec<f32> {
        (0..self.spectrum.len())
            .map(|i| i as f32 * self.sample_rate / (2.0 * self.spectrum.len() as f32))
            .collect()
    }
    
    /// Update spectrum with new data
    pub fn update(&mut self, spectrum_data: &[f32]) {
        if spectrum_data.len() == self.spectrum.len() {
            self.spectrum.copy_from_slice(spectrum_data);
        } else if !spectrum_data.is_empty() {
            // Resample if needed
            let spectrum_len = self.spectrum.len();
            for (i, spectrum_val) in self.spectrum.iter_mut().enumerate() {
                let src_idx = (i * spectrum_data.len()) / spectrum_len;
                *spectrum_val = spectrum_data.get(src_idx).copied().unwrap_or(0.0);
            }
        }
    }
    
    /// Render spectrum analyzer with theme colors
    pub fn render(&self, ui: &mut egui::Ui, theme: &crate::gui::theme::WavelinkTheme) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
        let rect = response.rect;
        
        if rect.width() > 0.0 && rect.height() > 0.0 {
            let bar_count = self.spectrum.len().min(64); // Limit for performance
            let bar_width = rect.width() / bar_count as f32;
            
            for (i, &magnitude) in self.spectrum.iter().take(bar_count).enumerate() {
                let x = rect.min.x + i as f32 * bar_width;
                let bar_height = magnitude * rect.height() * 0.8;
                let y = rect.max.y - bar_height;
                
                // Color gradient from green to yellow to red based on level
                let color = if magnitude < 0.6 {
                    egui::Color32::from_rgb(
                        (magnitude * 255.0) as u8,
                        255,
                        0
                    )
                } else if magnitude < 0.8 {
                    egui::Color32::from_rgb(
                        255,
                        (255.0 * (1.0 - magnitude)) as u8,
                        0
                    )
                } else {
                    egui::Color32::from_rgb(255, 0, 0)
                };
                
                let bar_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(x, y),
                    egui::Vec2::new(bar_width * 0.8, bar_height)
                );
                
                painter.rect_filled(bar_rect, egui::Rounding::same(1.0), color);
            }
            
            // Draw frequency labels
            let font_id = egui::FontId::proportional(10.0);
            painter.text(
                egui::Pos2::new(rect.min.x + 4.0, rect.max.y - 16.0),
                egui::Align2::LEFT_BOTTOM,
                "20Hz",
                font_id.clone(),
                theme.text_muted
            );
            painter.text(
                egui::Pos2::new(rect.max.x - 4.0, rect.max.y - 16.0),
                egui::Align2::RIGHT_BOTTOM,
                "20kHz",
                font_id,
                theme.text_muted
            );
        }
    }
}

pub struct VUMeter {
    peak_level: f32,
    rms_level: f32,
    peak_hold_time: f32,
    peak_decay_rate: f32,
    rms_window: Vec<f32>,
    window_index: usize,
}

impl VUMeter {
    pub fn new(window_size: usize) -> Self {
        Self {
            peak_level: 0.0,
            rms_level: 0.0,
            peak_hold_time: 0.0,
            peak_decay_rate: 0.99, // Peak decay per frame
            rms_window: vec![0.0; window_size],
            window_index: 0,
        }
    }
    
    pub fn process(&mut self, input: &[f32], dt: f32) -> (f32, f32) {
        let mut peak: f32 = 0.0;
        let mut rms_sum = 0.0;
        
        // Find peak and calculate RMS
        for &sample in input {
            let abs_sample = sample.abs();
            peak = peak.max(abs_sample);
            rms_sum += sample * sample;
        }
        
        // Update peak with hold and decay
        if peak > self.peak_level {
            self.peak_level = peak;
            self.peak_hold_time = 0.5; // Hold for 500ms
        } else {
            self.peak_hold_time -= dt;
            if self.peak_hold_time <= 0.0 {
                self.peak_level *= self.peak_decay_rate;
            }
        }
        
        // Update RMS with rolling average
        let rms_current = (rms_sum / input.len() as f32).sqrt();
        self.rms_window[self.window_index] = rms_current;
        self.window_index = (self.window_index + 1) % self.rms_window.len();
        
        self.rms_level = self.rms_window.iter().sum::<f32>() / self.rms_window.len() as f32;
        
        (self.peak_level, self.rms_level)
    }
}