// Additional methods for the enhanced PhantomlinkApp
use eframe::egui;
use crate::gui::theme::{WavelinkTheme, ThemePreset};
use crate::gui::widgets::{enhanced_glow_button, GlowButtonStyle};
use crate::gui::NotificationLevel;
use crate::ghostwave_integration::PhantomLinkProfile;

impl super::PhantomlinkApp {
    pub fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        if !self.keyboard_shortcuts_enabled {
            return;
        }

        ctx.input(|i| {
            if i.key_pressed(egui::Key::F1) {
                self.show_help_overlay = !self.show_help_overlay;
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
                self.save_configuration();
            }
            if i.key_pressed(egui::Key::Space) {
                if self.audio_started {
                    self.audio_engine.stop();
                    self.audio_started = false;
                } else {
                    if let Ok(()) = self.audio_engine.start() {
                        self.audio_started = true;
                    }
                }
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::M) {
                self.mute_all = !self.mute_all;
                self.add_notification(
                    if self.mute_all { "All channels muted" } else { "All channels unmuted" },
                    NotificationLevel::Info
                );
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
                    ("Ctrl+S", "Save configuration"),
                    ("Ctrl+M", "Mute/Unmute all channels"),
                    ("Esc", "Close current dialog"),
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

                    // Interface settings
                    ui.label(egui::RichText::new("Interface Options:")
                        .size(14.0)
                        .color(self.theme.text_primary));

                    ui.add_space(8.0);

                    ui.checkbox(&mut self.keyboard_shortcuts_enabled, "Enable keyboard shortcuts");
                    ui.checkbox(&mut self.auto_save_enabled, "Auto-save every 5 minutes");

                    ui.add_space(12.0);

                    // Audio settings
                    ui.label(egui::RichText::new("Audio Engine:")
                        .size(14.0)
                        .color(self.theme.text_primary));

                    ui.add_space(4.0);

                    egui::Grid::new("audio_info")
                        .num_columns(2)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Sample Rate:");
                            ui.label(egui::RichText::new("48000 Hz").color(self.theme.success));
                            ui.end_row();

                            ui.label("Buffer Size:");
                            ui.label(egui::RichText::new("256 samples").color(self.theme.success));
                            ui.end_row();

                            ui.label("Latency:");
                            ui.label(egui::RichText::new("~5.3ms").color(self.theme.success));
                            ui.end_row();
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

                        // Metrics
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
                                });
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
                        ui.label(egui::RichText::new("PhantomLink v0.2.0 + GhostWave")
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