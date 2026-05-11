//! UI Widget library for PhantomLink mixer interface.
//!
//! Provides professional audio-style widgets: knobs, meters, buttons, and channel strips.

#![allow(dead_code)] // Widget library - components used as needed in various panels

use crate::gui::theme::WavelinkTheme;
use eframe::egui;

/// Hardware-style rotary knob for gain/pan controls
pub struct HardwareKnob {
    value: f32,
    min: f32,
    max: f32,
    label: String,
    unit: String,
}

impl HardwareKnob {
    pub fn new(label: &str, value: f32, min: f32, max: f32, unit: &str) -> Self {
        Self {
            value,
            min,
            max,
            label: label.to_string(),
            unit: unit.to_string(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, theme: &WavelinkTheme) -> egui::Response {
        let size = egui::vec2(60.0, 80.0);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());

        if response.dragged() {
            let delta = -response.drag_delta().y * 0.005;
            self.value = (self.value + delta * (self.max - self.min)).clamp(self.min, self.max);
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = rect.center();
            let knob_radius = 22.0;

            // Outer ring (metallic look)
            painter.circle_filled(center, knob_radius + 3.0, egui::Color32::from_gray(60));
            painter.circle_stroke(
                center,
                knob_radius + 3.0,
                egui::Stroke::new(1.0, egui::Color32::from_gray(80)),
            );

            // Knob body with gradient effect
            let knob_color = if response.hovered() || response.dragged() {
                egui::Color32::from_gray(90)
            } else {
                egui::Color32::from_gray(70)
            };
            painter.circle_filled(center, knob_radius, knob_color);

            // Inner shadow
            painter.circle_stroke(
                center,
                knob_radius - 2.0,
                egui::Stroke::new(1.0, egui::Color32::from_gray(50)),
            );

            // Position indicator line
            let normalized = (self.value - self.min) / (self.max - self.min);
            let angle = std::f32::consts::PI * 0.75 + normalized * std::f32::consts::PI * 1.5;
            let indicator_start =
                center + egui::vec2(angle.cos(), angle.sin()) * (knob_radius * 0.4);
            let indicator_end =
                center + egui::vec2(angle.cos(), angle.sin()) * (knob_radius * 0.85);

            painter.line_segment(
                [indicator_start, indicator_end],
                egui::Stroke::new(3.0, theme.accent_primary),
            );

            // Arc track (background)
            let arc_radius = knob_radius + 8.0;
            for i in 0..32 {
                let t = i as f32 / 31.0;
                let a = std::f32::consts::PI * 0.75 + t * std::f32::consts::PI * 1.5;
                let pos = center + egui::vec2(a.cos(), a.sin()) * arc_radius;
                let color = if t <= normalized {
                    theme.accent_primary
                } else {
                    egui::Color32::from_gray(40)
                };
                painter.circle_filled(pos, 1.5, color);
            }

            // Label
            painter.text(
                egui::pos2(center.x, rect.top() + 4.0),
                egui::Align2::CENTER_TOP,
                &self.label,
                egui::FontId::proportional(10.0),
                theme.text_secondary,
            );

            // Value display
            let value_text = if self.unit == "dB" {
                format!("{:+.1}{}", self.value, self.unit)
            } else {
                format!("{:.0}{}", self.value * 100.0, self.unit)
            };
            painter.text(
                egui::pos2(center.x, rect.bottom() - 4.0),
                egui::Align2::CENTER_BOTTOM,
                value_text,
                egui::FontId::proportional(9.0),
                theme.text_muted,
            );
        }

        response
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, v: f32) {
        self.value = v.clamp(self.min, self.max);
    }
}

/// Professional VU meter with peak hold
pub struct ProfessionalMeter {
    peak: f32,
    rms: f32,
    peak_hold: f32,
    peak_hold_time: f32,
    clip_indicator: bool,
    clip_hold_time: f32,
}

impl Default for ProfessionalMeter {
    fn default() -> Self {
        Self {
            peak: 0.0,
            rms: 0.0,
            peak_hold: 0.0,
            peak_hold_time: 0.0,
            clip_indicator: false,
            clip_hold_time: 0.0,
        }
    }
}

impl ProfessionalMeter {
    pub fn update(&mut self, peak: f32, rms: f32, dt: f32) {
        self.peak = peak;
        self.rms = rms;

        // Peak hold with decay
        if peak > self.peak_hold {
            self.peak_hold = peak;
            self.peak_hold_time = 1.5; // Hold for 1.5 seconds
        } else if self.peak_hold_time > 0.0 {
            self.peak_hold_time -= dt;
        } else {
            self.peak_hold = (self.peak_hold - dt * 0.5).max(peak);
        }

        // Clip detection
        if peak >= 0.99 {
            self.clip_indicator = true;
            self.clip_hold_time = 2.0;
        } else if self.clip_hold_time > 0.0 {
            self.clip_hold_time -= dt;
        } else {
            self.clip_indicator = false;
        }
    }

    pub fn draw(&self, ui: &mut egui::Ui, theme: &WavelinkTheme, height: f32) {
        let width = 24.0;
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());

        // Click to reset clip indicator
        if response.clicked() {
            // Would reset clip here if mutable
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background
            painter.rect_filled(rect, egui::Rounding::same(3.0), theme.vu_meter_bg());

            // Segmented meter style (like hardware)
            let segment_height = 3.0;
            let segment_gap = 1.0;
            let segments = ((height - 4.0) / (segment_height + segment_gap)) as i32;

            let peak_db = if self.peak > 0.0001 {
                20.0 * self.peak.log10()
            } else {
                -60.0
            };
            let rms_db = if self.rms > 0.0001 {
                20.0 * self.rms.log10()
            } else {
                -60.0
            };
            let hold_db = if self.peak_hold > 0.0001 {
                20.0 * self.peak_hold.log10()
            } else {
                -60.0
            };

            let meter_rect = rect.shrink(2.0);

            for i in 0..segments {
                let segment_db = -60.0 + (i as f32 / segments as f32) * 66.0; // -60 to +6 dB range
                let y = meter_rect.bottom() - (i as f32 + 1.0) * (segment_height + segment_gap);

                let segment_rect = egui::Rect::from_min_size(
                    egui::pos2(meter_rect.left(), y),
                    egui::vec2(meter_rect.width(), segment_height),
                );

                // Determine segment color
                let base_color = if segment_db > 0.0 {
                    theme.vu_meter_red()
                } else if segment_db > -6.0 {
                    egui::Color32::from_rgb(255, 180, 0) // Orange
                } else if segment_db > -18.0 {
                    theme.vu_meter_yellow()
                } else {
                    theme.vu_meter_green()
                };

                let is_rms_lit = rms_db >= segment_db;
                let is_peak_lit = peak_db >= segment_db;
                let is_hold = (hold_db - segment_db).abs() < 2.0;

                let color = if is_hold {
                    // Peak hold indicator - bright
                    base_color
                } else if is_peak_lit {
                    // Peak level - slightly dimmer
                    egui::Color32::from_rgba_unmultiplied(
                        base_color.r(),
                        base_color.g(),
                        base_color.b(),
                        200,
                    )
                } else if is_rms_lit {
                    // RMS level - dimmer
                    egui::Color32::from_rgba_unmultiplied(
                        base_color.r(),
                        base_color.g(),
                        base_color.b(),
                        140,
                    )
                } else {
                    // Off segment
                    egui::Color32::from_gray(35)
                };

                painter.rect_filled(segment_rect, egui::Rounding::same(1.0), color);
            }

            // Clip indicator at top
            let clip_rect = egui::Rect::from_min_size(
                egui::pos2(meter_rect.left(), meter_rect.top()),
                egui::vec2(meter_rect.width(), 4.0),
            );
            let clip_color = if self.clip_indicator {
                egui::Color32::from_rgb(255, 0, 0)
            } else {
                egui::Color32::from_gray(50)
            };
            painter.rect_filled(clip_rect, egui::Rounding::same(1.0), clip_color);
        }
    }
}

/// Hardware telemetry data for mixer strip overlay
#[derive(Debug, Clone, Default)]
pub struct ChannelTelemetry {
    /// Processing latency in milliseconds
    pub latency_ms: f32,
    /// CPU usage percentage (0.0-100.0)
    pub cpu_percent: f32,
    /// Buffer underruns/xruns
    pub xruns: u32,
    /// GhostWave processing active
    pub ghostwave_active: bool,
    /// RTX acceleration active
    pub rtx_active: bool,
    /// VST processing active
    pub vst_active: bool,
}

pub struct ModernChannelStrip {
    pub volume: f32,
    pub gain: f32,
    pub pan: f32,
    pub muted: bool,
    pub solo: bool,
    pub selected_vst: Option<usize>,
    pub levels: [f32; 2], // [peak, rms]
    pub meter: ProfessionalMeter,
    pub gain_knob: HardwareKnob,
    pub telemetry: ChannelTelemetry,
    pub show_telemetry: bool,
}

impl ModernChannelStrip {
    pub fn new() -> Self {
        Self {
            volume: 0.8,
            gain: 0.0,
            pan: 0.0,
            muted: false,
            solo: false,
            selected_vst: None,
            levels: [0.0, 0.0],
            meter: ProfessionalMeter::default(),
            gain_knob: HardwareKnob::new("GAIN", 0.0, -20.0, 20.0, "dB"),
            telemetry: ChannelTelemetry::default(),
            show_telemetry: true, // Show by default
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        theme: &WavelinkTheme,
        channel_name: &str,
        vst_plugins: &[std::path::PathBuf],
        vst_plugin_info: &[crate::phantomlink::VstPluginInfo],
    ) -> ChannelStripResponse {
        let mut response = ChannelStripResponse::default();

        // Update meter with current levels
        self.meter.update(self.levels[0], self.levels[1], 0.016); // ~60fps

        // Channel strip container - Wavelink XLR style
        egui::Frame::none()
            .fill(theme.channel_strip_bg())
            .stroke(egui::Stroke::new(
                1.5,
                if self.solo {
                    theme.accent_secondary
                } else if self.muted {
                    theme.error.linear_multiply(0.5)
                } else {
                    theme.channel_strip_border()
                },
            ))
            .rounding(egui::Rounding::same(12.0))
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.set_min_width(140.0);
                ui.set_max_width(160.0);

                // Channel header - XLR style with icon
                ui.vertical_centered(|ui| {
                    let icon = if channel_name.contains("MIC") {
                        "🎤"
                    } else {
                        "🎸"
                    };
                    ui.label(egui::RichText::new(icon).size(20.0));
                    ui.label(egui::RichText::new(channel_name).size(14.0).strong().color(
                        if self.muted {
                            theme.text_muted
                        } else {
                            theme.accent_primary
                        },
                    ));
                });

                ui.add_space(8.0);

                // Main content: Meter + Controls side by side
                ui.horizontal(|ui| {
                    // Professional VU Meter (left side)
                    self.meter.draw(ui, theme, 180.0);

                    ui.add_space(8.0);

                    // Controls column (right side)
                    ui.vertical(|ui| {
                        // Hardware-style gain knob
                        self.gain_knob.set_value(self.gain);
                        if self.gain_knob.show(ui, theme).changed() {
                            self.gain = self.gain_knob.value();
                            response.gain_changed = true;
                        }

                        ui.add_space(4.0);

                        // Pan indicator (centered dot style)
                        self.draw_pan_indicator(ui, theme);
                        let pan_response = ui.add_sized(
                            [60.0, 20.0],
                            egui::Slider::new(&mut self.pan, -1.0..=1.0).show_value(false),
                        );
                        if pan_response.changed() {
                            response.pan_changed = true;
                        }

                        ui.add_space(8.0);

                        // Volume fader (vertical, Wavelink style)
                        ui.label(
                            egui::RichText::new("VOL")
                                .size(9.0)
                                .color(theme.text_secondary),
                        );
                        let volume_response = ui.add_sized(
                            [40.0, 60.0],
                            egui::Slider::new(&mut self.volume, 0.0..=1.0)
                                .show_value(false)
                                .vertical(),
                        );
                        if volume_response.changed() {
                            response.volume_changed = true;
                        }
                    });
                });

                ui.add_space(8.0);

                // VST Plugin selection with modern dropdown
                ui.label(
                    egui::RichText::new("VST")
                        .size(11.0)
                        .strong()
                        .color(theme.text_secondary),
                );

                let selected_text = if let Some(plugin_idx) = self.selected_vst {
                    vst_plugin_info
                        .get(plugin_idx)
                        .map(|info| info.name.as_str())
                        .or_else(|| {
                            vst_plugins
                                .get(plugin_idx)
                                .and_then(|p| p.file_name())
                                .and_then(|n| n.to_str())
                        })
                        .unwrap_or("Unknown")
                } else {
                    "None"
                };

                egui::ComboBox::from_id_salt(format!("vst_{}", channel_name))
                    .selected_text(selected_text)
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_value(&mut self.selected_vst, None, "None")
                            .clicked()
                        {
                            response.vst_changed = true;
                        }

                        if !vst_plugin_info.is_empty() {
                            for (idx, plugin_info) in vst_plugin_info.iter().enumerate() {
                                let display_name = if plugin_info.vendor.is_empty() {
                                    plugin_info.name.clone()
                                } else {
                                    format!("{}\n{}", plugin_info.name, plugin_info.vendor)
                                };

                                if ui
                                    .selectable_value(
                                        &mut self.selected_vst,
                                        Some(idx),
                                        display_name,
                                    )
                                    .clicked()
                                {
                                    response.vst_changed = true;
                                }
                            }
                        } else {
                            for (idx, plugin) in vst_plugins.iter().enumerate() {
                                let name = plugin
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Unknown");
                                if ui
                                    .selectable_value(&mut self.selected_vst, Some(idx), name)
                                    .clicked()
                                {
                                    response.vst_changed = true;
                                }
                            }
                        }
                    });

                ui.add_space(12.0);

                // Control buttons - Modern status toggles
                ui.horizontal(|ui| {
                    if ui
                        .add(status_toggle_button(
                            "MUTE",
                            self.muted,
                            theme,
                            StatusButtonType::Mute,
                        ))
                        .clicked()
                    {
                        self.muted = !self.muted;
                        response.mute_changed = true;
                    }

                    ui.add_space(4.0);

                    if ui
                        .add(status_toggle_button(
                            "SOLO",
                            self.solo,
                            theme,
                            StatusButtonType::Solo,
                        ))
                        .clicked()
                    {
                        self.solo = !self.solo;
                        response.solo_changed = true;
                    }
                });

                // Hardware telemetry overlay (collapsible)
                if self.show_telemetry {
                    ui.add_space(6.0);
                    self.draw_telemetry_overlay(ui, theme);
                }
            });

        response
    }

    /// Draw the hardware telemetry overlay at the bottom of the channel strip
    fn draw_telemetry_overlay(&self, ui: &mut egui::Ui, theme: &WavelinkTheme) {
        let tel = &self.telemetry;

        egui::Frame::none()
            .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 100))
            .rounding(egui::Rounding::same(4.0))
            .inner_margin(egui::Margin::same(4.0))
            .show(ui, |ui| {
                ui.style_mut().spacing.item_spacing.y = 2.0;

                // Latency indicator
                ui.horizontal(|ui| {
                    let lat_color = if tel.latency_ms < 5.0 {
                        theme.success
                    } else if tel.latency_ms < 15.0 {
                        theme.warning
                    } else {
                        theme.error
                    };
                    ui.label(
                        egui::RichText::new(format!("{:.1}ms", tel.latency_ms))
                            .size(9.0)
                            .color(lat_color),
                    );

                    // XRun indicator
                    if tel.xruns > 0 {
                        ui.label(
                            egui::RichText::new(format!("x{}", tel.xruns))
                                .size(8.0)
                                .color(theme.error),
                        );
                    }
                });

                // Processing indicators
                ui.horizontal(|ui| {
                    // GhostWave indicator
                    if tel.ghostwave_active {
                        let gw_color = if tel.rtx_active {
                            theme.success // RTX active = green
                        } else {
                            theme.warning // CPU fallback = yellow
                        };
                        ui.label(egui::RichText::new("GW").size(8.0).color(gw_color));
                    }

                    // RTX indicator
                    if tel.rtx_active {
                        ui.label(
                            egui::RichText::new("RTX")
                                .size(8.0)
                                .color(theme.accent_secondary),
                        );
                    }

                    // VST indicator
                    if tel.vst_active {
                        ui.label(egui::RichText::new("VST").size(8.0).color(theme.info));
                    }
                });
            });
    }

    fn draw_pan_indicator(&self, ui: &mut egui::Ui, theme: &WavelinkTheme) {
        let size = egui::vec2(60.0, 16.0);
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background track
            painter.rect_filled(rect, egui::Rounding::same(3.0), theme.bg_highlight);

            // Center line
            let center_x = rect.center().x;
            painter.line_segment(
                [
                    egui::pos2(center_x, rect.top() + 2.0),
                    egui::pos2(center_x, rect.bottom() - 2.0),
                ],
                egui::Stroke::new(1.0, theme.text_muted),
            );

            // Pan position indicator
            let pan_x = rect.center().x + self.pan * (rect.width() * 0.4);
            painter.circle_filled(
                egui::pos2(pan_x, rect.center().y),
                5.0,
                theme.accent_primary,
            );

            // L/R labels
            painter.text(
                egui::pos2(rect.left() + 4.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                "L",
                egui::FontId::proportional(8.0),
                theme.text_muted,
            );
            painter.text(
                egui::pos2(rect.right() - 4.0, rect.center().y),
                egui::Align2::RIGHT_CENTER,
                "R",
                egui::FontId::proportional(8.0),
                theme.text_muted,
            );
        }
    }
}

#[derive(Default)]
pub struct ChannelStripResponse {
    pub volume_changed: bool,
    pub gain_changed: bool,
    pub pan_changed: bool,
    pub mute_changed: bool,
    pub solo_changed: bool,
    pub vst_changed: bool,
}

pub struct ModernButton;

impl ModernButton {
    pub fn primary(text: &str) -> egui::Button<'_> {
        egui::Button::new(
            egui::RichText::new(text)
                .size(16.0) // Larger text
                .strong(),
        )
        .min_size(egui::vec2(140.0, 44.0)) // Larger touch target
    }

    pub fn secondary(text: &str) -> egui::Button<'_> {
        egui::Button::new(
            egui::RichText::new(text).size(14.0), // Larger text
        )
        .min_size(egui::vec2(100.0, 38.0)) // Larger touch target
    }

    pub fn icon_button(icon: &str, text: &str) -> egui::Button<'static> {
        egui::Button::new(
            egui::RichText::new(format!("{} {}", icon, text)).size(15.0), // Larger text
        )
        .min_size(egui::vec2(120.0, 40.0)) // Larger touch target
    }
}

pub struct StatusIndicator;

impl StatusIndicator {
    pub fn show(ui: &mut egui::Ui, theme: &WavelinkTheme, status: &str, is_active: bool) {
        let color = if is_active {
            theme.green_primary
        } else {
            theme.error
        }; // Use green instead of success
        let icon = "●";

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(icon)
                    .size(18.0) // Larger for touch
                    .color(color),
            );
            ui.label(
                egui::RichText::new(status)
                    .size(14.0) // Larger for touch
                    .color(if is_active {
                        theme.text_primary
                    } else {
                        theme.text_muted
                    }),
            );
        });
    }
}

pub fn glow_button(text: &str, color: egui::Color32) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| {
        let desired_size = egui::vec2(140.0, 44.0); // Larger touch target
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Glow effect
            for i in 0..5 {
                let glow_radius = 4.0 + i as f32 * 2.5; // Larger glow
                let glow_alpha = 35 - i * 7;
                let glow_color = egui::Color32::from_rgba_premultiplied(
                    color.r(),
                    color.g(),
                    color.b(),
                    glow_alpha,
                );
                painter.rect_filled(
                    rect.expand(glow_radius),
                    egui::Rounding::same(10.0 + glow_radius), // More rounded
                    glow_color,
                );
            }

            // Button background with translucency
            let bg_color = if response.hovered() {
                egui::Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 120)
            } else {
                egui::Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 80)
            };

            painter.rect_filled(rect, egui::Rounding::same(10.0), bg_color);
            painter.rect_stroke(
                rect,
                egui::Rounding::same(10.0),
                egui::Stroke::new(2.0, color),
            );

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
                egui::FontId::proportional(16.0), // Larger text
                text_color,
            );
        }

        response
    }
}

// Enhanced modern button with glass effect and consistent theming
pub fn modern_glass_button<'a>(
    text: &'a str,
    theme: &'a WavelinkTheme,
    enabled: bool,
) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| {
        let desired_size = egui::vec2(ui.available_width().min(200.0), 36.0);
        let response = ui.allocate_response(desired_size, egui::Sense::click());
        let rect = response.rect;

        if ui.is_rect_visible(rect) {
            let bg_color = if !enabled {
                theme.status_inactive()
            } else if response.hovered() {
                theme.glass_button_hover()
            } else if response.is_pointer_button_down_on() {
                theme.glass_button_active()
            } else {
                theme.glass_button_bg()
            };

            let text_color = if enabled {
                theme.text_primary
            } else {
                theme.text_muted
            };

            // Draw glass effect background
            ui.painter().rect(
                rect,
                egui::Rounding::same(12.0),
                bg_color,
                egui::Stroke::new(
                    1.5,
                    if enabled {
                        theme.green_primary
                    } else {
                        theme.medium_blue
                    },
                ),
            );

            // Add subtle inner glow for glass effect
            if enabled && response.hovered() {
                let inner_rect = rect.shrink(2.0);
                ui.painter().rect(
                    inner_rect,
                    egui::Rounding::same(10.0),
                    egui::Color32::TRANSPARENT,
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(34, 197, 94, 60)),
                );
            }

            // Draw text with proper alignment
            let text_rect = rect.shrink(8.0);
            ui.painter().text(
                text_rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(14.0),
                text_color,
            );
        }

        response
    }
}

// Status indicator button (for MUTE, SOLO, etc.)
pub fn status_toggle_button<'a>(
    text: &'a str,
    active: bool,
    theme: &'a WavelinkTheme,
    button_type: StatusButtonType,
) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| {
        let desired_size = egui::vec2(60.0, 28.0);
        let response = ui.allocate_response(desired_size, egui::Sense::click());
        let rect = response.rect;

        if ui.is_rect_visible(rect) {
            let (bg_color, text_color, border_color) = match button_type {
                StatusButtonType::Mute => {
                    if active {
                        (theme.error, theme.deep_blue, theme.error)
                    } else {
                        (
                            theme.translucent_input_bg(),
                            theme.text_secondary,
                            theme.medium_blue,
                        )
                    }
                }
                StatusButtonType::Solo => {
                    if active {
                        (theme.warning, theme.deep_blue, theme.warning)
                    } else {
                        (
                            theme.translucent_input_bg(),
                            theme.text_secondary,
                            theme.medium_blue,
                        )
                    }
                }
                StatusButtonType::Record => {
                    if active {
                        (theme.error, theme.text_primary, theme.error)
                    } else {
                        (
                            theme.translucent_input_bg(),
                            theme.text_secondary,
                            theme.medium_blue,
                        )
                    }
                }
                StatusButtonType::Active => {
                    if active {
                        (theme.green_primary, theme.deep_blue, theme.green_primary)
                    } else {
                        (
                            theme.translucent_input_bg(),
                            theme.text_secondary,
                            theme.medium_blue,
                        )
                    }
                }
            };

            // Enhance colors on hover
            let final_bg = if response.hovered() && !active {
                egui::Color32::from_rgba_premultiplied(
                    bg_color.r(),
                    bg_color.g(),
                    bg_color.b(),
                    150,
                )
            } else {
                bg_color
            };

            ui.painter().rect(
                rect,
                egui::Rounding::same(8.0),
                final_bg,
                egui::Stroke::new(1.5, border_color),
            );

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(11.0),
                text_color,
            );
        }

        response
    }
}

#[derive(Clone, Copy)]
pub enum StatusButtonType {
    Mute,
    Solo,
    Record,
    Active,
}

// Enhanced glow button with consistent theming
pub fn enhanced_glow_button<'a>(
    text: &'a str,
    theme: &'a WavelinkTheme,
    style: GlowButtonStyle,
) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| {
        let desired_size = egui::vec2(ui.available_width().min(140.0), 32.0);
        let response = ui.allocate_response(desired_size, egui::Sense::click());
        let rect = response.rect;

        if ui.is_rect_visible(rect) {
            let (base_color, glow_color) = match style {
                GlowButtonStyle::Primary => (theme.green_primary, theme.green_glow),
                GlowButtonStyle::Secondary => (theme.info, theme.light_blue),
                GlowButtonStyle::Success => (theme.success, theme.green_glow),
                GlowButtonStyle::Warning => (theme.warning, egui::Color32::from_rgb(251, 146, 60)),
                GlowButtonStyle::Danger => (theme.error, egui::Color32::from_rgb(220, 38, 38)),
            };

            let animation_progress = if response.hovered() { 1.0 } else { 0.6 };

            // Outer glow effect
            if response.hovered() {
                for i in 0..3 {
                    let alpha = 30 - (i * 10);
                    let glow_expand = (i + 1) as f32 * 2.0;
                    ui.painter().rect(
                        rect.expand(glow_expand),
                        egui::Rounding::same(14.0 + glow_expand),
                        egui::Color32::from_rgba_premultiplied(
                            glow_color.r(),
                            glow_color.g(),
                            glow_color.b(),
                            alpha,
                        ),
                        egui::Stroke::NONE,
                    );
                }
            }

            // Main button
            let button_color = if response.is_pointer_button_down_on() {
                egui::Color32::from_rgba_premultiplied(
                    (base_color.r() as f32 * 0.8) as u8,
                    (base_color.g() as f32 * 0.8) as u8,
                    (base_color.b() as f32 * 0.8) as u8,
                    base_color.a(),
                )
            } else {
                egui::Color32::from_rgba_premultiplied(
                    base_color.r(),
                    base_color.g(),
                    base_color.b(),
                    (255.0 * animation_progress) as u8,
                )
            };

            ui.painter().rect(
                rect,
                egui::Rounding::same(10.0),
                button_color,
                egui::Stroke::new(1.5, glow_color),
            );

            // Text with proper contrast
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(13.0),
                theme.deep_blue,
            );
        }

        response
    }
}

#[derive(Clone, Copy)]
pub enum GlowButtonStyle {
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
}
