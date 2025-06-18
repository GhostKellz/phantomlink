use eframe::egui;
use crate::gui::ChannelStrip;
use crate::config::AppConfig;
use crate::audio::AudioEngine;
use crate::rnnoise::Rnnoise;
use crate::scarlett::ScarlettSolo;
use std::sync::{Arc, Mutex};

pub fn glow_button(text: &str, color: egui::Color32) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| {
        let desired_size = egui::vec2(120.0, 35.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            // Glow effect
            for i in 0..5 {
                let glow_radius = 3.0 + i as f32 * 2.0;
                let glow_alpha = 30 - i * 6;
                let glow_color = egui::Color32::from_rgba_premultiplied(
                    color.r(), color.g(), color.b(), glow_alpha
                );
                painter.rect_filled(
                    rect.expand(glow_radius),
                    egui::Rounding::same(8.0 + glow_radius),
                    glow_color,
                );
            }
            
            // Button background
            let bg_color = if response.hovered() {
                egui::Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 100)
            } else {
                egui::Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 60)
            };
            
            painter.rect_filled(rect, egui::Rounding::same(8.0), bg_color);
            painter.rect_stroke(rect, egui::Rounding::same(8.0), egui::Stroke::new(1.5, color));
            
            // Text
            let text_color = if response.hovered() {
                egui::Color32::WHITE
            } else {
                color
            };
            
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(14.0),
                text_color,
            );
        }
        
        response
    }
}

pub fn level_meter(ui: &mut egui::Ui, level: f32, height: f32) -> egui::Response {
    let desired_size = egui::vec2(12.0, height);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        
        // Background
        painter.rect_filled(
            rect,
            egui::Rounding::same(2.0),
            egui::Color32::from_rgba_premultiplied(20, 20, 30, 180),
        );
        
        // Level indicator
        let level_height = rect.height() * level.min(1.0);
        let level_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x, rect.max.y - level_height),
            egui::vec2(rect.width(), level_height),
        );
        
        // Color based on level
        let color = if level > 0.9 {
            egui::Color32::from_rgb(255, 50, 50)  // Red - clipping
        } else if level > 0.7 {
            egui::Color32::from_rgb(255, 200, 50) // Yellow - hot
        } else {
            egui::Color32::from_rgb(80, 217, 176) // Green - normal
        };
        
        painter.rect_filled(level_rect, egui::Rounding::same(2.0), color);
        
        // Peak indicators
        if level > 0.95 {
            for i in 0..3 {
                let peak_y = rect.min.y + i as f32 * 3.0;
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(rect.min.x, peak_y),
                        egui::vec2(rect.width(), 2.0),
                    ),
                    egui::Rounding::same(1.0),
                    egui::Color32::from_rgb(255, 100, 100),
                );
            }
        }
    }
    
    response
}

pub fn channel_strip(
    ui: &mut egui::Ui,
    strip: &mut ChannelStrip,
    channel_idx: usize,
    config: &mut AppConfig,
    audio_engine: &AudioEngine,
    vst_plugin_names: &[String],
) {
    let strip_width = 100.0;
    let strip_height = 400.0;
    
    ui.vertical(|ui| {
        ui.set_width(strip_width);
        
        // Channel label with glow
        ui.horizontal(|ui| {
            let title_color = if strip.recording {
                egui::Color32::from_rgb(255, 100, 100)
            } else if strip.solo {
                egui::Color32::from_rgb(255, 200, 100)
            } else {
                egui::Color32::from_rgb(80, 217, 176)
            };
            
            ui.colored_label(title_color, &strip.name);
        });
        
        ui.add_space(10.0);
        
        // Level meters (L/R)
        ui.horizontal(|ui| {
            level_meter(ui, strip.level_meter[0], 60.0);
            ui.add_space(2.0);
            level_meter(ui, strip.level_meter[1], 60.0);
        });
        
        ui.add_space(10.0);
        
        // Gain knob
        ui.label("GAIN");
        let gain_response = ui.add(
            egui::Slider::new(&mut strip.gain, -20.0..=20.0)
                .step_by(0.1)
                .suffix(" dB")
                .show_value(false)
        );
        if gain_response.changed() {
            // Update audio engine
        }
        
        ui.add_space(5.0);
        
        // Volume fader
        ui.label("VOLUME");
        let vol_response = ui.add(
            egui::Slider::new(&mut strip.volume, 0.0..=1.0)
                .vertical()
                .step_by(0.01)
                .show_value(false)
        );
        if vol_response.changed() {
            audio_engine.update_channel(channel_idx, strip.volume, strip.muted);
            config.update_channel(channel_idx, strip.volume, strip.muted, strip.selected_vst);
        }
        
        ui.add_space(10.0);
        
        // Pan knob
        ui.label("PAN");
        ui.add(
            egui::Slider::new(&mut strip.pan, -1.0..=1.0)
                .step_by(0.01)
                .show_value(false)
        );
        
        ui.add_space(10.0);
        
        // VST plugin selector
        ui.label("PLUGIN");
        egui::ComboBox::from_id_source(format!("vst_{}", channel_idx))
            .selected_text(
                strip.selected_vst
                    .and_then(|idx| vst_plugin_names.get(idx))
                    .map(|s| s.as_str())
                    .unwrap_or("None")
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut strip.selected_vst, None, "None");
                for (idx, name) in vst_plugin_names.iter().enumerate() {
                    ui.selectable_value(&mut strip.selected_vst, Some(idx), name);
                }
            });
        
        ui.add_space(10.0);
        
        // Control buttons
        ui.horizontal(|ui| {
            // Mute button
            let mute_color = if strip.muted {
                egui::Color32::from_rgb(255, 100, 100)
            } else {
                egui::Color32::from_rgb(100, 100, 100)
            };
            
            if ui.add(glow_button("M", mute_color)).clicked() {
                strip.muted = !strip.muted;
                audio_engine.update_channel(channel_idx, strip.volume, strip.muted);
                config.update_channel(channel_idx, strip.volume, strip.muted, strip.selected_vst);
            }
            
            // Solo button
            let solo_color = if strip.solo {
                egui::Color32::from_rgb(255, 200, 100)
            } else {
                egui::Color32::from_rgb(100, 100, 100)
            };
            
            if ui.add(glow_button("S", solo_color)).clicked() {
                strip.solo = !strip.solo;
            }
        });
        
        ui.horizontal(|ui| {
            // Record button
            let rec_color = if strip.recording {
                egui::Color32::from_rgb(255, 50, 50)
            } else {
                egui::Color32::from_rgb(100, 100, 100)
            };
            
            if ui.add(glow_button("R", rec_color)).clicked() {
                strip.recording = !strip.recording;
            }
        });
    });
}

pub fn master_section(
    ui: &mut egui::Ui,
    master_volume: &mut f32,
    master_muted: &mut bool,
    spectrum_data: &Arc<Mutex<Vec<f32>>>,
) {
    ui.vertical(|ui| {
        ui.set_width(150.0);
        
        // Master title
        ui.label(egui::RichText::new("MASTER").size(16.0).color(egui::Color32::from_rgb(80, 217, 176)));
        
        ui.add_space(10.0);
        
        // Spectrum analyzer
        spectrum_analyzer(ui, spectrum_data);
        
        ui.add_space(15.0);
        
        // Master volume
        ui.label("MASTER VOLUME");
        ui.add(
            egui::Slider::new(master_volume, 0.0..=1.0)
                .vertical()
                .step_by(0.01)
                .show_value(false)
        );
        
        ui.add_space(10.0);
        
        // Master mute
        let mute_color = if *master_muted {
            egui::Color32::from_rgb(255, 100, 100)
        } else {
            egui::Color32::from_rgb(100, 100, 100)
        };
        
        if ui.add(glow_button("MUTE", mute_color)).clicked() {
            *master_muted = !*master_muted;
        }
    });
}

pub fn spectrum_analyzer(ui: &mut egui::Ui, spectrum_data: &Arc<Mutex<Vec<f32>>>) {
    let desired_size = egui::vec2(150.0, 100.0);
    let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        
        // Background
        painter.rect_filled(
            rect,
            egui::Rounding::same(4.0),
            egui::Color32::from_rgba_premultiplied(10, 15, 30, 200),
        );
        
        // Draw spectrum
        if let Ok(data) = spectrum_data.lock() {
            let bar_width = rect.width() / data.len() as f32;
            
            for (i, &magnitude) in data.iter().enumerate() {
                let bar_height = rect.height() * magnitude.min(1.0);
                let bar_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.min.x + i as f32 * bar_width, rect.max.y - bar_height),
                    egui::vec2(bar_width - 1.0, bar_height),
                );
                
                // Color based on frequency (low = red, mid = green, high = blue)
                let freq_ratio = i as f32 / data.len() as f32;
                let color = if freq_ratio < 0.3 {
                    egui::Color32::from_rgb(255, 100, 100) // Low frequencies - red
                } else if freq_ratio < 0.7 {
                    egui::Color32::from_rgb(80, 217, 176)  // Mid frequencies - green
                } else {
                    egui::Color32::from_rgb(100, 150, 255) // High frequencies - blue
                };
                
                painter.rect_filled(bar_rect, egui::Rounding::same(1.0), color);
            }
        }
        
        // Grid lines
        painter.rect_stroke(rect, egui::Rounding::same(4.0), egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 217, 176)));
    }
}

pub fn effects_panel(
    ui: &mut egui::Ui,
    rnnoise: &mut Rnnoise,
    config: &mut AppConfig,
    audio_engine: &AudioEngine,
) {
    ui.collapsing("EFFECTS", |ui| {
        ui.horizontal(|ui| {
            ui.label("RNNoise:");
            let mut enabled = rnnoise.is_enabled();
            let text = if enabled { "ON" } else { "OFF" };
            if ui.toggle_value(&mut enabled, text).changed() {
                if enabled {
                    rnnoise.enable();
                } else {
                    rnnoise.disable();
                }
                audio_engine.set_rnnoise_enabled(enabled);
                config.rnnoise_enabled = enabled;
            }
        });
        
        // Add more effects here
        ui.separator();
        ui.label("More effects coming soon...");
    });
}

pub fn hardware_panel(
    ui: &mut egui::Ui,
    scarlett: &Option<ScarlettSolo>,
    scarlett_error: &Option<String>,
    scarlett_gain: &mut f32,
    scarlett_monitor: &mut bool,
    config: &mut AppConfig,
) {
    ui.collapsing("HARDWARE", |ui| {
        if let Some(scarlett_dev) = scarlett {
            ui.label("Scarlett Solo Connected");
            
            // Input gain
            if ui.add(egui::Slider::new(scarlett_gain, 0.0..=1.0).text("Input Gain")).changed() {
                let _ = scarlett_dev.set_input_gain(*scarlett_gain);
                config.scarlett_gain = *scarlett_gain;
            }
            
            // Direct monitor
            if ui.checkbox(scarlett_monitor, "Direct Monitor").changed() {
                let _ = scarlett_dev.set_direct_monitor(*scarlett_monitor);
                config.scarlett_monitor = *scarlett_monitor;
            }
        } else if let Some(error) = scarlett_error {
            ui.colored_label(egui::Color32::RED, error);
        } else {
            ui.label("No Scarlett device detected");
        }
    });
}