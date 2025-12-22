// Additional methods for the enhanced PhantomlinkApp
use eframe::egui;
use crate::gui::theme::{WavelinkTheme, ThemePreset};
use crate::config::MicrophonePreset;
use crate::gui::widgets::{enhanced_glow_button, GlowButtonStyle};
use crate::gui::NotificationLevel;
use crate::gui::MainTab;
use crate::ghostwave_integration::PhantomLinkProfile;

// Re-export PipeWirePreset from pipewire module
pub use crate::pipewire::PipeWirePreset;

impl super::PhantomlinkApp {
    pub fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        if !self.keyboard_shortcuts_enabled {
            return;
        }

        ctx.input(|i| {
            // F1 - Help overlay
            if i.key_pressed(egui::Key::F1) {
                self.show_help_overlay = !self.show_help_overlay;
            }

            // Ctrl+S - Save configuration
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
                self.save_configuration();
            }

            // Space - Start/Stop audio engine
            if i.key_pressed(egui::Key::Space) && !i.modifiers.ctrl {
                if self.audio_started {
                    self.audio_engine.stop();
                    self.audio_started = false;
                } else if let Ok(()) = self.audio_engine.start() {
                    self.audio_started = true;
                }
            }

            // M - Toggle mute all (without Ctrl for quick access)
            if i.key_pressed(egui::Key::M) && !i.modifiers.ctrl && !i.modifiers.alt {
                self.mute_all = !self.mute_all;
                self.add_notification(
                    if self.mute_all { "All channels muted" } else { "All channels unmuted" },
                    NotificationLevel::Info
                );
            }

            // Ctrl+M - Also toggle mute (legacy)
            if i.modifiers.ctrl && i.key_pressed(egui::Key::M) {
                self.mute_all = !self.mute_all;
                self.add_notification(
                    if self.mute_all { "All channels muted" } else { "All channels unmuted" },
                    NotificationLevel::Info
                );
            }

            // G - Toggle GhostWave processing
            if i.key_pressed(egui::Key::G) && !i.modifiers.ctrl && !i.modifiers.alt {
                if let Some(ref mut gw) = self.ghostwave {
                    let enabled = !gw.is_enabled();
                    gw.set_enabled(enabled);
                    self.advanced_denoising_enabled = enabled;
                    self.add_notification(
                        if enabled { "GhostWave enabled" } else { "GhostWave disabled" },
                        NotificationLevel::Info
                    );
                }
            }

            // I - Toggle metrics info panel
            if i.key_pressed(egui::Key::I) && !i.modifiers.ctrl && !i.modifiers.alt {
                self.show_denoising_metrics = !self.show_denoising_metrics;
                self.add_notification(
                    if self.show_denoising_metrics { "Metrics shown" } else { "Metrics hidden" },
                    NotificationLevel::Info
                );
            }

            // Ctrl+, (Comma) - Open Settings tab
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Comma) {
                self.active_tab = MainTab::Settings;
                self.add_notification("Settings", NotificationLevel::Info);
            }

            // 1-4 - Quick tab switching
            if i.key_pressed(egui::Key::Num1) && !i.modifiers.ctrl {
                self.active_tab = MainTab::Mixer;
            }
            if i.key_pressed(egui::Key::Num2) && !i.modifiers.ctrl {
                self.active_tab = MainTab::Applications;
            }
            if i.key_pressed(egui::Key::Num3) && !i.modifiers.ctrl {
                self.active_tab = MainTab::Advanced;
            }
            if i.key_pressed(egui::Key::Num4) && !i.modifiers.ctrl {
                self.active_tab = MainTab::Settings;
            }

            // Escape - Close help overlay
            if i.key_pressed(egui::Key::Escape) {
                self.show_help_overlay = false;
            }
        });
    }

    pub fn handle_auto_save(&mut self) {
        if !self.auto_save_enabled {
            return;
        }

        let now = std::time::Instant::now();
        if now.duration_since(self.last_save_time).as_secs() >= 300 {
            self.save_configuration();
            self.last_save_time = now;
        }
    }

    pub fn save_configuration(&mut self) {
        self.add_notification("Configuration saved", NotificationLevel::Success);
    }

    pub fn update_notifications(&mut self) {
        let now = std::time::Instant::now();
        self.notifications.retain(|notification| {
            now.duration_since(notification.timestamp) < notification.duration
        });
    }

    /// Update channel strip telemetry from GhostWave and audio engine metrics
    pub fn update_channel_telemetry(&mut self) {
        // Get GhostWave metrics if available
        let gw_metrics = self.audio_engine.get_ghostwave_metrics();
        let gw_fallback = self.audio_engine.get_ghostwave_fallback_status();
        let rtx_active = self.audio_engine.is_rtx_active();
        let gw_enabled = self.audio_engine.is_ghostwave_enabled();

        // Read Scarlett hardware level meters (if available)
        // Channels 0-1 are analog inputs, 2-3 are PCM outputs
        if let Some(ref scarlett) = self.scarlett {
            if let Ok(meters) = scarlett.read_level_meters() {
                self.scarlett_level_meters = meters;
            }
        }

        // Update telemetry for each channel strip
        for (i, strip) in self.channel_strips.iter_mut().enumerate() {
            // Update from GhostWave metrics
            if let Some(ref metrics) = gw_metrics {
                strip.telemetry.latency_ms = metrics.latency_ms;
                strip.telemetry.xruns = metrics.xruns;
                strip.telemetry.ghostwave_active = gw_enabled;
                strip.telemetry.rtx_active = rtx_active;
            }

            // Update GPU fallback info
            if let Some(ref fallback) = gw_fallback {
                if fallback.fallback_active {
                    strip.telemetry.rtx_active = false;
                }
            }

            // Check if VST is active on this channel
            strip.telemetry.vst_active = strip.selected_vst.is_some();

            // Update audio levels - prefer Scarlett hardware meters for channels 0-1
            if i < 2 && self.scarlett.is_some() {
                // Use Scarlett hardware level meters for input channels
                let peak = self.scarlett_level_meters.get_normalized(i);
                let rms = peak * 0.707; // Approximate RMS from peak
                strip.levels = [peak, rms];
            } else if let Some(levels) = self.audio_engine.get_channel_levels(i) {
                // Use software audio engine levels for other channels
                strip.levels = levels;
            }
        }
    }

    pub fn add_notification(&mut self, text: &str, level: NotificationLevel) {
        use super::NotificationMessage;
        let duration = match level {
            NotificationLevel::Error => std::time::Duration::from_secs(10),
            NotificationLevel::Warning => std::time::Duration::from_secs(7),
            NotificationLevel::Success => std::time::Duration::from_secs(3),
            NotificationLevel::Info => std::time::Duration::from_secs(5),
        };

        self.notifications.push(NotificationMessage {
            text: text.to_string(),
            level,
            timestamp: std::time::Instant::now(),
            duration,
        });
    }

    pub fn draw_notifications(&self, ui: &mut egui::Ui) {
        for (i, notification) in self.notifications.iter().enumerate() {
            let y_offset = i as f32 * 60.0 + 20.0;

            egui::Area::new(format!("notification_{}", i).into())
                .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-20.0, y_offset))
                .show(ui.ctx(), |ui| {
                    let (bg_color, text_color) = match notification.level {
                        NotificationLevel::Error => (self.theme.error, self.theme.text_primary),
                        NotificationLevel::Warning => (self.theme.warning, self.theme.deep_blue),
                        NotificationLevel::Success => (self.theme.success, self.theme.deep_blue),
                        NotificationLevel::Info => (self.theme.info, self.theme.text_primary),
                    };

                    egui::Frame::none()
                        .fill(bg_color)
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(&notification.text)
                                .color(text_color)
                                .size(14.0));
                        });
                });
        }
    }

    pub fn draw_help_overlay(&mut self, ctx: &egui::Context) {
        if !self.show_help_overlay {
            return;
        }

        egui::Window::new("💡 Keyboard Shortcuts")
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.set_min_width(450.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⌨️").size(24.0));
                    ui.label(egui::RichText::new("Keyboard Shortcuts")
                        .size(18.0)
                        .strong());
                });

                ui.separator();
                ui.add_space(10.0);

                let shortcuts = [
                    ("F1", "Toggle this help dialog"),
                    ("Space", "Start/Stop audio engine"),
                    ("M", "Mute/Unmute all channels"),
                    ("G", "Toggle GhostWave AI denoising"),
                    ("I", "Toggle metrics info panel"),
                    ("1-4", "Switch tabs (Mixer/Apps/Advanced/Settings)"),
                    ("Ctrl+,", "Open Settings"),
                    ("Ctrl+S", "Save configuration"),
                    ("Esc", "Close dialogs"),
                ];

                egui::Grid::new("shortcuts_grid")
                    .num_columns(2)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        for (key, description) in &shortcuts {
                            ui.label(egui::RichText::new(*key)
                                .family(egui::FontFamily::Monospace)
                                .size(12.0)
                                .color(egui::Color32::from_rgb(100, 200, 255)));
                            ui.label(*description);
                            ui.end_row();
                        }
                    });

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("✓ Got it!").clicked() {
                        self.show_help_overlay = false;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.checkbox(&mut self.keyboard_shortcuts_enabled, "Enable shortcuts");
                    });
                });
            });
    }

    pub fn draw_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_top(|ui| {
            // Left column - Theme & Interface
            egui::Frame::none()
                .fill(self.theme.translucent_panel_bg())
                .stroke(egui::Stroke::new(1.0, self.theme.accent_primary))
                .rounding(egui::Rounding::same(16.0))
                .inner_margin(egui::Margin::same(20.0))
                .show(ui, |ui| {
                    ui.set_min_width(380.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🎨")
                            .size(22.0)
                            .color(self.theme.accent_primary));
                        ui.label(egui::RichText::new("Theme & Appearance")
                            .size(18.0)
                            .strong()
                            .color(self.theme.accent_primary));
                    });

                    ui.add_space(16.0);

                    // Theme selector
                    ui.label(egui::RichText::new("Color Theme:")
                        .size(14.0)
                        .color(self.theme.text_primary));

                    ui.add_space(8.0);

                    // Theme grid - 3 columns
                    egui::Grid::new("theme_grid")
                        .num_columns(3)
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            for (i, preset) in ThemePreset::all().iter().enumerate() {
                                let is_selected = self.theme_preset == *preset;
                                let button_text = preset.name();

                                let button = egui::Button::new(
                                    egui::RichText::new(button_text)
                                        .size(12.0)
                                        .color(if is_selected {
                                            self.theme.bg_dark
                                        } else {
                                            self.theme.text_primary
                                        })
                                )
                                .fill(if is_selected {
                                    self.theme.accent_primary
                                } else {
                                    self.theme.card_bg
                                })
                                .stroke(egui::Stroke::new(
                                    if is_selected { 2.0 } else { 1.0 },
                                    if is_selected { self.theme.accent_glow } else { self.theme.bg_highlight }
                                ))
                                .rounding(egui::Rounding::same(6.0))
                                .min_size(egui::Vec2::new(110.0, 32.0));

                                if ui.add(button).clicked() && !is_selected {
                                    self.theme_preset = *preset;
                                    self.theme = WavelinkTheme::with_preset(*preset);
                                    self.add_notification(
                                        &format!("Theme changed to {}", preset.name()),
                                        NotificationLevel::Success
                                    );
                                }

                                if (i + 1) % 3 == 0 {
                                    ui.end_row();
                                }
                            }
                        });

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(12.0);

                    // Microphone preset selector
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🎤")
                            .size(18.0)
                            .color(self.theme.accent_secondary));
                        ui.label(egui::RichText::new("Microphone Preset:")
                            .size(14.0)
                            .color(self.theme.text_primary));
                    });

                    ui.add_space(8.0);

                    // Microphone preset grid - 2 columns
                    egui::Grid::new("mic_preset_grid")
                        .num_columns(2)
                        .spacing([8.0, 6.0])
                        .show(ui, |ui| {
                            for (i, preset) in MicrophonePreset::all().iter().enumerate() {
                                let is_selected = self.microphone_preset == *preset;
                                let button_text = preset.name();

                                let button = egui::Button::new(
                                    egui::RichText::new(button_text)
                                        .size(11.0)
                                        .color(if is_selected {
                                            self.theme.bg_dark
                                        } else {
                                            self.theme.text_primary
                                        })
                                )
                                .fill(if is_selected {
                                    self.theme.accent_secondary
                                } else {
                                    self.theme.card_bg
                                })
                                .stroke(egui::Stroke::new(
                                    if is_selected { 2.0 } else { 1.0 },
                                    if is_selected { self.theme.accent_glow } else { self.theme.bg_highlight }
                                ))
                                .rounding(egui::Rounding::same(6.0))
                                .min_size(egui::Vec2::new(100.0, 28.0));

                                if ui.add(button)
                                    .on_hover_text(preset.description())
                                    .clicked() && !is_selected
                                {
                                    self.microphone_preset = *preset;

                                    // Show recommended settings
                                    let msg = format!(
                                        "{}: Gain ~{}dB, Gate {}dB",
                                        preset.name(),
                                        preset.recommended_gain_db() as i32,
                                        preset.gate_threshold_db() as i32
                                    );
                                    self.add_notification(&msg, NotificationLevel::Info);

                                    // Warn about phantom power for condensers
                                    if preset.needs_phantom_power() {
                                        self.add_notification(
                                            "Enable 48V phantom power for this microphone",
                                            NotificationLevel::Warning
                                        );
                                    }
                                }

                                if (i + 1) % 2 == 0 {
                                    ui.end_row();
                                }
                            }
                        });

                    // Show current preset info
                    ui.add_space(8.0);
                    egui::Frame::none()
                        .fill(self.theme.bg_highlight)
                        .rounding(egui::Rounding::same(6.0))
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Recommended:")
                                    .size(11.0)
                                    .color(self.theme.text_muted));
                                ui.label(egui::RichText::new(
                                    format!("Gain: {}dB", self.microphone_preset.recommended_gain_db() as i32)
                                ).size(11.0).color(self.theme.success));
                                ui.label(egui::RichText::new("|").color(self.theme.text_muted));
                                ui.label(egui::RichText::new(
                                    format!("Gate: {}dB", self.microphone_preset.gate_threshold_db() as i32)
                                ).size(11.0).color(self.theme.info));
                                if self.microphone_preset.needs_phantom_power() {
                                    ui.label(egui::RichText::new("|").color(self.theme.text_muted));
                                    ui.label(egui::RichText::new("48V")
                                        .size(11.0)
                                        .color(self.theme.warning));
                                }
                            });
                        });

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(12.0);

                    // Interface settings
                    ui.label(egui::RichText::new("Interface Options:")
                        .size(14.0)
                        .color(self.theme.text_primary));

                    ui.add_space(8.0);

                    ui.checkbox(&mut self.keyboard_shortcuts_enabled, "Enable keyboard shortcuts");
                    ui.checkbox(&mut self.auto_save_enabled, "Auto-save every 5 minutes");

                    ui.add_space(12.0);

                    // PipeWire Audio Preset
                    ui.label(egui::RichText::new("PipeWire Audio Preset:")
                        .size(14.0)
                        .color(self.theme.text_primary));

                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        for preset in PipeWirePreset::all() {
                            let is_selected = self.pipewire_preset == *preset;

                            let button = egui::Button::new(
                                egui::RichText::new(preset.name())
                                    .size(11.0)
                                    .color(if is_selected {
                                        self.theme.bg_dark
                                    } else {
                                        self.theme.text_secondary
                                    })
                            )
                            .fill(if is_selected {
                                self.theme.accent_primary
                            } else {
                                self.theme.input_bg
                            })
                            .rounding(egui::Rounding::same(4.0));

                            if ui.add(button).on_hover_text(preset.description()).clicked() {
                                self.pipewire_preset = *preset;
                                self.use_custom_buffer = false; // Reset custom buffer when selecting preset
                                self.add_notification(
                                    &format!("Audio preset: {} ({})", preset.name(), preset.buffer_size()),
                                    NotificationLevel::Info
                                );
                            }
                        }
                    });

                    ui.add_space(12.0);

                    // Custom buffer size control
                    ui.checkbox(&mut self.use_custom_buffer, "Use custom buffer size");

                    if self.use_custom_buffer {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label("Buffer Size:");

                            // Buffer size slider (powers of 2: 64, 128, 256, 512, 1024, 2048)
                            let buffer_options = [64u32, 128, 256, 512, 1024, 2048];
                            let current_idx = buffer_options.iter()
                                .position(|&b| b == self.custom_buffer_size)
                                .unwrap_or(2); // Default to 256

                            let mut slider_val = current_idx as f32;
                            let slider = egui::Slider::new(&mut slider_val, 0.0..=5.0)
                                .step_by(1.0)
                                .custom_formatter(|v, _| {
                                    let idx = v.round() as usize;
                                    format!("{} samples", buffer_options.get(idx).unwrap_or(&256))
                                });

                            if ui.add(slider).changed() {
                                let new_idx = slider_val.round() as usize;
                                if let Some(&new_size) = buffer_options.get(new_idx) {
                                    self.custom_buffer_size = new_size;
                                    let latency_ms = (new_size as f32 / 48000.0) * 1000.0;
                                    self.add_notification(
                                        &format!("Buffer: {} samples (~{:.1}ms)", new_size, latency_ms),
                                        NotificationLevel::Info
                                    );
                                }
                            }
                        });

                        // Show latency estimate
                        let latency_ms = (self.custom_buffer_size as f32 / 48000.0) * 1000.0;
                        let latency_color = if latency_ms < 6.0 {
                            self.theme.success
                        } else if latency_ms < 15.0 {
                            self.theme.warning
                        } else {
                            self.theme.info
                        };
                        ui.label(egui::RichText::new(format!("Latency: ~{:.1}ms", latency_ms))
                            .size(11.0)
                            .color(latency_color));
                    }

                    ui.add_space(12.0);

                    // Audio settings (dynamic based on preset or custom)
                    ui.label(egui::RichText::new("Audio Engine:")
                        .size(14.0)
                        .color(self.theme.text_primary));

                    ui.add_space(4.0);

                    // Use custom buffer size if enabled, otherwise use preset
                    let buffer_size = if self.use_custom_buffer {
                        self.custom_buffer_size
                    } else {
                        self.pipewire_preset.buffer_size()
                    };
                    let latency_ms = if self.use_custom_buffer {
                        (self.custom_buffer_size as f32 / 48000.0) * 1000.0
                    } else {
                        self.pipewire_preset.latency_ms()
                    };

                    egui::Grid::new("audio_info")
                        .num_columns(2)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Sample Rate:");
                            ui.label(egui::RichText::new("48000 Hz").color(self.theme.success));
                            ui.end_row();

                            ui.label("Buffer Size:");
                            ui.label(egui::RichText::new(format!("{} samples", buffer_size))
                                .color(self.theme.success));
                            ui.end_row();

                            ui.label("Latency:");
                            let latency_color = if latency_ms < 6.0 {
                                self.theme.success
                            } else if latency_ms < 15.0 {
                                self.theme.warning
                            } else {
                                self.theme.info
                            };
                            ui.label(egui::RichText::new(format!("~{:.1}ms", latency_ms))
                                .color(latency_color));
                            ui.end_row();

                            // PipeWire status
                            ui.label("PipeWire:");
                            let pw_running = crate::pipewire::is_pipewire_running();
                            ui.label(egui::RichText::new(
                                if pw_running { "Connected" } else { "Not detected" }
                            ).color(if pw_running { self.theme.success } else { self.theme.warning }));
                            ui.end_row();

                            // Virtual device status
                            ui.label("Virtual Device:");
                            let vd_status = if self.pipewire_device.is_some() {
                                ("PhantomLink Clean ✓", self.theme.success)
                            } else if pw_running {
                                ("Not created", self.theme.warning)
                            } else {
                                ("Unavailable", self.theme.text_muted)
                            };
                            ui.label(egui::RichText::new(vd_status.0).color(vd_status.1));
                            ui.end_row();

                            // Linked source
                            if let Some(ref mgr) = self.pipewire_device {
                                ui.label("Linked Source:");
                                let source_name = mgr.get_linked_source()
                                    .map(|s| s.name.as_str())
                                    .unwrap_or("None");
                                ui.label(egui::RichText::new(source_name)
                                    .color(self.theme.text_secondary));
                                ui.end_row();
                            }
                        });

                    // Virtual device control buttons
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if self.pipewire_device.is_none() {
                            if ui.add(enhanced_glow_button("Create Virtual Device", &self.theme, GlowButtonStyle::Primary))
                                .clicked()
                            {
                                let mut mgr = crate::pipewire::VirtualDeviceManager::default();
                                if let Err(e) = mgr.create_virtual_device() {
                                    self.add_notification(&format!("Failed to create device: {}", e), NotificationLevel::Error);
                                } else {
                                    let _ = mgr.auto_link_source(Some("Scarlett"));
                                    self.add_notification("Virtual device created", NotificationLevel::Success);
                                    self.pipewire_device = Some(mgr);
                                }
                            }
                        } else {
                            if ui.add(enhanced_glow_button("Destroy Device", &self.theme, GlowButtonStyle::Warning))
                                .clicked()
                            {
                                if let Some(ref mut mgr) = self.pipewire_device {
                                    let _ = mgr.destroy_virtual_device();
                                }
                                self.pipewire_device = None;
                                self.add_notification("Virtual device removed", NotificationLevel::Info);
                            }

                            if ui.add(enhanced_glow_button("Re-link Source", &self.theme, GlowButtonStyle::Secondary))
                                .clicked()
                            {
                                if let Some(ref mut mgr) = self.pipewire_device {
                                    if let Err(e) = mgr.auto_link_source(Some("Scarlett")) {
                                        self.add_notification(&format!("Link failed: {}", e), NotificationLevel::Error);
                                    } else {
                                        self.add_notification("Source re-linked", NotificationLevel::Success);
                                    }
                                }
                            }
                        }
                    });
                });

            ui.add_space(16.0);

            // Right column - GhostWave AI Denoising
            egui::Frame::none()
                .fill(self.theme.translucent_panel_bg())
                .stroke(egui::Stroke::new(1.0, self.theme.accent_secondary))
                .rounding(egui::Rounding::same(16.0))
                .inner_margin(egui::Margin::same(20.0))
                .show(ui, |ui| {
                    ui.set_min_width(400.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🔇")
                            .size(22.0)
                            .color(self.theme.accent_secondary));
                        ui.label(egui::RichText::new("GhostWave AI Denoising")
                            .size(18.0)
                            .strong()
                            .color(self.theme.accent_secondary));
                    });

                    ui.add_space(12.0);

                    // GPU Status
                    egui::Frame::none()
                        .fill(self.theme.bg_highlight)
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let rtx_active = self.ghostwave.as_ref()
                                    .map(|g| g.is_rtx_active())
                                    .unwrap_or(false);

                                let status_color = if rtx_active {
                                    self.theme.success
                                } else if self.ghostwave.is_some() {
                                    self.theme.warning
                                } else {
                                    self.theme.error
                                };

                                ui.label(egui::RichText::new("●")
                                    .size(16.0)
                                    .color(status_color));

                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(&self.driver_info.gpu_name)
                                        .size(13.0)
                                        .strong()
                                        .color(self.theme.text_primary));

                                    let status_text = if rtx_active {
                                        format!("RTX Active - {}", self.driver_info.driver_type)
                                    } else if self.ghostwave.is_some() {
                                        "CPU Mode".to_string()
                                    } else {
                                        "GhostWave unavailable".to_string()
                                    };

                                    ui.label(egui::RichText::new(status_text)
                                        .size(11.0)
                                        .color(self.theme.text_muted));
                                });
                            });
                        });

                    ui.add_space(16.0);

                    // Enable toggle
                    let mut enabled = self.ghostwave.as_ref()
                        .map(|g| g.is_enabled())
                        .unwrap_or(false);

                    if ui.checkbox(&mut enabled, "Enable AI Noise Suppression").changed() {
                        if let Some(ref mut gw) = self.ghostwave {
                            gw.set_enabled(enabled);
                        }
                    }

                    if enabled && self.ghostwave.is_some() {
                        ui.add_space(12.0);

                        // Profile selector
                        ui.label(egui::RichText::new("Processing Profile:")
                            .size(13.0)
                            .color(self.theme.text_primary));

                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            for profile in PhantomLinkProfile::all() {
                                let is_selected = self.ghostwave_profile == *profile;

                                let button = egui::Button::new(
                                    egui::RichText::new(profile.name())
                                        .size(11.0)
                                        .color(if is_selected {
                                            self.theme.bg_dark
                                        } else {
                                            self.theme.text_secondary
                                        })
                                )
                                .fill(if is_selected {
                                    self.theme.accent_secondary
                                } else {
                                    self.theme.input_bg
                                })
                                .rounding(egui::Rounding::same(4.0));

                                if ui.add(button).on_hover_text(profile.description()).clicked() {
                                    self.ghostwave_profile = *profile;
                                    if let Some(ref mut gw) = self.ghostwave {
                                        let _ = gw.set_profile(*profile);
                                    }
                                }
                            }
                        });

                        ui.add_space(12.0);

                        // Strength slider
                        ui.horizontal(|ui| {
                            ui.label("Suppression Strength:");

                            let slider = egui::Slider::new(&mut self.ghostwave_strength, 0.0..=1.0)
                                .show_value(true)
                                .custom_formatter(|v, _| format!("{:.0}%", v * 100.0));

                            if ui.add(slider).changed() {
                                if let Some(ref mut gw) = self.ghostwave {
                                    let _ = gw.set_noise_strength(self.ghostwave_strength);
                                }
                            }
                        });

                        ui.add_space(8.0);

                        // Echo Cancellation toggle
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("🔊")
                                .size(14.0)
                                .color(if self.echo_cancellation_enabled {
                                    self.theme.accent_secondary
                                } else {
                                    self.theme.text_muted
                                }));

                            if ui.checkbox(&mut self.echo_cancellation_enabled, "Echo Cancellation (AEC)")
                                .on_hover_text("Removes speaker audio from microphone input")
                                .changed()
                            {
                                if let Some(ref mut gw) = self.ghostwave {
                                    gw.set_echo_cancellation(self.echo_cancellation_enabled);
                                }
                            }
                        });

                        if self.echo_cancellation_enabled {
                            ui.label(egui::RichText::new("Uses speaker output as reference for echo removal")
                                .size(10.0)
                                .italics()
                                .color(self.theme.text_muted));
                        }

                        ui.add_space(12.0);

                        // Metrics with extended telemetry
                        if let Some(ref gw) = self.ghostwave {
                            let metrics = gw.get_metrics();

                            egui::Grid::new("ghostwave_metrics")
                                .num_columns(2)
                                .spacing([12.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label("Processing Latency:");
                                    let latency_color = if metrics.latency_ms < 5.0 {
                                        self.theme.success
                                    } else if metrics.latency_ms < 15.0 {
                                        self.theme.warning
                                    } else {
                                        self.theme.error
                                    };
                                    ui.label(egui::RichText::new(format!("{:.1}ms", metrics.latency_ms))
                                        .color(latency_color));
                                    ui.end_row();

                                    ui.label("Frames Processed:");
                                    ui.label(egui::RichText::new(format!("{}", metrics.frames_processed))
                                        .color(self.theme.text_secondary));
                                    ui.end_row();

                                    // XRuns counter
                                    ui.label("XRuns:");
                                    let xrun_color = if metrics.xruns == 0 {
                                        self.theme.success
                                    } else if metrics.xruns < 10 {
                                        self.theme.warning
                                    } else {
                                        self.theme.error
                                    };
                                    ui.label(egui::RichText::new(format!("{}", metrics.xruns))
                                        .color(xrun_color));
                                    ui.end_row();

                                    // Voice activity
                                    ui.label("Voice Activity:");
                                    ui.label(egui::RichText::new(
                                        if metrics.voice_activity { "Active" } else { "Silent" }
                                    ).color(if metrics.voice_activity {
                                        self.theme.success
                                    } else {
                                        self.theme.text_muted
                                    }));
                                    ui.end_row();

                                    // Noise reduction amount
                                    if metrics.noise_reduction_db > 0.0 {
                                        ui.label("Noise Reduction:");
                                        ui.label(egui::RichText::new(format!("-{:.1}dB", metrics.noise_reduction_db))
                                            .color(self.theme.accent_secondary));
                                        ui.end_row();
                                    }
                                });
                        }
                    }

                    // GPU Fallback Warning with Restart Action
                    if let Some(ref gw) = self.ghostwave {
                        let fallback = gw.get_gpu_fallback();
                        if fallback.fallback_active {
                            ui.add_space(12.0);
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgba_premultiplied(
                                    self.theme.warning.r(),
                                    self.theme.warning.g(),
                                    self.theme.warning.b(),
                                    40
                                ))
                                .stroke(egui::Stroke::new(1.0, self.theme.warning))
                                .rounding(egui::Rounding::same(8.0))
                                .inner_margin(egui::Margin::same(10.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("⚠")
                                            .size(16.0)
                                            .color(self.theme.warning));
                                        ui.vertical(|ui| {
                                            ui.label(egui::RichText::new("GPU Fallback Active")
                                                .size(12.0)
                                                .strong()
                                                .color(self.theme.warning));
                                            if let Some(reason) = &fallback.fallback_reason {
                                                ui.label(egui::RichText::new(reason)
                                                    .size(10.0)
                                                    .color(self.theme.text_muted));
                                            }
                                            ui.label(egui::RichText::new(format!("Fallback count: {}", fallback.fallback_count))
                                                .size(10.0)
                                                .color(self.theme.text_muted));
                                        });
                                    });
                                });
                        }
                    }

                    ui.add_space(8.0);

                    // Restart GPU button (shown when in fallback or for manual reset)
                    if let Some(ref gw) = self.ghostwave {
                        if gw.get_gpu_fallback().fallback_active || !gw.is_rtx_active() {
                            if ui.add(enhanced_glow_button("🔄 Restart GPU", &self.theme, GlowButtonStyle::Warning))
                                .on_hover_text("Re-initialize GPU processing")
                                .clicked()
                            {
                                if let Some(ref mut gw) = self.ghostwave {
                                    match gw.restart_gpu() {
                                        Ok(()) => {
                                            self.add_notification("GPU processing restarted", NotificationLevel::Success);
                                        }
                                        Err(e) => {
                                            self.add_notification(&format!("GPU restart failed: {}", e), NotificationLevel::Error);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    ui.add_space(12.0);

                    ui.label(egui::RichText::new("NVIDIA Broadcast-quality noise suppression for Linux")
                        .size(11.0)
                        .italics()
                        .color(self.theme.text_muted));
                });
        });

        ui.add_space(20.0);

        // Scarlett Solo Hardware Control Panel
        egui::Frame::none()
            .fill(self.theme.translucent_panel_bg())
            .stroke(egui::Stroke::new(1.0, self.theme.error.linear_multiply(0.6)))
            .rounding(egui::Rounding::same(16.0))
            .inner_margin(egui::Margin::same(20.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🎛")
                        .size(22.0)
                        .color(self.theme.error));
                    ui.label(egui::RichText::new("Scarlett Solo 4th Gen")
                        .size(18.0)
                        .strong()
                        .color(self.theme.error));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Connection status
                        let connected = self.scarlett.is_some();
                        let usb_connected = crate::scarlett::is_scarlett_solo_connected();

                        if connected {
                            ui.label(egui::RichText::new("● Connected")
                                .size(12.0)
                                .color(self.theme.success));
                        } else if usb_connected {
                            ui.label(egui::RichText::new("● USB Detected")
                                .size(12.0)
                                .color(self.theme.warning));
                        } else {
                            ui.label(egui::RichText::new("○ Not Connected")
                                .size(12.0)
                                .color(self.theme.text_muted));
                        }
                    });
                });

                // Collect errors to show after the borrow ends
                let mut scarlett_errors: Vec<String> = Vec::new();

                if let Some(ref mut scarlett) = self.scarlett {
                    ui.add_space(16.0);

                    // Capture theme colors before entering closures
                    let theme_text_primary = self.theme.text_primary;
                    let theme_text_secondary = self.theme.text_secondary;
                    let theme_text_muted = self.theme.text_muted;
                    let theme_error = self.theme.error;
                    let theme_warning = self.theme.warning;
                    let theme_success = self.theme.success;
                    let theme_input_bg = self.theme.input_bg;
                    let theme_bg_dark = self.theme.bg_dark;

                    // Capture current state
                    let mut phantom_power = self.scarlett_phantom_power;
                    let mut air_mode = self.scarlett_air_mode;
                    let mut input_level = self.scarlett_input_level;
                    let mut direct_monitor = self.scarlett_direct_monitor;
                    let level_meters = self.scarlett_level_meters.clone();

                    ui.horizontal(|ui| {
                        // Left: Input Controls
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Input Controls")
                                .size(14.0)
                                .strong()
                                .color(theme_text_primary));

                            ui.add_space(8.0);

                            // 48V Phantom Power
                            ui.horizontal(|ui| {
                                let phantom_label = if phantom_power { "⚡ 48V ON" } else { "48V OFF" };
                                let phantom_color = if phantom_power {
                                    theme_error
                                } else {
                                    theme_text_muted
                                };

                                if ui.add(egui::Button::new(
                                    egui::RichText::new(phantom_label)
                                        .size(12.0)
                                        .color(if phantom_power { theme_bg_dark } else { phantom_color })
                                ).fill(if phantom_power { theme_error } else { theme_input_bg })
                                 .rounding(egui::Rounding::same(4.0)))
                                .on_hover_text("Toggle 48V phantom power for condenser microphones")
                                .clicked()
                                {
                                    phantom_power = !phantom_power;
                                    if let Err(e) = scarlett.set_phantom_power(phantom_power) {
                                        scarlett_errors.push(format!("Failed to set phantom power: {}", e));
                                    }
                                }
                            });

                            ui.add_space(4.0);

                            // Air Mode
                            ui.horizontal(|ui| {
                                ui.label("Air Mode:");
                                for mode in crate::scarlett::AirMode::all() {
                                    let selected = air_mode == *mode;
                                    if ui.selectable_label(selected, mode.name()).clicked() {
                                        air_mode = *mode;
                                        if let Err(e) = scarlett.set_air_mode(*mode) {
                                            scarlett_errors.push(format!("Air mode error: {}", e));
                                        }
                                    }
                                }
                            });

                            ui.add_space(4.0);

                            // Input Level (Line/Inst)
                            ui.horizontal(|ui| {
                                ui.label("Input 1:");
                                for level in &[crate::scarlett::InputLevel::Line, crate::scarlett::InputLevel::Instrument] {
                                    let selected = input_level == *level;
                                    if ui.selectable_label(selected, level.name()).clicked() {
                                        input_level = *level;
                                        if let Err(e) = scarlett.set_input_level(*level) {
                                            scarlett_errors.push(format!("Input level error: {}", e));
                                        }
                                    }
                                }
                            });
                        });

                        ui.add_space(30.0);

                        // Right: Monitoring & Levels
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Monitoring")
                                .size(14.0)
                                .strong()
                                .color(theme_text_primary));

                            ui.add_space(8.0);

                            // Direct Monitor toggle
                            if ui.checkbox(&mut direct_monitor, "Direct Monitor")
                                .on_hover_text("Monitor inputs directly with zero latency")
                                .changed()
                            {
                                if let Err(e) = scarlett.set_direct_monitor(direct_monitor) {
                                    scarlett_errors.push(format!("Direct monitor error: {}", e));
                                }
                            }

                            ui.add_space(8.0);

                            // Input level meters
                            ui.label(egui::RichText::new("Input Levels:")
                                .size(11.0)
                                .color(theme_text_secondary));

                            ui.horizontal(|ui| {
                                // Input 1 (Line/Inst)
                                let level1 = level_meters.get_normalized(0);
                                let level1_db = level_meters.get_db(0);
                                ui.vertical(|ui| {
                                    ui.label("In 1");
                                    let bar_color = if level1 > 0.9 { theme_error }
                                        else if level1 > 0.7 { theme_warning }
                                        else { theme_success };
                                    ui.add(egui::ProgressBar::new(level1)
                                        .fill(bar_color)
                                        .desired_width(60.0));
                                    ui.label(egui::RichText::new(format!("{:.0}dB", level1_db))
                                        .size(9.0)
                                        .color(theme_text_muted));
                                });

                                // Input 2 (XLR/Mic)
                                let level2 = level_meters.get_normalized(1);
                                let level2_db = level_meters.get_db(1);
                                ui.vertical(|ui| {
                                    ui.label("In 2");
                                    let bar_color = if level2 > 0.9 { theme_error }
                                        else if level2 > 0.7 { theme_warning }
                                        else { theme_success };
                                    ui.add(egui::ProgressBar::new(level2)
                                        .fill(bar_color)
                                        .desired_width(60.0));
                                    ui.label(egui::RichText::new(format!("{:.0}dB", level2_db))
                                        .size(9.0)
                                        .color(theme_text_muted));
                                });
                            });
                        });
                    });

                    // Update state after closure
                    self.scarlett_phantom_power = phantom_power;
                    self.scarlett_air_mode = air_mode;
                    self.scarlett_input_level = input_level;
                    self.scarlett_direct_monitor = direct_monitor;
                } else {
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("Connect your Scarlett Solo 4th Gen to enable hardware controls")
                        .size(12.0)
                        .italics()
                        .color(self.theme.text_muted));

                    ui.add_space(8.0);
                    if ui.add(enhanced_glow_button("🔄 Detect Device", &self.theme, GlowButtonStyle::Primary)).clicked() {
                        match crate::scarlett::ScarlettSolo::new() {
                            Ok(s) => {
                                self.scarlett_phantom_power = s.get_phantom_power();
                                self.scarlett_air_mode = s.get_air_mode();
                                self.scarlett_input_level = s.get_input_level();
                                self.scarlett_direct_monitor = s.get_direct_monitor();
                                self.scarlett = Some(s);
                                self.add_notification("Scarlett Solo detected!", NotificationLevel::Success);
                            }
                            Err(e) => {
                                self.add_notification(&format!("Detection failed: {}", e), NotificationLevel::Error);
                            }
                        }
                    }
                }

                // Show any Scarlett control errors collected from the closure
                for err in scarlett_errors {
                    self.add_notification(&err, NotificationLevel::Error);
                }
            });

        ui.add_space(20.0);

        // Bottom action bar
        egui::Frame::none()
            .fill(self.theme.translucent_panel_bg())
            .stroke(egui::Stroke::new(1.0, self.theme.bg_highlight))
            .rounding(egui::Rounding::same(12.0))
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.add(enhanced_glow_button("💾 Save Settings", &self.theme, GlowButtonStyle::Success)).clicked() {
                        self.save_configuration();
                    }

                    if ui.add(enhanced_glow_button("🔄 Reset to Defaults", &self.theme, GlowButtonStyle::Warning)).clicked() {
                        self.theme_preset = ThemePreset::default();
                        self.theme = WavelinkTheme::with_preset(self.theme_preset);
                        self.add_notification("Settings reset to defaults", NotificationLevel::Info);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("PhantomLink v0.3.0 + GhostWave v0.2.0")
                            .size(11.0)
                            .color(self.theme.text_muted));

                        ui.add_space(16.0);

                        if ui.add(enhanced_glow_button("❓ Help", &self.theme, GlowButtonStyle::Secondary)).clicked() {
                            self.show_help_overlay = true;
                        }
                    });
                });
            });
    }
}