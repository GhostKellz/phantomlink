use eframe::egui;
use crate::gui::theme::WavelinkTheme;

pub struct ModernChannelStrip {
    pub volume: f32,
    pub gain: f32,
    pub pan: f32,
    pub muted: bool,
    pub solo: bool,
    pub selected_vst: Option<usize>,
    pub levels: [f32; 2], // [peak, rms]
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
        
        // Channel strip container with modern translucent styling
        egui::Frame::none()
            .fill(theme.channel_strip_bg())
            .stroke(egui::Stroke::new(1.5, theme.channel_strip_border()))
            .rounding(egui::Rounding::same(16.0))  // More rounded for modern look
            .inner_margin(egui::Margin::same(16.0))  // More padding for touch
            .show(ui, |ui| {
                ui.set_min_width(160.0);  // Wider for touch
                ui.set_max_width(180.0);
                
                // Channel header with modern typography
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new(channel_name)
                            .size(16.0)  // Larger text for touch
                            .strong()
                            .color(theme.green_primary)  // Green accent
                    );
                });
                
                ui.add_space(8.0);
                
                // VU Meter - Modern vertical style
                self.draw_modern_vu_meter(ui, theme);
                
                ui.add_space(12.0);
                
                // Gain control with modern styling
                ui.label(
                    egui::RichText::new("GAIN")
                        .size(11.0)
                        .strong()
                        .color(theme.text_secondary)
                );
                
                let gain_response = ui.add_sized(
                    [ui.available_width(), 100.0],  // Taller for easier touch control
                    egui::Slider::new(&mut self.gain, -20.0..=20.0)
                        .suffix(" dB")
                        .show_value(true)
                        .vertical()
                );
                
                if gain_response.changed() {
                    response.gain_changed = true;
                }
                
                ui.add_space(12.0);
                
                // Pan control with larger size
                ui.label(
                    egui::RichText::new("PAN")
                        .size(11.0)
                        .strong()
                        .color(theme.text_secondary)
                );
                
                let pan_response = ui.add_sized(
                    [ui.available_width(), 30.0],  // Taller for touch
                    egui::Slider::new(&mut self.pan, -1.0..=1.0)
                        .show_value(false)
                );
                
                if pan_response.changed() {
                    response.pan_changed = true;
                }
                
                ui.add_space(12.0);
                
                // Volume fader - Wavelink style with more height
                ui.label(
                    egui::RichText::new("VOLUME")
                        .size(11.0)
                        .strong()
                        .color(theme.text_secondary)
                );
                
                let volume_response = ui.add_sized(
                    [ui.available_width(), 120.0],  // Taller for better control
                    egui::Slider::new(&mut self.volume, 0.0..=1.0)
                        .show_value(false)
                        .vertical()
                );
                
                if volume_response.changed() {
                    response.volume_changed = true;
                }
                
                ui.add_space(12.0);
                
                // VST Plugin selection with modern dropdown
                ui.label(
                    egui::RichText::new("VST")
                        .size(11.0)
                        .strong()
                        .color(theme.text_secondary)
                );
                
                let selected_text = if let Some(plugin_idx) = self.selected_vst {
                    vst_plugin_info.get(plugin_idx)
                        .map(|info| info.name.as_str())
                        .or_else(|| {
                            vst_plugins.get(plugin_idx)
                                .and_then(|p| p.file_name())
                                .and_then(|n| n.to_str())
                        })
                        .unwrap_or("Unknown")
                } else {
                    "None"
                };
                
                egui::ComboBox::from_id_source(format!("vst_{}", channel_name))
                    .selected_text(selected_text)
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut self.selected_vst, None, "None").clicked() {
                            response.vst_changed = true;
                        }
                        
                        if !vst_plugin_info.is_empty() {
                            for (idx, plugin_info) in vst_plugin_info.iter().enumerate() {
                                let display_name = if plugin_info.vendor.is_empty() {
                                    plugin_info.name.clone()
                                } else {
                                    format!("{}\n{}", plugin_info.name, plugin_info.vendor)
                                };
                                
                                if ui.selectable_value(&mut self.selected_vst, Some(idx), display_name).clicked() {
                                    response.vst_changed = true;
                                }
                            }
                        } else {
                            for (idx, plugin) in vst_plugins.iter().enumerate() {
                                let name = plugin.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Unknown");
                                if ui.selectable_value(&mut self.selected_vst, Some(idx), name).clicked() {
                                    response.vst_changed = true;
                                }
                            }
                        }
                    });
                
                ui.add_space(12.0);
                
                // Control buttons - Touch-friendly Wavelink style
                ui.horizontal(|ui| {
                    let mute_color = if self.muted { theme.error } else { theme.text_secondary };
                    if ui.add(
                        egui::Button::new(
                            egui::RichText::new("MUTE")
                                .size(12.0)
                                .strong()
                                .color(mute_color)
                        ).min_size(egui::vec2(65.0, 32.0))  // Larger touch target
                    ).clicked() {
                        self.muted = !self.muted;
                        response.mute_changed = true;
                    }
                    
                    let solo_color = if self.solo { theme.green_primary } else { theme.text_secondary };
                    if ui.add(
                        egui::Button::new(
                            egui::RichText::new("SOLO")
                                .size(12.0)
                                .strong()
                                .color(solo_color)
                        ).min_size(egui::vec2(65.0, 32.0))  // Larger touch target
                    ).clicked() {
                        self.solo = !self.solo;
                        response.solo_changed = true;
                    }
                });
            });
        
        response
    }
    
    fn draw_modern_vu_meter(&self, ui: &mut egui::Ui, theme: &WavelinkTheme) {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 60.0), egui::Sense::hover());
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            // Background
            painter.rect_filled(
                rect,
                egui::Rounding::same(4.0),
                theme.vu_meter_bg(),
            );
            
            // Border
            painter.rect_stroke(
                rect,
                egui::Rounding::same(4.0),
                egui::Stroke::new(1.0, theme.light_blue),
            );
            
            // Calculate levels in dB
            let peak_db = if self.levels[0] > 0.0001 {
                20.0 * self.levels[0].log10().max(-60.0)
            } else {
                -60.0
            };
            
            let rms_db = if self.levels[1] > 0.0001 {
                20.0 * self.levels[1].log10().max(-60.0)
            } else {
                -60.0
            };
            
            // Normalize to 0-1 range (-60dB to 0dB)
            let peak_norm = ((peak_db + 60.0) / 60.0).clamp(0.0, 1.0);
            let rms_norm = ((rms_db + 60.0) / 60.0).clamp(0.0, 1.0);
            
            let meter_rect = rect.shrink(2.0);
            let peak_height = meter_rect.height() * peak_norm;
            let rms_height = meter_rect.height() * rms_norm;
            
            // RMS level (background)
            if rms_norm > 0.0 {
                let rms_rect = egui::Rect::from_min_size(
                    egui::pos2(meter_rect.left(), meter_rect.bottom() - rms_height),
                    egui::vec2(meter_rect.width() * 0.6, rms_height),
                );
                
                let rms_color = if rms_db > -6.0 {
                    theme.vu_meter_red()
                } else if rms_db > -18.0 {
                    theme.vu_meter_yellow()
                } else {
                    theme.vu_meter_green()
                };
                
                painter.rect_filled(rms_rect, egui::Rounding::same(2.0), rms_color);
            }
            
            // Peak level (foreground)
            if peak_norm > 0.0 {
                let peak_rect = egui::Rect::from_min_size(
                    egui::pos2(meter_rect.right() - meter_rect.width() * 0.3, meter_rect.bottom() - peak_height),
                    egui::vec2(meter_rect.width() * 0.3, peak_height),
                );
                
                let peak_color = if peak_db > -6.0 {
                    theme.vu_meter_red()
                } else if peak_db > -18.0 {
                    theme.vu_meter_yellow()
                } else {
                    theme.vu_meter_green()
                };
                
                painter.rect_filled(peak_rect, egui::Rounding::same(2.0), peak_color);
            }
            
            // dB scale markers
            for &db in &[-60, -40, -20, -6, 0] {
                let y_pos = meter_rect.bottom() - (meter_rect.height() * ((db + 60) as f32 / 60.0));
                painter.line_segment(
                    [egui::pos2(meter_rect.right() + 2.0, y_pos), egui::pos2(meter_rect.right() + 6.0, y_pos)],
                    egui::Stroke::new(1.0, theme.text_muted),
                );
                
                if db == 0 || db == -20 || db == -40 {
                    painter.text(
                        egui::pos2(meter_rect.right() + 8.0, y_pos - 6.0),
                        egui::Align2::LEFT_CENTER,
                        format!("{}", db),
                        egui::FontId::proportional(8.0),
                        theme.text_muted,
                    );
                }
            }
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
    pub fn primary(text: &str) -> egui::Button {
        egui::Button::new(
            egui::RichText::new(text)
                .size(16.0)  // Larger text
                .strong()
        ).min_size(egui::vec2(140.0, 44.0))  // Larger touch target
    }
    
    pub fn secondary(text: &str) -> egui::Button {
        egui::Button::new(
            egui::RichText::new(text)
                .size(14.0)  // Larger text
        ).min_size(egui::vec2(100.0, 38.0))  // Larger touch target
    }
    
    pub fn icon_button(icon: &str, text: &str) -> egui::Button<'static> {
        egui::Button::new(
            egui::RichText::new(format!("{} {}", icon, text))
                .size(15.0)  // Larger text
        ).min_size(egui::vec2(120.0, 40.0))  // Larger touch target
    }
}

pub struct StatusIndicator;

impl StatusIndicator {
    pub fn show(ui: &mut egui::Ui, theme: &WavelinkTheme, status: &str, is_active: bool) {
        let color = if is_active { theme.green_primary } else { theme.error };  // Use green instead of success
        let icon = if is_active { "●" } else { "●" };
        
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(icon)
                    .size(18.0)  // Larger for touch
                    .color(color)
            );
            ui.label(
                egui::RichText::new(status)
                    .size(14.0)  // Larger for touch
                    .color(if is_active { theme.text_primary } else { theme.text_muted })
            );
        });
    }
}

pub fn glow_button(text: &str, color: egui::Color32) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| {
        let desired_size = egui::vec2(140.0, 44.0);  // Larger touch target
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            
            // Glow effect
            for i in 0..5 {
                let glow_radius = 4.0 + i as f32 * 2.5;  // Larger glow
                let glow_alpha = 35 - i * 7;
                let glow_color = egui::Color32::from_rgba_premultiplied(
                    color.r(), color.g(), color.b(), glow_alpha
                );
                painter.rect_filled(
                    rect.expand(glow_radius),
                    egui::Rounding::same(10.0 + glow_radius),  // More rounded
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
            painter.rect_stroke(rect, egui::Rounding::same(10.0), egui::Stroke::new(2.0, color));
            
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
                egui::FontId::proportional(16.0),  // Larger text
                text_color,
            );
        }
        
        response
    }
}
