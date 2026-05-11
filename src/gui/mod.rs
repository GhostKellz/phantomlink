pub mod applications;
mod enhanced_methods;
pub mod mixer;
pub mod theme;
pub mod visualizer;
pub mod waveform;
pub mod widgets;

use crate::advanced_denoising::{DenoisingMetrics, DenoisingMode};
use crate::audio::AudioEngine;
use crate::config::MicrophonePreset;
use crate::ghostwave_integration::{
    DenoiserBackend, DriverInfo, GhostWaveIntegration, LatencyMode, PhantomLinkProfile,
    StatusHealth, detect_nvidia_driver,
};
use crate::gui::applications::ApplicationManager;
use crate::gui::mixer::MixerPanel;
use crate::gui::theme::{ThemePreset, WavelinkTheme};
use crate::gui::visualizer::SpectrumAnalyzer;
use crate::gui::widgets::{
    GlowButtonStyle, ModernChannelStrip, StatusIndicator, enhanced_glow_button,
};
use crate::jack_client::JackClient;
use crate::phantomlink;
use crate::scarlett::{AirMode, CaptureSource, InputLevel, LevelMeters, ScarlettSolo};
use eframe::egui;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum MainTab {
    #[default]
    Mixer,
    Applications,
    Advanced,
    Settings,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NotificationMessage {
    pub text: String,
    pub level: NotificationLevel,
    pub timestamp: std::time::Instant,
    pub duration: std::time::Duration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

pub struct PhantomlinkApp {
    vst_plugins: Vec<std::path::PathBuf>,
    vst_plugin_info: Vec<phantomlink::VstPluginInfo>,
    channel_strips: [ModernChannelStrip; 4],
    scarlett: Option<ScarlettSolo>,
    scarlett_phantom_power: bool,
    scarlett_air_mode: AirMode,
    scarlett_input_level: InputLevel,
    scarlett_direct_monitor: bool,
    audio_engine: AudioEngine,
    audio_started: bool,
    // Theme system
    theme: WavelinkTheme,
    theme_preset: ThemePreset,
    // GhostWave AI denoising
    ghostwave: Option<GhostWaveIntegration>,
    ghostwave_profile: PhantomLinkProfile,
    ghostwave_strength: f32,
    ghostwave_latency_mode: LatencyMode,
    ghostwave_denoiser_backend: DenoiserBackend,
    driver_info: DriverInfo,
    // Legacy denoising state (fallback)
    #[allow(dead_code)] // Preserved for denoising settings panel
    current_denoising_mode: DenoisingMode,
    advanced_denoising_enabled: bool,
    show_denoising_metrics: bool,
    // Echo cancellation
    echo_cancellation_enabled: bool,
    // Scarlett DSP routing
    scarlett_level_meters: LevelMeters,
    scarlett_dsp1_source: CaptureSource,
    scarlett_dsp2_source: CaptureSource,
    show_dsp_routing: bool,
    // GUI Panels
    application_manager: ApplicationManager,
    mixer_panel: MixerPanel,
    spectrum_analyzer: SpectrumAnalyzer,
    // Tab state
    active_tab: MainTab,
    // Production-ready features
    keyboard_shortcuts_enabled: bool,
    notifications: Vec<NotificationMessage>,
    show_help_overlay: bool,
    auto_save_enabled: bool,
    last_save_time: std::time::Instant,
    master_volume: f32,
    mute_all: bool,
    // PipeWire audio preset
    pipewire_preset: enhanced_methods::PipeWirePreset,
    // Custom buffer size control
    custom_buffer_size: u32,
    use_custom_buffer: bool,
    // Microphone preset
    microphone_preset: MicrophonePreset,
    // PipeWire virtual device manager
    pipewire_device: Option<crate::pipewire::VirtualDeviceManager>,
    #[allow(dead_code)] // State tracking for PipeWire device status
    pipewire_device_active: bool,
    #[allow(dead_code)]
    solo_any: bool,
    // JACK Audio integration
    jack_client: Option<JackClient>,
    jack_processing_enabled: bool,
}

impl Default for PhantomlinkApp {
    fn default() -> Self {
        let scarlett = ScarlettSolo::new().ok();
        let vst_plugins = phantomlink::find_vst_plugins();
        let vst_plugin_info = phantomlink::scan_vst_plugins().unwrap_or_default();

        // Read Scarlett state if available
        let (phantom_power, air_mode, input_level, direct_monitor) = scarlett
            .as_ref()
            .map(|s| {
                (
                    s.get_phantom_power(),
                    s.get_air_mode(),
                    s.get_input_level(),
                    s.get_direct_monitor(),
                )
            })
            .unwrap_or((false, AirMode::Off, InputLevel::Line, false));

        // Initialize GhostWave if available
        let ghostwave = GhostWaveIntegration::new().ok();
        let driver_info = detect_nvidia_driver();

        // Load saved configuration
        let saved_config = crate::config::AppConfig::load();

        // Restore theme from config
        let theme_preset = match saved_config.theme.as_str() {
            "TokyoNightStorm" => ThemePreset::TokyoNightStorm,
            "TokyoNightMoon" => ThemePreset::TokyoNightMoon,
            _ => ThemePreset::TokyoNight,
        };
        let theme = WavelinkTheme::with_preset(theme_preset);

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
            scarlett_phantom_power: phantom_power,
            scarlett_air_mode: air_mode,
            scarlett_input_level: input_level,
            scarlett_direct_monitor: direct_monitor,
            audio_engine: AudioEngine::new(),
            audio_started: false,
            // Theme
            theme,
            theme_preset,
            // GhostWave (restore from saved config)
            ghostwave,
            ghostwave_profile: PhantomLinkProfile::default(),
            ghostwave_strength: saved_config.ghostwave.noise_strength.max(0.01),
            ghostwave_latency_mode: LatencyMode::default(),
            ghostwave_denoiser_backend: DenoiserBackend::default(),
            driver_info,
            // Legacy denoising
            current_denoising_mode: DenoisingMode::Enhanced,
            advanced_denoising_enabled: saved_config.ghostwave.enabled,
            show_denoising_metrics: saved_config.ghostwave.show_metrics,
            // Echo cancellation
            echo_cancellation_enabled: saved_config.echo_cancellation,
            // Scarlett DSP routing
            scarlett_level_meters: LevelMeters::default(),
            scarlett_dsp1_source: CaptureSource::Analogue1,
            scarlett_dsp2_source: CaptureSource::Analogue2,
            show_dsp_routing: false,
            // Panels
            application_manager: ApplicationManager::default(),
            mixer_panel: MixerPanel::default(),
            spectrum_analyzer: SpectrumAnalyzer::new(48000.0),
            active_tab: MainTab::default(),
            keyboard_shortcuts_enabled: true,
            notifications: Vec::new(),
            show_help_overlay: false,
            auto_save_enabled: true,
            last_save_time: std::time::Instant::now(),
            master_volume: 0.8,
            mute_all: false,
            pipewire_preset: enhanced_methods::PipeWirePreset::default(),
            // Custom buffer size control
            custom_buffer_size: 256,
            use_custom_buffer: false,
            // Microphone preset (default to Rode PodMic)
            microphone_preset: MicrophonePreset::default(),
            // Initialize PipeWire virtual device manager
            pipewire_device: if crate::pipewire::is_pipewire_running() {
                let mut mgr = crate::pipewire::VirtualDeviceManager::default();
                // Try to create virtual device on startup
                if mgr.create_virtual_device().is_ok() {
                    // Auto-link to Scarlett Solo if available
                    let _ = mgr.auto_link_source(Some("Scarlett"));
                    Some(mgr)
                } else {
                    None
                }
            } else {
                None
            },
            pipewire_device_active: false,
            solo_any: false,
            // JACK client (gracefully returns None if JACK server not running)
            jack_client: JackClient::new().ok(),
            jack_processing_enabled: true,
        }
    }
}

impl eframe::App for PhantomlinkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply the new Wavelink theme with green accents and translucency
        self.theme.apply(ctx);

        // Handle keyboard shortcuts
        self.handle_keyboard_shortcuts(ctx);

        // Auto-save functionality
        self.handle_auto_save();

        // Update notifications
        self.update_notifications();

        // Update channel telemetry from GhostWave and audio engine
        self.update_channel_telemetry();

        // Show help overlay if enabled
        if self.show_help_overlay {
            self.draw_help_overlay(ctx);
        }

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
                    .show(ui, |ui| match self.active_tab {
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
                        MainTab::Settings => {
                            self.draw_settings_panel(ui);
                        }
                    });

                // Draw floating notifications
                self.draw_notifications(ui);
            });

        // Request repaint for animations
        ctx.request_repaint();
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
                                .color(self.theme.green_primary),
                        );
                        ui.label(
                            egui::RichText::new("Professional Audio Mixer")
                                .size(14.0)
                                .color(self.theme.text_secondary),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Transport controls - more touch-friendly
                        ui.horizontal(|ui| {
                            // Audio engine control with enhanced styling
                            let button_text = if self.audio_started {
                                "⏹ STOP"
                            } else {
                                "▶ START"
                            };
                            let button_style = if self.audio_started {
                                GlowButtonStyle::Danger
                            } else {
                                GlowButtonStyle::Success
                            };

                            if ui
                                .add(enhanced_glow_button(button_text, &self.theme, button_style))
                                .clicked()
                            {
                                if self.audio_started {
                                    self.audio_engine.stop();
                                    self.audio_started = false;
                                    self.push_notification(
                                        "Audio engine stopped",
                                        NotificationLevel::Info,
                                    );
                                } else {
                                    match self.audio_engine.start() {
                                        Ok(()) => {
                                            self.audio_started = true;
                                            self.push_notification(
                                                "Audio engine started",
                                                NotificationLevel::Success,
                                            );
                                        }
                                        Err(e) => {
                                            self.push_notification(
                                                format!("Engine start failed: {}", e),
                                                NotificationLevel::Error,
                                            );
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

                // Toast notifications are drawn separately via draw_notifications()
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
                        (
                            MainTab::Mixer,
                            "🎛️ MIXER",
                            "Audio mixing and channel controls",
                        ),
                        (
                            MainTab::Applications,
                            "🎮 APPS",
                            "Application audio management",
                        ),
                        (
                            MainTab::Advanced,
                            "⚡ ADVANCED",
                            "Advanced features and settings",
                        ),
                        (
                            MainTab::Settings,
                            "⚙️ SETTINGS",
                            "Application settings and preferences",
                        ),
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
                            enhanced_glow_button(label, &self.theme, button_style),
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
            // GhostWave v0.2.0 Controls with Telemetry
            egui::Frame::none()
                .fill(self.theme.translucent_input_bg())
                .stroke(egui::Stroke::new(1.0, self.theme.accent_primary))
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(egui::Margin::same(16.0))
                .show(ui, |ui| {
                    ui.set_min_width(450.0);

                    // Header with status indicator
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("🔇 GhostWave v0.2.0")
                                .size(18.0)
                                .strong()
                                .color(self.theme.accent_primary),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Status health indicator
                            if let Some(ref gw) = self.ghostwave {
                                let health = gw.get_status_health();
                                let (color, icon) = match health {
                                    StatusHealth::Healthy => (self.theme.success, "✓"),
                                    StatusHealth::CpuOnly => (self.theme.info, "⚡"),
                                    StatusHealth::Warning => (self.theme.warning, "⚠"),
                                    StatusHealth::Disabled => (self.theme.text_muted, "○"),
                                };
                                ui.label(
                                    egui::RichText::new(format!("{} {}", icon, health.name()))
                                        .size(12.0)
                                        .color(color),
                                );
                            }

                            if ui
                                .small_button("📊")
                                .on_hover_text("Toggle Metrics")
                                .clicked()
                            {
                                self.show_denoising_metrics = !self.show_denoising_metrics;
                            }
                        });
                    });

                    ui.add_space(12.0);

                    // GPU info row
                    if let Some(ref gw) = self.ghostwave {
                        let rtx = gw.get_rtx_status();
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "GPU: {} • {}",
                                    rtx.gpu_name, rtx.precision
                                ))
                                .size(11.0)
                                .color(self.theme.text_secondary),
                            );
                        });
                        ui.add_space(8.0);
                    }

                    // Enable/Disable toggle
                    let mut advanced_enabled = self.advanced_denoising_enabled;
                    if ui
                        .add(egui::Checkbox::new(
                            &mut advanced_enabled,
                            "Enable RTX AI Denoising",
                        ))
                        .changed()
                    {
                        self.advanced_denoising_enabled = advanced_enabled;
                        self.audio_engine
                            .set_advanced_denoising_enabled(advanced_enabled);
                        if let Some(ref mut gw) = self.ghostwave {
                            gw.set_enabled(advanced_enabled);
                        }
                    }

                    if self.advanced_denoising_enabled {
                        ui.add_space(12.0);

                        // Profile selection
                        ui.label(
                            egui::RichText::new("Processing Profile:")
                                .size(13.0)
                                .color(self.theme.text_primary),
                        );
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            for profile in PhantomLinkProfile::all() {
                                let is_selected = self.ghostwave_profile == *profile;
                                if ui
                                    .add(egui::RadioButton::new(is_selected, profile.name()))
                                    .on_hover_text(profile.description())
                                    .clicked()
                                {
                                    self.ghostwave_profile = *profile;
                                    if let Some(ref mut gw) = self.ghostwave {
                                        let _ = gw.set_profile(*profile);
                                    }
                                }
                            }
                        });

                        ui.add_space(12.0);

                        // Latency mode selection (v0.2.0)
                        ui.label(
                            egui::RichText::new("Latency Mode:")
                                .size(13.0)
                                .color(self.theme.text_primary),
                        );
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            for mode in LatencyMode::all() {
                                let is_selected = self.ghostwave_latency_mode == *mode;
                                if ui
                                    .add(egui::RadioButton::new(is_selected, mode.name()))
                                    .on_hover_text(mode.description())
                                    .clicked()
                                {
                                    self.ghostwave_latency_mode = *mode;
                                    if let Some(ref mut gw) = self.ghostwave {
                                        gw.set_latency_mode(*mode);
                                    }
                                }
                            }
                        });

                        ui.add_space(12.0);

                        // Strength slider
                        ui.horizontal(|ui| {
                            ui.label("Strength:");
                            if ui
                                .add(
                                    egui::Slider::new(&mut self.ghostwave_strength, 0.0..=1.0)
                                        .show_value(true),
                                )
                                .changed()
                                && let Some(ref mut gw) = self.ghostwave
                            {
                                let _ = gw.set_noise_strength(self.ghostwave_strength);
                            }
                        });

                        // Telemetry panel
                        if self.show_denoising_metrics {
                            ui.add_space(12.0);
                            ui.separator();
                            ui.add_space(8.0);

                            ui.label(
                                egui::RichText::new("Processing Telemetry")
                                    .size(13.0)
                                    .strong()
                                    .color(self.theme.accent_secondary),
                            );
                            ui.add_space(4.0);

                            if let Some(ref gw) = self.ghostwave {
                                let metrics = gw.get_metrics();
                                let fallback = gw.get_gpu_fallback();

                                egui::Grid::new("ghostwave_metrics")
                                    .num_columns(2)
                                    .spacing([16.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Latency:");
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{:.2} ms",
                                                metrics.latency_ms
                                            ))
                                            .color(self.theme.text_primary),
                                        );
                                        ui.end_row();

                                        ui.label("Frames:");
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{}",
                                                metrics.frames_processed
                                            ))
                                            .color(self.theme.text_primary),
                                        );
                                        ui.end_row();

                                        ui.label("XRuns:");
                                        let xrun_color = if metrics.xruns > 0 {
                                            self.theme.warning
                                        } else {
                                            self.theme.success
                                        };
                                        ui.label(
                                            egui::RichText::new(format!("{}", metrics.xruns))
                                                .color(xrun_color),
                                        );
                                        ui.end_row();

                                        ui.label("GPU Status:");
                                        let status_color = if fallback.gpu_active {
                                            self.theme.success
                                        } else {
                                            self.theme.warning
                                        };
                                        ui.label(
                                            egui::RichText::new(fallback.status_string())
                                                .color(status_color),
                                        );
                                        ui.end_row();

                                        if metrics.voice_activity {
                                            ui.label("Voice:");
                                            ui.label(
                                                egui::RichText::new("Detected")
                                                    .color(self.theme.success),
                                            );
                                            ui.end_row();
                                        }
                                    });

                                // GPU fallback warning with restart button
                                if fallback.fallback_active {
                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new("⚠ GPU fallback active")
                                                .size(12.0)
                                                .color(self.theme.warning),
                                        );
                                        if ui.small_button("🔄 Restart GPU").clicked()
                                            && let Some(ref mut gw) = self.ghostwave
                                            && let Err(e) = gw.restart_gpu()
                                        {
                                            self.push_notification(
                                                format!("GPU restart failed: {}", e),
                                                NotificationLevel::Error,
                                            );
                                        }
                                    });
                                }
                            }
                        }
                    }

                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(
                            "RTX Voice-quality noise suppression • nvidia-open 545+",
                        )
                        .size(11.0)
                        .color(self.theme.text_muted),
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
                            .color(self.theme.green_primary),
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
        // Scarlett Solo 4th Gen Controls
        if self.scarlett.is_some() {
            egui::Frame::none()
                .fill(self.theme.translucent_input_bg())
                .stroke(egui::Stroke::new(
                    1.5,
                    egui::Color32::from_rgb(0xdc, 0x14, 0x3c),
                )) // Focusrite red
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(egui::Margin::same(20.0))
                .show(ui, |ui| {
                    // Header
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🎤").size(22.0));
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("Scarlett Solo 4th Gen")
                                    .size(18.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(0xdc, 0x14, 0x3c)),
                            );
                            if let Some(ref scarlett) = self.scarlett {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Card {} • Firmware {}",
                                        scarlett.get_card_num(),
                                        scarlett.get_firmware_version()
                                    ))
                                    .size(11.0)
                                    .color(self.theme.text_muted),
                                );
                            }
                        });
                    });

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(12.0);

                    // Two column layout
                    ui.horizontal_top(|ui| {
                        // Left column - XLR Input (Mic)
                        ui.vertical(|ui| {
                            ui.set_min_width(200.0);

                            ui.label(
                                egui::RichText::new("XLR Input (Rode PodMic)")
                                    .size(14.0)
                                    .strong()
                                    .color(self.theme.text_primary),
                            );

                            ui.add_space(10.0);

                            // 48V Phantom Power
                            ui.horizontal(|ui| {
                                let phantom_color = if self.scarlett_phantom_power {
                                    egui::Color32::from_rgb(0xff, 0x66, 0x00) // Orange when on
                                } else {
                                    self.theme.text_muted
                                };

                                ui.label(egui::RichText::new("⚡").size(16.0).color(phantom_color));

                                if ui
                                    .checkbox(&mut self.scarlett_phantom_power, "48V Phantom Power")
                                    .changed()
                                    && let Some(ref mut scarlett) = self.scarlett
                                    && let Err(e) =
                                        scarlett.set_phantom_power(self.scarlett_phantom_power)
                                {
                                    self.push_notification(
                                        format!("Phantom power: {}", e),
                                        NotificationLevel::Error,
                                    );
                                }
                            });

                            ui.add_space(8.0);

                            // Air Mode
                            ui.label(
                                egui::RichText::new("Air Mode:")
                                    .size(12.0)
                                    .color(self.theme.text_secondary),
                            );

                            ui.add_space(4.0);

                            ui.horizontal(|ui| {
                                for mode in AirMode::all() {
                                    let is_selected = self.scarlett_air_mode == *mode;

                                    let button = egui::Button::new(
                                        egui::RichText::new(mode.name()).size(11.0).color(
                                            if is_selected {
                                                self.theme.bg_dark
                                            } else {
                                                self.theme.text_secondary
                                            },
                                        ),
                                    )
                                    .fill(if is_selected {
                                        egui::Color32::from_rgb(0x00, 0x9e, 0xc3) // Focusrite teal
                                    } else {
                                        self.theme.input_bg
                                    })
                                    .rounding(egui::Rounding::same(4.0));

                                    if ui.add(button).on_hover_text(mode.description()).clicked() {
                                        self.scarlett_air_mode = *mode;
                                        if let Some(ref mut scarlett) = self.scarlett
                                            && let Err(e) = scarlett.set_air_mode(*mode)
                                        {
                                            self.push_notification(
                                                format!("Air mode error: {}", e),
                                                NotificationLevel::Error,
                                            );
                                        }
                                    }
                                }
                            });
                        });

                        ui.add_space(24.0);

                        // Right column - Line Input + Monitor
                        ui.vertical(|ui| {
                            ui.set_min_width(180.0);

                            ui.label(
                                egui::RichText::new("Line Input 1 (1/4\")")
                                    .size(14.0)
                                    .strong()
                                    .color(self.theme.text_primary),
                            );

                            ui.add_space(10.0);

                            // Input Level
                            ui.horizontal(|ui| {
                                ui.label("Level:");

                                let line_selected = self.scarlett_input_level == InputLevel::Line;
                                if ui.selectable_label(line_selected, "Line").clicked()
                                    && !line_selected
                                {
                                    self.scarlett_input_level = InputLevel::Line;
                                    if let Some(ref mut scarlett) = self.scarlett {
                                        let _ = scarlett.set_input_level(InputLevel::Line);
                                    }
                                }

                                let inst_selected =
                                    self.scarlett_input_level == InputLevel::Instrument;
                                if ui.selectable_label(inst_selected, "Inst").clicked()
                                    && !inst_selected
                                {
                                    self.scarlett_input_level = InputLevel::Instrument;
                                    if let Some(ref mut scarlett) = self.scarlett {
                                        let _ = scarlett.set_input_level(InputLevel::Instrument);
                                    }
                                }
                            });

                            ui.add_space(16.0);

                            // Direct Monitor
                            ui.label(
                                egui::RichText::new("Monitoring")
                                    .size(14.0)
                                    .strong()
                                    .color(self.theme.text_primary),
                            );

                            ui.add_space(8.0);

                            let monitor_color = if self.scarlett_direct_monitor {
                                self.theme.success
                            } else {
                                self.theme.text_muted
                            };

                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🎧").size(16.0).color(monitor_color));

                                if ui
                                    .checkbox(&mut self.scarlett_direct_monitor, "Direct Monitor")
                                    .changed()
                                    && let Some(ref mut scarlett) = self.scarlett
                                    && let Err(e) =
                                        scarlett.set_direct_monitor(self.scarlett_direct_monitor)
                                {
                                    self.push_notification(
                                        format!("Direct monitor error: {}", e),
                                        NotificationLevel::Error,
                                    );
                                }
                            });

                            ui.label(
                                egui::RichText::new("Zero-latency hardware monitoring")
                                    .size(10.0)
                                    .color(self.theme.text_muted),
                            );
                        });
                    });

                    // Hardware Level Meters Section
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("📊 Hardware Level Meters")
                                .size(14.0)
                                .strong()
                                .color(self.theme.text_primary),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("🔧").on_hover_text("DSP Routing").clicked() {
                                self.show_dsp_routing = !self.show_dsp_routing;
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // Read level meters from hardware
                    if let Some(ref scarlett) = self.scarlett
                        && let Ok(meters) = scarlett.read_level_meters()
                    {
                        self.scarlett_level_meters = meters;
                    }

                    // Display the 4 primary level meters (Input 1, Input 2, Mix A L, Mix A R)
                    let meter_channels = [
                        (0, "IN 1"),  // Analogue 1
                        (1, "IN 2"),  // Analogue 2
                        (4, "OUT L"), // Mix A L
                        (5, "OUT R"), // Mix A R
                    ];

                    ui.horizontal(|ui| {
                        for (idx, label) in &meter_channels {
                            ui.vertical(|ui| {
                                ui.set_min_width(50.0);
                                ui.label(
                                    egui::RichText::new(*label)
                                        .size(10.0)
                                        .color(self.theme.text_secondary),
                                );

                                let level = self.scarlett_level_meters.get_normalized(*idx);
                                let db = self.scarlett_level_meters.get_db(*idx);

                                // Vertical meter bar
                                let (response, painter) = ui.allocate_painter(
                                    egui::Vec2::new(12.0, 80.0),
                                    egui::Sense::hover(),
                                );

                                let rect = response.rect;
                                painter.rect_filled(rect, 2.0, self.theme.bg_highlight);

                                // Fill based on level
                                let fill_height = rect.height() * level;
                                let fill_rect = egui::Rect::from_min_max(
                                    egui::Pos2::new(rect.left(), rect.bottom() - fill_height),
                                    rect.max,
                                );

                                // Color gradient based on level
                                let color = if level > 0.9 {
                                    self.theme.error
                                } else if level > 0.7 {
                                    egui::Color32::from_rgb(0xff, 0xcc, 0x00) // Yellow
                                } else {
                                    self.theme.success
                                };

                                painter.rect_filled(fill_rect, 2.0, color);

                                // dB label
                                if db > -60.0 {
                                    ui.label(
                                        egui::RichText::new(format!("{:.0}", db))
                                            .size(9.0)
                                            .color(self.theme.text_muted),
                                    );
                                }
                            });
                            ui.add_space(4.0);
                        }
                    });

                    // DSP Routing Panel (collapsible)
                    if self.show_dsp_routing {
                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        ui.label(
                            egui::RichText::new("🔀 DSP Input Routing")
                                .size(12.0)
                                .strong()
                                .color(self.theme.accent_secondary),
                        );

                        ui.add_space(6.0);

                        // DSP Input 1 source selector
                        ui.horizontal(|ui| {
                            ui.label("DSP 1:");
                            egui::ComboBox::from_id_salt("dsp1_source")
                                .selected_text(self.scarlett_dsp1_source.name())
                                .show_ui(ui, |ui| {
                                    for source in CaptureSource::all() {
                                        if ui
                                            .selectable_value(
                                                &mut self.scarlett_dsp1_source,
                                                *source,
                                                source.name(),
                                            )
                                            .changed()
                                            && let Some(ref scarlett) = self.scarlett
                                        {
                                            let _ = scarlett.set_dsp_input(1, *source);
                                        }
                                    }
                                });
                        });

                        // DSP Input 2 source selector
                        ui.horizontal(|ui| {
                            ui.label("DSP 2:");
                            egui::ComboBox::from_id_salt("dsp2_source")
                                .selected_text(self.scarlett_dsp2_source.name())
                                .show_ui(ui, |ui| {
                                    for source in CaptureSource::all() {
                                        if ui
                                            .selectable_value(
                                                &mut self.scarlett_dsp2_source,
                                                *source,
                                                source.name(),
                                            )
                                            .changed()
                                            && let Some(ref scarlett) = self.scarlett
                                        {
                                            let _ = scarlett.set_dsp_input(2, *source);
                                        }
                                    }
                                });
                        });
                    }
                });
        } else {
            // No Scarlett detected
            egui::Frame::none()
                .fill(self.theme.translucent_input_bg())
                .stroke(egui::Stroke::new(1.0, self.theme.warning))
                .rounding(egui::Rounding::same(12.0))
                .inner_margin(egui::Margin::same(16.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("⚠")
                                .size(18.0)
                                .color(self.theme.warning),
                        );
                        ui.label(
                            egui::RichText::new("Scarlett Solo 4th Gen not detected")
                                .color(self.theme.text_secondary),
                        );
                    });
                });
        }
    }

    fn draw_mixer_panel(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(self.theme.translucent_panel_bg())
            .stroke(egui::Stroke::new(1.0, self.theme.bg_highlight))
            .rounding(egui::Rounding::same(12.0))
            .inner_margin(egui::Margin::same(20.0))
            .show(ui, |ui| {
                // --- INPUTS section ---
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("INPUTS")
                            .size(13.0)
                            .strong()
                            .color(self.theme.text_muted),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(ref gw) = self.ghostwave
                            && gw.is_enabled()
                        {
                            let health_color = match gw.get_status_health() {
                                StatusHealth::Healthy => self.theme.success,
                                StatusHealth::CpuOnly => self.theme.warning,
                                StatusHealth::Warning => self.theme.warning,
                                StatusHealth::Disabled => self.theme.text_muted,
                            };
                            ui.label(
                                egui::RichText::new("AI Denoise")
                                    .size(11.0)
                                    .color(health_color),
                            );
                        }
                    });
                });

                ui.add_space(12.0);

                // Channel strips row
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;

                    let channel_configs: [(usize, &str, egui::Color32); 4] = [
                        (0, "MIC 1", self.theme.accent_primary),
                        (1, "MIC 2", self.theme.accent_secondary),
                        (2, "LINE 1", self.theme.purple),
                        (3, "LINE 2", self.theme.magenta),
                    ];

                    for (i, name, badge_color) in channel_configs {
                        if let Some(levels) = self.audio_engine.get_channel_levels(i) {
                            self.channel_strips[i].levels = levels;
                        }

                        let response = self.channel_strips[i].show(
                            ui,
                            &self.theme,
                            name,
                            &self.vst_plugins,
                            &self.vst_plugin_info,
                        );

                        // Draw colored badge on the strip frame
                        let strip_rect = ui.min_rect();
                        if ui.is_rect_visible(strip_rect) {
                            let badge_rect = egui::Rect::from_min_size(
                                egui::pos2(strip_rect.right() - 8.0, strip_rect.top() + 4.0),
                                egui::vec2(4.0, 20.0),
                            );
                            ui.painter()
                                .rect_filled(badge_rect, egui::Rounding::same(2.0), badge_color);
                        }

                        if response.volume_changed
                            || response.gain_changed
                            || response.pan_changed
                            || response.mute_changed
                        {
                            self.audio_engine.update_channel_advanced(
                                i,
                                self.channel_strips[i].volume,
                                self.channel_strips[i].muted,
                                self.channel_strips[i].gain,
                                self.channel_strips[i].pan,
                            );
                        }
                    }

                    // Right side: spectrum + master
                    ui.add_space(16.0);
                    ui.vertical(|ui| {
                        ui.set_min_width(260.0);

                        // Compact spectrum analyzer
                        egui::Frame::none()
                            .fill(self.theme.vu_meter_bg())
                            .stroke(egui::Stroke::new(1.0, self.theme.bg_highlight))
                            .rounding(egui::Rounding::same(8.0))
                            .inner_margin(egui::Margin::same(8.0))
                            .show(ui, |ui| {
                                ui.set_min_size(egui::Vec2::new(240.0, 160.0));
                                if let Some(spectrum_data) =
                                    self.audio_engine.get_spectrum_data_vec()
                                {
                                    self.spectrum_analyzer.update(&spectrum_data);
                                }
                                self.spectrum_analyzer.render(ui, &self.theme);
                            });

                        ui.add_space(12.0);

                        // Master volume
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("MASTER")
                                    .size(10.0)
                                    .color(self.theme.text_muted),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.master_volume, 0.0..=1.0)
                                    .show_value(false),
                            );
                        });

                        ui.add_space(4.0);

                        // Mute all
                        let mute_style = if self.mute_all {
                            GlowButtonStyle::Danger
                        } else {
                            GlowButtonStyle::Secondary
                        };
                        if ui
                            .add(enhanced_glow_button(
                                if self.mute_all { "UNMUTE ALL" } else { "MUTE ALL" },
                                &self.theme,
                                mute_style,
                            ))
                            .clicked()
                        {
                            self.mute_all = !self.mute_all;
                            for strip in &mut self.channel_strips {
                                strip.muted = self.mute_all;
                            }
                        }
                    });
                });

                ui.add_space(20.0);

                // --- OUTPUTS section ---
                ui.label(
                    egui::RichText::new("OUTPUTS")
                        .size(13.0)
                        .strong()
                        .color(self.theme.text_muted),
                );

                ui.add_space(8.0);

                // Output rows (Wavelink-style)
                for (idx, output) in self.mixer_panel.outputs.iter_mut().enumerate() {
                    if idx >= 2 && !output.enabled {
                        continue; // Only show Monitor/Stream by default
                    }

                    egui::Frame::none()
                        .fill(self.theme.card_bg)
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(16.0, 10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Output label
                                let label_color = match idx {
                                    0 => self.theme.accent_primary,
                                    1 => self.theme.accent_secondary,
                                    _ => self.theme.purple,
                                };
                                let icon = match idx {
                                    0 => "🎧",
                                    1 => "📡",
                                    2 => "💬",
                                    _ => "🎙",
                                };

                                ui.label(
                                    egui::RichText::new(format!("{} {}", icon, output.name.to_uppercase()))
                                        .size(12.0)
                                        .strong()
                                        .color(label_color),
                                );

                                ui.add_space(16.0);

                                // Level bar (visual indicator)
                                let bar_rect = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width() - 180.0, 6.0),
                                    egui::Sense::hover(),
                                ).0;
                                if ui.is_rect_visible(bar_rect) {
                                    ui.painter().rect_filled(
                                        bar_rect,
                                        egui::Rounding::same(3.0),
                                        self.theme.bg_highlight,
                                    );
                                    let fill_width = bar_rect.width() * output.volume;
                                    let fill_rect = egui::Rect::from_min_size(
                                        bar_rect.min,
                                        egui::vec2(fill_width, bar_rect.height()),
                                    );
                                    ui.painter().rect_filled(
                                        fill_rect,
                                        egui::Rounding::same(3.0),
                                        label_color,
                                    );
                                }

                                ui.add_space(8.0);

                                // Volume slider (compact)
                                ui.add_sized(
                                    [100.0, 20.0],
                                    egui::Slider::new(&mut output.volume, 0.0..=1.0)
                                        .show_value(false),
                                );

                                // Enable toggle
                                let toggle_color = if output.enabled {
                                    label_color
                                } else {
                                    self.theme.text_muted
                                };
                                let toggle_response = ui.allocate_response(
                                    egui::vec2(24.0, 24.0),
                                    egui::Sense::click(),
                                );
                                if toggle_response.clicked() {
                                    output.enabled = !output.enabled;
                                }
                                if ui.is_rect_visible(toggle_response.rect) {
                                    ui.painter().circle_filled(
                                        toggle_response.rect.center(),
                                        8.0,
                                        toggle_color,
                                    );
                                }
                            });
                        });
                    ui.add_space(4.0);
                }
            });
    }

    #[allow(dead_code)] // UI helper for legacy denoising metrics display
    fn show_denoising_metrics_ui(&self, ui: &mut egui::Ui, metrics: &DenoisingMetrics) {
        ui.label(
            egui::RichText::new("Performance Metrics:")
                .size(13.0)
                .strong()
                .color(self.theme.green_primary),
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
                    .strong(),
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
                    .strong(),
            );
        });

        // Quality score
        if metrics.quality_score > 0.0 {
            ui.horizontal(|ui| {
                ui.label("Quality:");
                ui.label(
                    egui::RichText::new(format!("{:.0}%", metrics.quality_score * 100.0))
                        .color(self.theme.green_primary)
                        .strong(),
                );
            });
        }

        // Memory usage
        if metrics.memory_usage_mb > 0.0 {
            ui.horizontal(|ui| {
                ui.label("Memory:");
                ui.label(
                    egui::RichText::new(format!("{:.0}MB", metrics.memory_usage_mb))
                        .color(self.theme.text_primary),
                );
            });
        }
    }
}
