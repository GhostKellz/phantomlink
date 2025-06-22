pub mod theme;
pub mod widgets;
pub mod visualizer;
pub mod mixer;
pub mod applications;
pub mod waveform;

use eframe::egui;
use crate::rnnoise::Rnnoise;
use crate::phantomlink;
use crate::scarlett::ScarlettSolo;
use crate::audio::AudioEngine;
use crate::gui::theme::WavelinkTheme;
use crate::gui::widgets::{ModernChannelStrip, ModernButton, StatusIndicator, glow_button};

pub struct PhantomlinkApp {
    rnnoise: Rnnoise,
    vst_plugins: Vec<std::path::PathBuf>,
    vst_plugin_info: Vec<phantomlink::VstPluginInfo>,
    channel_strips: [ModernChannelStrip; 4],
    scarlett: Option<ScarlettSolo>,
    scarlett_gain: f32,
    scarlett_monitor: bool,
    audio_engine: AudioEngine,
    audio_started: bool,
    error_message: Option<String>,
    theme: WavelinkTheme,
    show_advanced: bool,
}

impl Default for PhantomlinkApp {
    fn default() -> Self {
        let scarlett = ScarlettSolo::new().ok();
        let vst_plugins = phantomlink::find_vst_plugins();
        let vst_plugin_info = phantomlink::scan_vst_plugins().unwrap_or_default();
        
        Self {
            rnnoise: Rnnoise::new(),
            vst_plugins,
            vst_plugin_info,
            channel_strips: [
                ModernChannelStrip::new(),
                ModernChannelStrip::new(),
                ModernChannelStrip::new(),
                ModernChannelStrip::new(),
            ],
            scarlett,
            scarlett_gain: 0.5,
            scarlett_monitor: false,
            audio_engine: AudioEngine::new(),
            audio_started: false,
            error_message: None,
            theme: WavelinkTheme::new(),
            show_advanced: false,
        }
    }
}

impl eframe::App for PhantomlinkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply the new Wavelink theme with green accents and translucency
        self.theme.apply(ctx);
        
        // Main background with translucency
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(self.theme.translucent_deep_bg()))
            .show(ctx, |ui| {
                self.draw_header(ui);
                ui.add_space(12.0);
                
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        self.draw_mixer_panel(ui);
                        ui.add_space(16.0);
                        self.draw_control_sections(ui);
                    });
            });
    }
}

impl PhantomlinkApp {
    fn draw_header(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(self.theme.translucent_panel_bg())
            .stroke(egui::Stroke::new(1.5, self.theme.green_primary))
            .rounding(egui::Rounding::same(16.0))
            .inner_margin(egui::Margin::same(20.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Logo and title with green accent
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("PhantomLink")
                                .size(32.0)
                                .strong()
                                .color(self.theme.green_primary)
                        );
                        ui.label(
                            egui::RichText::new("Professional Audio Mixer")
                                .size(14.0)
                                .color(self.theme.text_secondary)
                        );
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Transport controls - more touch-friendly
                        ui.horizontal(|ui| {
                            // Audio engine control with glow effect
                            let button_text = if self.audio_started { "⏹ STOP" } else { "▶ START" };
                            
                            if ui.add(glow_button(button_text, self.theme.green_primary)).clicked() {
                                if self.audio_started {
                                    self.audio_engine.stop();
                                    self.audio_started = false;
                                    self.error_message = None;
                                } else {
                                    match self.audio_engine.start() {
                                        Ok(()) => {
                                            self.audio_started = true;
                                            self.error_message = None;
                                        }
                                        Err(e) => {
                                            self.error_message = Some(format!("Engine start failed: {}", e));
                                        }
                                    }
                                }
                            }
                            
                            ui.add_space(16.0);
                            
                            // Engine status
                            let (status_text, is_active) = if self.audio_started {
                                ("Engine Running", true)
                            } else {
                                ("Engine Stopped", false)
                            };
                            StatusIndicator::show(ui, &self.theme, status_text, is_active);
                        });
                    });
                });
                
                // Error message display
                if let Some(ref error) = self.error_message {
                    ui.add_space(8.0);
                    ui.colored_label(self.theme.error, format!("⚠ {}", error));
                }
            });
    }
    
    fn draw_mixer_panel(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(self.theme.translucent_panel_bg())
            .stroke(egui::Stroke::new(1.0, self.theme.medium_blue))
            .rounding(egui::Rounding::same(16.0))
            .inner_margin(egui::Margin::same(24.0))
            .show(ui, |ui| {
                // Mixer header
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("🎛️ AUDIO MIXER")
                            .size(22.0)
                            .strong()
                            .color(self.theme.green_primary)
                    );
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("4-Channel Professional Mixer")
                                .size(13.0)
                                .color(self.theme.text_secondary)
                        );
                    });
                });
                
                ui.add_space(20.0);
                
                // Channel strips with better spacing for touch
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 20.0;  // More spacing for touch
                    
                    for (i, channel_strip) in self.channel_strips.iter_mut().enumerate() {
                        // Update channel levels from audio engine if available
                        if let Some(levels) = self.audio_engine.get_channel_levels(i) {
                            channel_strip.levels = levels;
                        }
                        
                        let channel_name = match i {
                            0 => "MIC 1",
                            1 => "MIC 2", 
                            2 => "LINE 1",
                            3 => "LINE 2",
                            _ => "CHANNEL",
                        };
                        
                        let _response = channel_strip.show(
                            ui,
                            &self.theme,
                            channel_name,
                            &self.vst_plugins,
                            &self.vst_plugin_info,
                        );
                        
                        // TODO: Handle channel strip responses for audio engine updates
                    }
                });
            });
    }
    
    fn draw_control_sections(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_top(|ui| {
            // RNNoise Controls
            egui::Frame::none()
                .fill(self.theme.translucent_input_bg())
                .stroke(egui::Stroke::new(1.0, self.theme.light_blue))
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(egui::Margin::same(16.0))
                .show(ui, |ui| {
                    ui.set_min_width(300.0);
                    
                    ui.label(
                        egui::RichText::new("🔇 Noise Suppression")
                            .size(16.0)
                            .strong()
                            .color(self.theme.green_primary)
                    );
                    
                    ui.add_space(12.0);
                    
                    let mut rnnoise_enabled = self.rnnoise.is_enabled();
                    if ui.add(
                        egui::Checkbox::new(&mut rnnoise_enabled, "Enable RNNoise AI Noise Reduction")
                    ).changed() {
                        if rnnoise_enabled {
                            self.rnnoise.enable();
                        } else {
                            self.rnnoise.disable();
                        }
                    }
                    
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("Real-time AI-powered noise suppression")
                            .size(12.0)
                            .color(self.theme.text_muted)
                    );
                });
            
            ui.add_space(16.0);
            
            // Scarlett Solo Controls
            if let Some(ref _scarlett) = self.scarlett {
                egui::Frame::none()
                    .fill(self.theme.translucent_input_bg())
                    .stroke(egui::Stroke::new(1.0, self.theme.green_primary))
                    .rounding(egui::Rounding::same(12.0))
                    .inner_margin(egui::Margin::same(16.0))
                    .show(ui, |ui| {
                        ui.set_min_width(250.0);
                        
                        ui.label(
                            egui::RichText::new("🎤 Scarlett Solo")
                                .size(16.0)
                                .strong()
                                .color(self.theme.green_primary)
                        );
                        
                        ui.add_space(12.0);
                        
                        ui.horizontal(|ui| {
                            ui.label("Input Gain:");
                            ui.add(
                                egui::Slider::new(&mut self.scarlett_gain, 0.0..=1.0)
                                    .show_value(false)
                            );
                        });
                        
                        ui.add_space(8.0);
                        
                        ui.checkbox(&mut self.scarlett_monitor, "Direct Monitor");
                    });
            }
        });
    }
}
