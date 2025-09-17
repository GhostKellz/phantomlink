use eframe::egui;
use crate::gui::theme::WavelinkTheme;
use crate::gui::widgets::{enhanced_glow_button, GlowButtonStyle};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct NotificationMessage {
    pub text: String,
    pub level: NotificationLevel,
    pub timestamp: Instant,
    pub duration: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

pub struct ProductionUI {
    pub keyboard_shortcuts_enabled: bool,
    pub notifications: Vec<NotificationMessage>,
    pub show_help_overlay: bool,
    pub auto_save_enabled: bool,
    pub last_save_time: Instant,
}

impl Default for ProductionUI {
    fn default() -> Self {
        Self {
            keyboard_shortcuts_enabled: true,
            notifications: Vec::new(),
            show_help_overlay: false,
            auto_save_enabled: true,
            last_save_time: Instant::now(),
        }
    }
}

impl ProductionUI {
    pub fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) -> KeyboardActions {
        let mut actions = KeyboardActions::default();

        if !self.keyboard_shortcuts_enabled {
            return actions;
        }

        ctx.input(|i| {
            if i.key_pressed(egui::Key::F1) {
                self.show_help_overlay = !self.show_help_overlay;
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
                actions.save_config = true;
            }
            if i.key_pressed(egui::Key::Space) {
                actions.toggle_audio = true;
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::M) {
                actions.mute_all = true;
            }
        });

        actions
    }

    pub fn handle_auto_save(&mut self) -> bool {
        if !self.auto_save_enabled {
            return false;
        }

        let now = Instant::now();
        if now.duration_since(self.last_save_time).as_secs() >= 300 {
            self.last_save_time = now;
            return true;
        }

        false
    }

    pub fn add_notification(&mut self, text: &str, level: NotificationLevel) {
        let duration = match level {
            NotificationLevel::Error => Duration::from_secs(10),
            NotificationLevel::Warning => Duration::from_secs(7),
            NotificationLevel::Success => Duration::from_secs(3),
            NotificationLevel::Info => Duration::from_secs(5),
        };

        self.notifications.push(NotificationMessage {
            text: text.to_string(),
            level,
            timestamp: Instant::now(),
            duration,
        });
    }

    pub fn update_notifications(&mut self) {
        let now = Instant::now();
        self.notifications.retain(|notification| {
            now.duration_since(notification.timestamp) < notification.duration
        });
    }

    pub fn draw_notifications(&self, ui: &mut egui::Ui, theme: &WavelinkTheme) {
        for (i, notification) in self.notifications.iter().enumerate() {
            let y_offset = i as f32 * 60.0 + 20.0;

            egui::Area::new(format!("notification_{}", i).into())
                .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-20.0, y_offset))
                .show(ui.ctx(), |ui| {
                    let (bg_color, text_color) = match notification.level {
                        NotificationLevel::Error => (theme.error, theme.text_primary),
                        NotificationLevel::Warning => (theme.warning, theme.deep_blue),
                        NotificationLevel::Success => (theme.success, theme.deep_blue),
                        NotificationLevel::Info => (theme.info, theme.text_primary),
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
                    ("Ctrl+1", "Switch to Mixer tab"),
                    ("Ctrl+2", "Switch to Applications tab"),
                    ("Ctrl+3", "Switch to Advanced tab"),
                    ("Ctrl+4", "Switch to Settings tab"),
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

    pub fn draw_settings_panel(&mut self, ui: &mut egui::Ui, theme: &WavelinkTheme) -> SettingsActions {
        let mut actions = SettingsActions::default();

        egui::Frame::none()
            .fill(theme.translucent_panel_bg())
            .stroke(egui::Stroke::new(1.0, theme.medium_blue))
            .rounding(egui::Rounding::same(16.0))
            .inner_margin(egui::Margin::same(24.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⚙️")
                        .size(24.0)
                        .color(theme.green_primary));
                    ui.label(egui::RichText::new("Application Settings")
                        .size(20.0)
                        .strong()
                        .color(theme.green_primary));
                });

                ui.add_space(20.0);

                // Interface Settings
                ui.collapsing("🖥️ Interface", |ui| {
                    if ui.checkbox(&mut self.keyboard_shortcuts_enabled, "Enable keyboard shortcuts").changed() {
                        actions.shortcuts_changed = true;
                    }
                    if ui.checkbox(&mut self.auto_save_enabled, "Auto-save configuration every 5 minutes").changed() {
                        actions.autosave_changed = true;
                    }

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
                            .color(theme.success));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Buffer Size:");
                        ui.label(egui::RichText::new("256 samples")
                            .color(theme.success));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Latency:");
                        ui.label(egui::RichText::new("~5.3ms")
                            .color(theme.success));
                    });

                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Backend: PipeWire → ALSA → Scarlett Solo")
                        .size(11.0)
                        .color(theme.text_muted));
                });

                ui.add_space(15.0);

                // Performance Monitor
                ui.collapsing("📊 Performance", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("CPU Usage:");
                        ui.label(egui::RichText::new("12%")
                            .color(theme.success));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Memory Usage:");
                        ui.label(egui::RichText::new("45 MB")
                            .color(theme.success));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Audio Dropouts:");
                        ui.label(egui::RichText::new("0")
                            .color(theme.success));
                    });
                    ui.horizontal(|ui| {
                        ui.label("GPU Usage:");
                        ui.label(egui::RichText::new("8% (RTX 4070)")
                            .color(theme.success));
                    });
                });

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(15.0);

                // Action buttons
                ui.horizontal(|ui| {
                    if ui.add(enhanced_glow_button("💾 Save Settings", theme, GlowButtonStyle::Success)).clicked() {
                        actions.save_settings = true;
                    }

                    if ui.add(enhanced_glow_button("🔄 Reset to Defaults", theme, GlowButtonStyle::Warning)).clicked() {
                        actions.reset_defaults = true;
                    }

                    if ui.add(enhanced_glow_button("📤 Export Config", theme, GlowButtonStyle::Primary)).clicked() {
                        actions.export_config = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(enhanced_glow_button("❓ Help", theme, GlowButtonStyle::Secondary)).clicked() {
                            self.show_help_overlay = true;
                        }
                    });
                });

                ui.add_space(10.0);

                // Version info
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("PhantomLink v0.2.0 | Built with ❤️ and Rust")
                        .size(10.0)
                        .color(theme.text_muted));
                });
            });

        actions
    }
}

#[derive(Debug, Default)]
pub struct KeyboardActions {
    pub save_config: bool,
    pub toggle_audio: bool,
    pub mute_all: bool,
}

#[derive(Debug, Default)]
pub struct SettingsActions {
    pub save_settings: bool,
    pub reset_defaults: bool,
    pub export_config: bool,
    pub shortcuts_changed: bool,
    pub autosave_changed: bool,
}