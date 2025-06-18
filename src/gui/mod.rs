pub mod theme;
pub mod widgets;
pub mod visualizer;
pub mod mixer;
pub mod applications;
pub mod waveform;

use eframe::egui;
use crate::rnnoise::Rnnoise;
use crate::phantomlink;
use crate::vst_host::VstProcessor;
use crate::scarlett::ScarlettSolo;
use crate::audio::AudioEngine;
use crate::config::AppConfig;
use crate::jack_client::JackClient;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use rand::prelude::*;

use mixer::MixerPanel;
use applications::ApplicationManager;
use waveform::{MultiChannelWaveform, WaveformDisplay};

const CHANNELS: &[&str] = &["System", "Voice", "Game", "App"];

pub struct ChannelStrip {
    pub name: String,
    pub volume: f32,
    pub muted: bool,
    pub selected_vst: Option<usize>,
    pub gain: f32,
    pub pan: f32,
    pub solo: bool,
    pub recording: bool,
    pub level_meter: [f32; 2], // L/R levels
}

impl ChannelStrip {
    pub fn new(name: String) -> Self {
        Self {
            name,
            volume: 0.8,
            muted: false,
            selected_vst: None,
            gain: 0.0,
            pan: 0.0,
            solo: false,
            recording: false,
            level_meter: [0.0, 0.0],
        }
    }
}

pub struct PhantomlinkApp {
    rnnoise: Rnnoise,
    vst_plugins: Vec<PathBuf>,
    vst_plugin_names: Vec<String>,
    channel_strips: Vec<ChannelStrip>,
    scarlett_gain: f32,
    scarlett_monitor: bool,
    scarlett: Option<ScarlettSolo>,
    scarlett_error: Option<String>,
    audio_engine: AudioEngine,
    audio_running: bool,
    config: AppConfig,
    jack_client: Option<JackClient>,
    jack_enabled: bool,
    spectrum_data: Arc<Mutex<Vec<f32>>>,
    master_volume: f32,
    master_muted: bool,
    background_animation: f32,
    show_advanced: bool,
    theme: theme::SpaceTheme,
    
    // WaveLink-style features
    mixer_panel: MixerPanel,
    application_manager: ApplicationManager,
    waveform_display: MultiChannelWaveform,
    current_tab: AppTab,
    show_waveforms: bool,
    monitor_mix_enabled: bool,
    stream_mix_enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppTab {
    Mixer,
    Applications,
    Effects,
    Settings,
}

impl Default for PhantomlinkApp {
    fn default() -> Self {
        let config = AppConfig::load();
        
        let vst_plugins = phantomlink::find_vst_plugins();
        let vst_plugin_names: Vec<String> = vst_plugins
            .iter()
            .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
            .collect();
            
        let channel_strips: Vec<ChannelStrip> = CHANNELS.iter().enumerate().map(|(i, &name)| {
            let mut strip = ChannelStrip::new(name.to_string());
            strip.volume = config.get_channel_volume(i);
            strip.muted = config.get_channel_muted(i);
            strip.selected_vst = config.get_channel_plugin(i);
            strip
        }).collect();
        
        let (scarlett, scarlett_error) = match ScarlettSolo::new() {
            Ok(dev) => (Some(dev), None),
            Err(e) => (None, Some(format!("ScarlettSolo error: {}", e))),
        };
        
        let mut rnnoise = Rnnoise::new();
        if config.rnnoise_enabled {
            rnnoise.enable();
        }
        
        let jack_client = JackClient::new().ok();
        
        Self {
            rnnoise,
            vst_plugins,
            vst_plugin_names,
            channel_strips,
            scarlett_gain: config.scarlett_gain,
            scarlett_monitor: config.scarlett_monitor,
            scarlett,
            scarlett_error,
            audio_engine: AudioEngine::new(),
            audio_running: false,
            config,
            jack_client,
            jack_enabled: false,
            spectrum_data: Arc::new(Mutex::new(vec![0.0; 512])),
            master_volume: 0.85,
            master_muted: false,
            background_animation: 0.0,
            show_advanced: false,
            theme: theme::SpaceTheme::new(),
            
            // WaveLink-style features
            mixer_panel: MixerPanel::default(),
            application_manager: ApplicationManager::default(),
            waveform_display: MultiChannelWaveform::new(4, 512),
            current_tab: AppTab::Mixer,
            show_waveforms: true,
            monitor_mix_enabled: true,
            stream_mix_enabled: true,
        }
    }
}

impl eframe::App for PhantomlinkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply custom space theme
        self.theme.apply(ctx);
        
        // Update background animation
        self.background_animation += ctx.input(|i| i.unstable_dt);
        
        // Force continuous repaint for animations
        ctx.request_repaint();
        
        // Main UI
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                self.render_main_interface(ui, ctx);
            });
    }
}

impl PhantomlinkApp {
    fn render_main_interface(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Background with animated stars
        self.render_background(ui, ctx);
        
        ui.vertical(|ui| {
            // Header with tabs
            self.render_header_with_tabs(ui);
            
            ui.add_space(15.0);
            
            // Main content area based on selected tab
            match self.current_tab {
                AppTab::Mixer => self.render_mixer_tab(ui),
                AppTab::Applications => self.render_applications_tab(ui),
                AppTab::Effects => self.render_effects_tab(ui),
                AppTab::Settings => self.render_settings_tab(ui),
            }
            
            ui.add_space(15.0);
            
            // Waveform display (if enabled)
            if self.show_waveforms {
                self.render_waveform_section(ui);
                ui.add_space(10.0);
            }
            
            // Bottom status and controls
            self.render_bottom_status(ui);
        });
    }
    
    fn render_background(&self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let rect = ui.available_rect_before_wrap();
        let painter = ui.painter();
        
        // Dark space background with gradient
        let bg_color = egui::Color32::from_rgba_premultiplied(5, 8, 20, 240);
        painter.rect_filled(rect, 0.0, bg_color);
        
        // Animated stars
        let mut rng = StdRng::seed_from_u64(42); // Consistent seed for stable stars
        
        for _ in 0..150 {
            let x = rect.min.x + rng.r#gen::<f32>() * rect.width();
            let y = rect.min.y + rng.r#gen::<f32>() * rect.height();
            let size = rng.r#gen::<f32>() * 2.0 + 0.5;
            let alpha = ((self.background_animation * 2.0 + rng.r#gen::<f32>() * 6.28).sin() * 0.3 + 0.7).max(0.0);
            let color = egui::Color32::from_rgba_premultiplied(
                (80.0 * alpha) as u8,
                (217.0 * alpha) as u8,
                (176.0 * alpha) as u8,
                (255.0 * alpha) as u8,
            );
            painter.circle_filled(egui::pos2(x, y), size, color);
        }
        
        // Nebula-like effects
        for i in 0..5 {
            let x = rect.center().x + (self.background_animation * 0.1 + i as f32).sin() * 200.0;
            let y = rect.center().y + (self.background_animation * 0.15 + i as f32 * 2.0).cos() * 100.0;
            let radius = 60.0 + (self.background_animation * 0.2 + i as f32).sin() * 20.0;
            let alpha = 0.05 + (self.background_animation * 0.1 + i as f32).sin().abs() * 0.02;
            let color = egui::Color32::from_rgba_premultiplied(
                (19.0 * alpha * 255.0) as u8,
                (158.0 * alpha * 255.0) as u8,
                (209.0 * alpha * 255.0) as u8,
                (alpha * 50.0) as u8,
            );
            painter.circle_filled(egui::pos2(x, y), radius, color);
        }
    }
    
    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Phantom logo/title with glow effect
            let title_text = egui::RichText::new("⟨ PHANTOMLINK ⟩")
                .size(42.0)
                .color(egui::Color32::from_rgb(80, 217, 176));
            
            ui.add_space(20.0);
            ui.label(title_text);
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Status indicators
                let status_color = if self.audio_running {
                    egui::Color32::from_rgb(0, 255, 100)
                } else {
                    egui::Color32::from_rgb(255, 100, 100)
                };
                
                ui.colored_label(status_color, "●");
                ui.label(egui::RichText::new(if self.audio_running { "ONLINE" } else { "OFFLINE" })
                    .color(egui::Color32::from_rgb(150, 150, 150)));
                
                ui.add_space(20.0);
                
                if ui.button("⚙").clicked() {
                    self.show_advanced = !self.show_advanced;
                }
            });
        });
        
        ui.separator();
    }
    
    fn render_channel_strips(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for (i, strip) in self.channel_strips.iter_mut().enumerate() {
                widgets::channel_strip(ui, strip, i, &mut self.config, &self.audio_engine, &self.vst_plugin_names);
                ui.add_space(5.0);
            }
        });
    }
    
    fn render_master_section(&mut self, ui: &mut egui::Ui) {
        widgets::master_section(ui, &mut self.master_volume, &mut self.master_muted, &self.spectrum_data);
    }
    
    fn render_effects_section(&mut self, ui: &mut egui::Ui) {
        widgets::effects_panel(ui, &mut self.rnnoise, &mut self.config, &self.audio_engine);
    }
    
    fn render_hardware_section(&mut self, ui: &mut egui::Ui) {
        widgets::hardware_panel(ui, &self.scarlett, &self.scarlett_error, &mut self.scarlett_gain, &mut self.scarlett_monitor, &mut self.config);
    }
    
    fn render_header_with_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Phantom logo/title with glow effect
            let title_text = egui::RichText::new("⟨ PHANTOMLINK ⟩")
                .size(32.0)
                .color(egui::Color32::from_rgb(80, 217, 176));
            
            ui.add_space(20.0);
            ui.label(title_text);
            
            ui.add_space(40.0);
            
            // Tab buttons
            let mixer_color = if self.current_tab == AppTab::Mixer {
                egui::Color32::from_rgb(80, 217, 176)
            } else {
                egui::Color32::from_rgb(100, 100, 100)
            };
            
            if ui.add(widgets::glow_button("MIXER", mixer_color)).clicked() {
                self.current_tab = AppTab::Mixer;
            }
            
            let apps_color = if self.current_tab == AppTab::Applications {
                egui::Color32::from_rgb(80, 217, 176)
            } else {
                egui::Color32::from_rgb(100, 100, 100)
            };
            
            if ui.add(widgets::glow_button("APPLICATIONS", apps_color)).clicked() {
                self.current_tab = AppTab::Applications;
            }
            
            let effects_color = if self.current_tab == AppTab::Effects {
                egui::Color32::from_rgb(80, 217, 176)
            } else {
                egui::Color32::from_rgb(100, 100, 100)
            };
            
            if ui.add(widgets::glow_button("EFFECTS", effects_color)).clicked() {
                self.current_tab = AppTab::Effects;
            }
            
            let settings_color = if self.current_tab == AppTab::Settings {
                egui::Color32::from_rgb(80, 217, 176)
            } else {
                egui::Color32::from_rgb(100, 100, 100)
            };
            
            if ui.add(widgets::glow_button("SETTINGS", settings_color)).clicked() {
                self.current_tab = AppTab::Settings;
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Status indicators
                let status_color = if self.audio_running {
                    egui::Color32::from_rgb(0, 255, 100)
                } else {
                    egui::Color32::from_rgb(255, 100, 100)
                };
                
                ui.colored_label(status_color, "●");
                ui.label(egui::RichText::new(if self.audio_running { "ONLINE" } else { "OFFLINE" })
                    .color(egui::Color32::from_rgb(150, 150, 150)));
                
                ui.add_space(20.0);
                
                // Waveform toggle
                let wave_color = if self.show_waveforms {
                    egui::Color32::from_rgb(80, 217, 176)
                } else {
                    egui::Color32::from_rgb(100, 100, 100)
                };
                
                if ui.add(widgets::glow_button("WAVE", wave_color)).clicked() {
                    self.show_waveforms = !self.show_waveforms;
                }
            });
        });
        
        ui.separator();
    }
    
    fn render_mixer_tab(&mut self, ui: &mut egui::Ui) {
        let channel_names: Vec<String> = self.channel_strips.iter().map(|strip| strip.name.clone()).collect();
        self.mixer_panel.render(ui, &channel_names);
    }
    
    fn render_applications_tab(&mut self, ui: &mut egui::Ui) {
        self.application_manager.render(ui);
    }
    
    fn render_effects_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("AUDIO EFFECTS").size(16.0).color(egui::Color32::from_rgb(80, 217, 176)));
            ui.add_space(10.0);
            
            // RNNoise controls
            self.render_effects_section(ui);
            
            ui.add_space(15.0);
            
            // Channel strips with VST controls
            ui.label(egui::RichText::new("CHANNEL EFFECTS").size(14.0));
            ui.add_space(10.0);
            
            self.render_channel_strips(ui);
        });
    }
    
    fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("SYSTEM SETTINGS").size(16.0).color(egui::Color32::from_rgb(80, 217, 176)));
            ui.add_space(10.0);
            
            // Hardware section
            self.render_hardware_section(ui);
            
            ui.add_space(15.0);
            
            // Audio settings
            ui.label(egui::RichText::new("AUDIO ENGINE").size(14.0));
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.label("Master Volume:");
                ui.add(egui::Slider::new(&mut self.master_volume, 0.0..=1.0).step_by(0.01));
                
                ui.add_space(10.0);
                
                ui.checkbox(&mut self.master_muted, "Master Mute");
            });
            
            ui.add_space(10.0);
            
            // JACK controls
            ui.horizontal(|ui| {
                let jack_text = if self.jack_enabled { "JACK: ENABLED" } else { "JACK: DISABLED" };
                let jack_color = if self.jack_enabled {
                    egui::Color32::from_rgb(255, 150, 0)
                } else {
                    egui::Color32::from_rgb(100, 100, 100)
                };
                
                if ui.add(widgets::glow_button(jack_text, jack_color)).clicked() {
                    self.toggle_jack();
                }
            });
            
            ui.add_space(15.0);
            
            // Advanced settings
            ui.checkbox(&mut self.show_advanced, "Show Advanced Options");
            
            if self.show_advanced {
                ui.separator();
                ui.label("Advanced Settings");
                
                ui.horizontal(|ui| {
                    ui.label("VST Plugin Path:");
                    ui.label(format!("{} plugins found", self.vst_plugin_names.len()));
                });
            }
        });
    }
    
    fn render_waveform_section(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("WAVEFORM DISPLAY").size(14.0).color(egui::Color32::from_rgb(80, 217, 176)));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Clear").clicked() {
                        self.waveform_display.clear_all();
                    }
                    
                    ui.add_space(10.0);
                    
                    ui.label("Scale:");
                    let mut scale = 1.0; // TODO: Store scale in struct
                    if ui.add(egui::Slider::new(&mut scale, 0.1..=5.0).step_by(0.1)).changed() {
                        self.waveform_display.set_scale(scale);
                    }
                });
            });
            
            ui.add_space(5.0);
            
            // Waveform display
            self.waveform_display.render(ui, egui::vec2(ui.available_width(), 150.0));
        });
    }
    
    fn render_bottom_status(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Audio engine controls
            if !self.audio_running {
                if ui.add(widgets::glow_button("▶ START ENGINE", egui::Color32::from_rgb(0, 255, 100))).clicked() {
                    self.audio_engine.start();
                    self.audio_running = true;
                }
            } else {
                ui.add(widgets::glow_button("■ ENGINE RUNNING", egui::Color32::from_rgb(0, 200, 255)));
            }
            
            ui.add_space(20.0);
            
            // Mix status indicators
            let monitor_color = if self.monitor_mix_enabled {
                egui::Color32::from_rgb(0, 255, 100)
            } else {
                egui::Color32::from_rgb(100, 100, 100)
            };
            
            if ui.add(widgets::glow_button("MONITOR MIX", monitor_color)).clicked() {
                self.monitor_mix_enabled = !self.monitor_mix_enabled;
            }
            
            ui.add_space(5.0);
            
            let stream_color = if self.stream_mix_enabled {
                egui::Color32::from_rgb(0, 255, 100)
            } else {
                egui::Color32::from_rgb(100, 100, 100)
            };
            
            if ui.add(widgets::glow_button("STREAM MIX", stream_color)).clicked() {
                self.stream_mix_enabled = !self.stream_mix_enabled;
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(widgets::glow_button("SAVE CONFIG", egui::Color32::from_rgb(80, 217, 176))).clicked() {
                    let _ = self.config.save();
                }
            });
        });
    }
    
    fn render_bottom_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Audio engine controls
            if !self.audio_running {
                if ui.add(widgets::glow_button("▶ START ENGINE", egui::Color32::from_rgb(0, 255, 100))).clicked() {
                    self.audio_engine.start();
                    self.audio_running = true;
                }
            } else {
                ui.add(widgets::glow_button("■ ENGINE RUNNING", egui::Color32::from_rgb(0, 200, 255)));
            }
            
            ui.add_space(20.0);
            
            // JACK toggle
            let jack_text = if self.jack_enabled { "JACK: ON" } else { "JACK: OFF" };
            let jack_color = if self.jack_enabled {
                egui::Color32::from_rgb(255, 150, 0)
            } else {
                egui::Color32::from_rgb(100, 100, 100)
            };
            
            if ui.add(widgets::glow_button(jack_text, jack_color)).clicked() {
                self.toggle_jack();
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(widgets::glow_button("SAVE CONFIG", egui::Color32::from_rgb(80, 217, 176))).clicked() {
                    let _ = self.config.save();
                }
            });
        });
    }
    
    fn toggle_jack(&mut self) {
        if self.jack_enabled {
            self.jack_client = None;
            self.jack_enabled = false;
        } else {
            match JackClient::new() {
                Ok(client) => {
                    self.jack_client = Some(client);
                    self.jack_enabled = true;
                },
                Err(_) => {
                    // Show error in UI
                }
            }
        }
    }
}