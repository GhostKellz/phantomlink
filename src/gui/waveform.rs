use eframe::egui;
use std::collections::VecDeque;

pub struct WaveformDisplay {
    samples: VecDeque<f32>,
    max_samples: usize,
    scale: f32,
    color: egui::Color32,
}

impl WaveformDisplay {
    pub fn new(max_samples: usize, color: egui::Color32) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
            scale: 1.0,
            color,
        }
    }
    
    pub fn add_sample(&mut self, sample: f32) {
        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }
    
    pub fn add_samples(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.add_sample(sample);
        }
    }
    
    pub fn render(&self, ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            // Background
            painter.rect_filled(
                rect,
                egui::Rounding::same(2.0),
                egui::Color32::from_rgba_premultiplied(10, 15, 30, 200),
            );
            
            // Draw waveform
            if self.samples.len() > 1 {
                let mut points = Vec::new();
                let center_y = rect.center().y;
                let amplitude = rect.height() * 0.4 * self.scale;
                
                for (i, &sample) in self.samples.iter().enumerate() {
                    let x = rect.min.x + (i as f32 / (self.max_samples - 1) as f32) * rect.width();
                    let y = center_y - (sample * amplitude);
                    points.push(egui::pos2(x, y));
                }
                
                // Draw the waveform line
                for window in points.windows(2) {
                    painter.line_segment(
                        [window[0], window[1]],
                        egui::Stroke::new(1.5, self.color),
                    );
                }
                
                // Draw center line
                painter.line_segment(
                    [egui::pos2(rect.min.x, center_y), egui::pos2(rect.max.x, center_y)],
                    egui::Stroke::new(0.5, egui::Color32::from_rgba_premultiplied(255, 255, 255, 50)),
                );
            }
            
            // Border
            painter.rect_stroke(
                rect,
                egui::Rounding::same(2.0),
                egui::Stroke::new(1.0, self.color),
            );
        }
        
        response
    }
    
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale.clamp(0.1, 5.0);
    }
    
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

pub struct MultiChannelWaveform {
    channels: Vec<WaveformDisplay>,
    channel_names: Vec<String>,
    show_labels: bool,
}

impl MultiChannelWaveform {
    pub fn new(channel_count: usize, max_samples: usize) -> Self {
        let colors = [
            egui::Color32::from_rgb(80, 217, 176),   // Mint green
            egui::Color32::from_rgb(19, 158, 209),   // Cyan blue
            egui::Color32::from_rgb(255, 120, 180),  // Pink
            egui::Color32::from_rgb(255, 200, 100),  // Yellow
            egui::Color32::from_rgb(150, 255, 150),  // Light green
            egui::Color32::from_rgb(255, 150, 255),  // Light purple
        ];
        
        let mut channels = Vec::new();
        let mut channel_names = Vec::new();
        
        for i in 0..channel_count {
            let color = colors[i % colors.len()];
            channels.push(WaveformDisplay::new(max_samples, color));
            channel_names.push(format!("CH {}", i + 1));
        }
        
        Self {
            channels,
            channel_names,
            show_labels: true,
        }
    }
    
    pub fn add_samples(&mut self, channel: usize, samples: &[f32]) {
        if channel < self.channels.len() {
            self.channels[channel].add_samples(samples);
        }
    }
    
    pub fn render(&self, ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
        
        if ui.is_rect_visible(rect) {
            let channel_height = rect.height() / self.channels.len() as f32;
            
            for (i, (channel, name)) in self.channels.iter().zip(&self.channel_names).enumerate() {
                let channel_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.min.x, rect.min.y + i as f32 * channel_height),
                    egui::vec2(rect.width(), channel_height - 2.0),
                );
                
                // Create a temporary UI for this channel
                let mut child_ui = ui.child_ui(channel_rect, *ui.layout());
                
                channel.render(&mut child_ui, channel_rect.size());
                
                // Draw channel label if enabled
                if self.show_labels {
                    let painter = ui.painter();
                    painter.text(
                        egui::pos2(channel_rect.min.x + 5.0, channel_rect.min.y + 5.0),
                        egui::Align2::LEFT_TOP,
                        name,
                        egui::FontId::proportional(10.0),
                        channel.color,
                    );
                }
            }
        }
        
        response
    }
    
    pub fn set_channel_name(&mut self, channel: usize, name: String) {
        if channel < self.channel_names.len() {
            self.channel_names[channel] = name;
        }
    }
    
    pub fn set_show_labels(&mut self, show: bool) {
        self.show_labels = show;
    }
    
    pub fn clear_all(&mut self) {
        for channel in &mut self.channels {
            channel.clear();
        }
    }
    
    pub fn set_scale(&mut self, scale: f32) {
        for channel in &mut self.channels {
            channel.set_scale(scale);
        }
    }
}

pub fn render_level_meter_advanced(
    ui: &mut egui::Ui,
    level: f32,
    peak: f32,
    size: egui::Vec2,
    orientation: egui::Direction,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
    
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        
        // Background
        painter.rect_filled(
            rect,
            egui::Rounding::same(2.0),
            egui::Color32::from_rgba_premultiplied(20, 20, 30, 180),
        );
        
        match orientation {
            egui::Direction::TopDown => {
                // Vertical meter
                let level_height = rect.height() * level.min(1.0);
                let peak_height = rect.height() * peak.min(1.0);
                
                // Level bar
                let level_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.min.x, rect.max.y - level_height),
                    egui::vec2(rect.width(), level_height),
                );
                
                // Color based on level
                let level_color = if level > 0.9 {
                    egui::Color32::from_rgb(255, 50, 50)
                } else if level > 0.7 {
                    egui::Color32::from_rgb(255, 200, 50)
                } else {
                    egui::Color32::from_rgb(80, 217, 176)
                };
                
                painter.rect_filled(level_rect, egui::Rounding::same(1.0), level_color);
                
                // Peak indicator
                if peak > 0.0 {
                    let peak_y = rect.max.y - peak_height;
                    painter.line_segment(
                        [egui::pos2(rect.min.x, peak_y), egui::pos2(rect.max.x, peak_y)],
                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                    );
                }
                
                // Scale marks
                for i in 0..5 {
                    let y = rect.min.y + (i as f32 / 4.0) * rect.height();
                    painter.line_segment(
                        [egui::pos2(rect.max.x - 3.0, y), egui::pos2(rect.max.x, y)],
                        egui::Stroke::new(0.5, egui::Color32::GRAY),
                    );
                }
            },
            egui::Direction::LeftToRight => {
                // Horizontal meter
                let level_width = rect.width() * level.min(1.0);
                let peak_width = rect.width() * peak.min(1.0);
                
                // Level bar
                let level_rect = egui::Rect::from_min_size(
                    rect.min,
                    egui::vec2(level_width, rect.height()),
                );
                
                let level_color = if level > 0.9 {
                    egui::Color32::from_rgb(255, 50, 50)
                } else if level > 0.7 {
                    egui::Color32::from_rgb(255, 200, 50)
                } else {
                    egui::Color32::from_rgb(80, 217, 176)
                };
                
                painter.rect_filled(level_rect, egui::Rounding::same(1.0), level_color);
                
                // Peak indicator
                if peak > 0.0 {
                    let peak_x = rect.min.x + peak_width;
                    painter.line_segment(
                        [egui::pos2(peak_x, rect.min.y), egui::pos2(peak_x, rect.max.y)],
                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                    );
                }
            },
            _ => {}
        }
        
        // Border
        painter.rect_stroke(
            rect,
            egui::Rounding::same(2.0),
            egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 217, 176)),
        );
    }
    
    response
}