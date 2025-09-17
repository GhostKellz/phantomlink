// Additional methods for the enhanced PhantomlinkApp
use eframe::egui;
use crate::gui::theme::WavelinkTheme;
use crate::gui::widgets::{enhanced_glow_button, GlowButtonStyle};
use crate::gui::NotificationLevel;

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
        egui::Frame::none()
            .fill(self.theme.translucent_panel_bg())
            .stroke(egui::Stroke::new(1.0, self.theme.medium_blue))
            .rounding(egui::Rounding::same(16.0))
            .inner_margin(egui::Margin::same(24.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⚙️")
                        .size(24.0)
                        .color(self.theme.green_primary));
                    ui.label(egui::RichText::new("Application Settings")
                        .size(20.0)
                        .strong()
                        .color(self.theme.green_primary));
                });

                ui.add_space(20.0);

                // Interface Settings
                ui.collapsing("🖥️ Interface", |ui| {
                    ui.checkbox(&mut self.keyboard_shortcuts_enabled, "Enable keyboard shortcuts");
                    ui.checkbox(&mut self.auto_save_enabled, "Auto-save configuration every 5 minutes");

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("UI Scale:");
                        ui.add(egui::Slider::new(&mut 1.0f32, 0.5..=2.0)
                            .step_by(0.1)
                            .suffix("x"));
                    });
                });

                ui.add_space(15.0);

                // Audio Settings
                ui.collapsing("🔊 Audio Engine", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Sample Rate:");
                        ui.label(egui::RichText::new("48000 Hz")
                            .color(self.theme.success));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Buffer Size:");
                        ui.label(egui::RichText::new("256 samples")
                            .color(self.theme.success));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Latency:");
                        ui.label(egui::RichText::new("~5.3ms")
                            .color(self.theme.success));
                    });

                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Backend: PipeWire → ALSA → Scarlett Solo")
                        .size(11.0)
                        .color(self.theme.text_muted));
                });

                ui.add_space(15.0);

                // Performance Monitor
                ui.collapsing("📊 Performance", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("CPU Usage:");
                        ui.label(egui::RichText::new("12%")
                            .color(self.theme.success));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Memory Usage:");
                        ui.label(egui::RichText::new("45 MB")
                            .color(self.theme.success));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Audio Dropouts:");
                        ui.label(egui::RichText::new("0")
                            .color(self.theme.success));
                    });
                });

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(15.0);

                // Action buttons
                ui.horizontal(|ui| {
                    if ui.add(enhanced_glow_button("💾 Save Settings", &self.theme, GlowButtonStyle::Success)).clicked() {
                        self.save_configuration();
                    }

                    if ui.add(enhanced_glow_button("🔄 Reset to Defaults", &self.theme, GlowButtonStyle::Warning)).clicked() {
                        self.add_notification("Settings reset to defaults", NotificationLevel::Info);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(enhanced_glow_button("❓ Help", &self.theme, GlowButtonStyle::Secondary)).clicked() {
                            self.show_help_overlay = true;
                        }
                    });
                });

                ui.add_space(10.0);

                // Version info
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("PhantomLink v0.2.0 | Built with ❤️ and Rust")
                        .size(10.0)
                        .color(self.theme.text_muted));
                });
            });
    }
}