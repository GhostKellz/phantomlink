pub mod theme;
pub mod widgets;
pub mod visualizer;
pub mod mixer;
pub mod applications;
pub mod waveform;

use eframe::egui;
use crate::phantomlink;
use crate::scarlett::ScarlettSolo;
use crate::audio::AudioEngine;
use crate::gui::theme::WavelinkTheme;
use crate::gui::widgets::{ModernChannelStrip, StatusIndicator, enhanced_glow_button, GlowButtonStyle};
use crate::gui::applications::ApplicationManager;
use crate::gui::mixer::MixerPanel;
use crate::gui::visualizer::SpectrumAnalyzer;
use crate::advanced_denoising::{DenoisingMode, DenoisingMetrics};

#[derive(Debug, Clone, PartialEq)]
pub enum MainTab {
    Mixer,
    Applications,
    Advanced,
}

impl Default for MainTab {
    fn default() -> Self {
        Self::Mixer
    }
}

pub struct PhantomlinkApp {
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
    // Advanced denoising state
    current_denoising_mode: DenoisingMode,
    advanced_denoising_enabled: bool,
    show_denoising_metrics: bool,
    // GUI Panels
    application_manager: ApplicationManager,
    mixer_panel: MixerPanel,
    spectrum_analyzer: SpectrumAnalyzer,
    // Tab state
    active_tab: MainTab,
}

impl Default for PhantomlinkApp {
    fn default() -> Self {
        let scarlett = ScarlettSolo::new().ok();
        let vst_plugins = phantomlink::find_vst_plugins();
        let vst_plugin_info = phantomlink::scan_vst_plugins().unwrap_or_default();
        
        Self {
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
            current_denoising_mode: DenoisingMode::Enhanced,
            advanced_denoising_enabled: true,
            show_denoising_metrics: false,
            application_manager: ApplicationManager::default(),
            mixer_panel: MixerPanel::default(),
            spectrum_analyzer: SpectrumAnalyzer::new(48000.0),
            active_tab: MainTab::default(),
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
                
                self.draw_navigation_tabs(ui);
                ui.add_space(12.0);
                
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        match self.active_tab {
                            MainTab::Mixer => {
                                self.draw_mixer_panel(ui);
                                ui.add_space(16.0);
                                self.draw_scarlett_controls(ui);
                            }
                            MainTab::Applications => {
                                self.draw_applications_panel(ui);
                            }
                            MainTab::Advanced => {
                                self.draw_advanced_panel(ui);
                            }
                        }
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
                            // Audio engine control with enhanced styling
                            let button_text = if self.audio_started { "⏹ STOP" } else { "▶ START" };
                            let button_style = if self.audio_started { 
                                GlowButtonStyle::Danger 
                            } else { 
                                GlowButtonStyle::Success 
                            };
                            
                            if ui.add(enhanced_glow_button(button_text, &self.theme, button_style)).clicked() {
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
    
    fn draw_navigation_tabs(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(self.theme.translucent_panel_bg())
            .stroke(egui::Stroke::new(1.0, self.theme.medium_blue))
            .rounding(egui::Rounding::same(12.0))
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    
                    let tabs = [
                        (MainTab::Mixer, "🎛️ MIXER", "Audio mixing and channel controls"),
                        (MainTab::Applications, "🎮 APPLICATIONS", "Application audio management"),
                        (MainTab::Advanced, "⚡ ADVANCED", "Advanced features and settings"),
                    ];
                    
                    for (tab, label, tooltip) in &tabs {
                        let is_active = self.active_tab == *tab;
                        let button_style = if is_active {
                            widgets::GlowButtonStyle::Success
                        } else {
                            widgets::GlowButtonStyle::Primary
                        };
                        
                        let button_response = ui.add_sized(
                            egui::Vec2::new(120.0, 40.0),
                            enhanced_glow_button(label, &self.theme, button_style)
                        );
                        
                        if button_response.on_hover_text(*tooltip).clicked() {
                            self.active_tab = tab.clone();
                        }
                    }
                });
            });
    }
    
    fn draw_applications_panel(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(self.theme.translucent_panel_bg())
            .stroke(egui::Stroke::new(1.0, self.theme.medium_blue))
            .rounding(egui::Rounding::same(16.0))
            .inner_margin(egui::Margin::same(24.0))
            .show(ui, |ui| {
                self.application_manager.render(ui);
            });
    }
    
    fn draw_advanced_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_top(|ui| {
            // Advanced Noise Suppression Controls
            egui::Frame::none()
                .fill(self.theme.translucent_input_bg())
                .stroke(egui::Stroke::new(1.0, self.theme.light_blue))
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(egui::Margin::same(16.0))
                .show(ui, |ui| {
                    ui.set_min_width(400.0);
                    
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("🔇 Advanced Noise Suppression")
                                .size(18.0)
                                .strong()
                                .color(self.theme.green_primary)
                        );
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("📊").on_hover_text("Show Metrics").clicked() {
                                self.show_denoising_metrics = !self.show_denoising_metrics;
                            }
                        });
                    });
                    
                    ui.add_space(16.0);
                    
                    // Enable/Disable toggle
                    let mut advanced_enabled = self.advanced_denoising_enabled;
                    if ui.add(
                        egui::Checkbox::new(&mut advanced_enabled, "Enable Advanced AI Denoising")
                    ).changed() {
                        self.advanced_denoising_enabled = advanced_enabled;
                        self.audio_engine.set_advanced_denoising_enabled(advanced_enabled);
                    }
                    
                    if self.advanced_denoising_enabled {
                        ui.add_space(12.0);
                        
                        // Denoising mode selection
                        ui.label(
                            egui::RichText::new("Denoising Mode:")
                                .size(14.0)
                                .color(self.theme.text_primary)
                        );
                        
                        ui.add_space(8.0);
                        
                        ui.horizontal(|ui| {
                            let modes = [
                                (DenoisingMode::Basic, "Basic", "Fast RNNoise only"),
                                (DenoisingMode::Enhanced, "Enhanced", "RNNoise + Deep Learning"),
                                (DenoisingMode::Maximum, "Maximum", "All denoising tiers")
                            ];
                            
                            for (mode, label, description) in &modes {
                                let is_selected = std::mem::discriminant(&self.current_denoising_mode) == std::mem::discriminant(mode);
                                
                                if ui.add(
                                    egui::RadioButton::new(is_selected, *label)
                                ).on_hover_text(*description).clicked() {
                                    self.current_denoising_mode = mode.clone();
                                    if let Err(e) = self.audio_engine.set_denoising_mode(mode.clone()) {
                                        self.error_message = Some(format!("Failed to set denoising mode: {}", e));
                                    }
                                }
                            }
                        });
                        
                        // Show metrics if enabled
                        if self.show_denoising_metrics {
                            ui.add_space(12.0);
                            ui.separator();
                            ui.add_space(12.0);
                            
                            if let Some(metrics) = self.audio_engine.get_denoising_metrics() {
                                self.show_denoising_metrics_ui(ui, &metrics);
                            } else {
                                ui.label(
                                    egui::RichText::new("Metrics unavailable")
                                        .size(12.0)
                                        .color(self.theme.text_muted)
                                );
                            }
                        }
                    }
                    
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("RTX Voice-like noise suppression for Linux")
                            .size(12.0)
                            .color(self.theme.text_muted)
                    );
                });
            
            ui.add_space(20.0);
            
            // Mixer panel in advanced view for routing matrix
            egui::Frame::none()
                .fill(self.theme.translucent_input_bg())
                .stroke(egui::Stroke::new(1.0, self.theme.green_primary))
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(egui::Margin::same(16.0))
                .show(ui, |ui| {
                    ui.set_min_width(300.0);
                    
                    ui.label(
                        egui::RichText::new("🎛️ Advanced Mixer")
                            .size(18.0)
                            .strong()
                            .color(self.theme.green_primary)
                    );
                    
                    ui.add_space(16.0);
                    
                    let channel_names = vec![
                        "MIC 1".to_string(),
                        "MIC 2".to_string(),
                        "LINE 1".to_string(),
                        "LINE 2".to_string(),
                    ];
                    
                    self.mixer_panel.render(ui, &channel_names);
                });
        });
    }
    
    fn draw_scarlett_controls(&mut self, ui: &mut egui::Ui) {
        // Scarlett Solo Controls
        if let Some(ref _scarlett) = self.scarlett {
            egui::Frame::none()
                .fill(self.theme.translucent_input_bg())
                .stroke(egui::Stroke::new(1.0, self.theme.green_primary))
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(egui::Margin::same(16.0))
                .show(ui, |ui| {
                    ui.set_max_width(400.0);
                    
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
                
                // Main content area with channel strips and spectrum analyzer
                ui.horizontal_top(|ui| {
                    // Channel strips section
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Channel Strips")
                                .size(14.0)
                                .strong()
                                .color(self.theme.text_primary)
                        );
                        
                        ui.add_space(8.0);
                        
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
                    
                    ui.add_space(32.0);
                    
                    // Real-time spectrum analyzer
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("🌊 Spectrum Analyzer")
                                .size(14.0)
                                .strong()
                                .color(self.theme.green_primary)
                        );
                        
                        ui.add_space(8.0);
                        
                        // Spectrum analyzer display
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 64))
                            .stroke(egui::Stroke::new(1.0, self.theme.green_primary))
                            .rounding(egui::Rounding::same(8.0))
                            .inner_margin(egui::Margin::same(12.0))
                            .show(ui, |ui| {
                                ui.set_min_size(egui::Vec2::new(280.0, 200.0));
                                
                                // Update spectrum data from audio engine
                                if let Some(spectrum_data) = self.audio_engine.get_spectrum_data_vec() {
                                    self.spectrum_analyzer.update(&spectrum_data);
                                }
                                
                                self.spectrum_analyzer.render(ui, &self.theme);
                            });
                        
                        ui.add_space(12.0);
                        
                        // Master controls
                        ui.label(
                            egui::RichText::new("Master Controls")
                                .size(12.0)
                                .strong()
                                .color(self.theme.text_secondary)
                        );
                        
                        ui.add_space(8.0);
                        
                        ui.horizontal(|ui| {
                            ui.label("Master Volume:");
                            ui.add(egui::Slider::new(&mut 0.8f32, 0.0..=1.0).show_value(false));
                        });
                        
                        ui.add_space(4.0);
                        
                        ui.horizontal(|ui| {
                            if ui.small_button("🔇 MUTE ALL").clicked() {
                                // TODO: Mute all channels
                            }
                            if ui.small_button("🔊 UNMUTE ALL").clicked() {
                                // TODO: Unmute all channels
                            }
                        });
                    });
                });
            });
    }
    
    fn show_denoising_metrics_ui(&self, ui: &mut egui::Ui, metrics: &DenoisingMetrics) {
        ui.label(
            egui::RichText::new("Performance Metrics:")
                .size(13.0)
                .strong()
                .color(self.theme.green_primary)
        );
        
        ui.add_space(4.0);
        
        // Latency indicator
        let latency_color = if metrics.latency_ms < 20.0 {
            self.theme.green_primary
        } else if metrics.latency_ms < 50.0 {
            egui::Color32::YELLOW
        } else {
            egui::Color32::RED
        };
        
        ui.horizontal(|ui| {
            ui.label("Latency:");
            ui.label(
                egui::RichText::new(format!("{:.1}ms", metrics.latency_ms))
                    .color(latency_color)
                    .strong()
            );
        });
        
        // CPU usage indicator
        let cpu_color = if metrics.cpu_usage_percent < 25.0 {
            self.theme.green_primary
        } else if metrics.cpu_usage_percent < 50.0 {
            egui::Color32::YELLOW
        } else {
            egui::Color32::RED
        };
        
        ui.horizontal(|ui| {
            ui.label("CPU Usage:");
            ui.label(
                egui::RichText::new(format!("{:.1}%", metrics.cpu_usage_percent))
                    .color(cpu_color)
                    .strong()
            );
        });
        
        // Quality score
        if metrics.quality_score > 0.0 {
            ui.horizontal(|ui| {
                ui.label("Quality:");
                ui.label(
                    egui::RichText::new(format!("{:.0}%", metrics.quality_score * 100.0))
                        .color(self.theme.green_primary)
                        .strong()
                );
            });
        }
        
        // Memory usage
        if metrics.memory_usage_mb > 0.0 {
            ui.horizontal(|ui| {
                ui.label("Memory:");
                ui.label(
                    egui::RichText::new(format!("{:.0}MB", metrics.memory_usage_mb))
                        .color(self.theme.text_primary)
                );
            });
        }
    }
}
