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
                
                // Control buttons - Modern status toggles
                ui.horizontal(|ui| {
                    if ui.add(status_toggle_button("🔇 MUTE", self.muted, theme, StatusButtonType::Mute)).clicked() {
                        self.muted = !self.muted;
                        response.mute_changed = true;
                    }
                    
                    ui.add_space(4.0);
                    
                    if ui.add(status_toggle_button("🎯 SOLO", self.solo, theme, StatusButtonType::Solo)).clicked() {
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
            
            // RMS level (background) with gradient effect
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
                
                // Create gradient effect by drawing multiple segments
                let segments = 10;
                for i in 0..segments {
                    let segment_height = rms_height / segments as f32;
                    let segment_y = rms_rect.bottom() - (i + 1) as f32 * segment_height;
                    let alpha = (255.0 * (0.4 + 0.6 * (i as f32 / segments as f32))) as u8;
                    
                    let segment_rect = egui::Rect::from_min_size(
                        egui::pos2(rms_rect.left(), segment_y),
                        egui::vec2(rms_rect.width(), segment_height),
                    );
                    
                    painter.rect_filled(
                        segment_rect, 
                        egui::Rounding::same(1.0), 
                        egui::Color32::from_rgba_premultiplied(rms_color.r(), rms_color.g(), rms_color.b(), alpha)
                    );
                }
            }
            
            // Peak level (foreground) with smooth gradient
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
                
                // Add subtle glow effect for better visibility
                painter.rect_filled(
                    peak_rect.expand(1.0), 
                    egui::Rounding::same(3.0), 
                    egui::Color32::from_rgba_premultiplied(peak_color.r(), peak_color.g(), peak_color.b(), 40)
                );
                painter.rect_filled(peak_rect, egui::Rounding::same(2.0), peak_color);
                
                // Peak hold indicator
                if peak_norm > 0.9 {
                    let peak_line_y = meter_rect.bottom() - peak_height;
                    painter.line_segment(
                        [egui::pos2(meter_rect.left(), peak_line_y), egui::pos2(meter_rect.right(), peak_line_y)],
                        egui::Stroke::new(2.0, theme.vu_meter_red())
                    );
                }
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
    
    pub fn icon_button<'a>(icon: &'a str, text: &'a str) -> egui::Button<'a> {
        egui::Button::new(
            egui::RichText::new(format!("{} {}", icon, text))
                .size(15.0)  // Larger text
        ).min_size(egui::vec2(120.0, 40.0))  // Larger touch target
    }
    
    pub fn animated_button(text: &str, hovered: bool) -> egui::Button {
        let size = if hovered { 16.5 } else { 16.0 };
        let width = if hovered { 145.0 } else { 140.0 };
        let height = if hovered { 46.0 } else { 44.0 };
        egui::Button::new(
            egui::RichText::new(text)
                .size(size)
                .strong()
        ).min_size(egui::vec2(width, height))
    }
}

pub struct StatusIndicator;

impl StatusIndicator {
    pub fn new(status: &str, color: egui::Color32) -> impl egui::Widget + '_ {
        move |ui: &mut egui::Ui| {
            let response = ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("●")
                        .size(18.0)
                        .color(color)
                );
                ui.label(
                    egui::RichText::new(status)
                        .size(14.0)
                        .color(color)
                );
            });
            response.response
        }
    }
    
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

// Enhanced modern button with glass effect and consistent theming
pub fn modern_glass_button<'a>(text: &'a str, theme: &'a WavelinkTheme, enabled: bool) -> impl egui::Widget + 'a {
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
                egui::Stroke::new(1.5, if enabled { theme.green_primary } else { theme.medium_blue }),
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
pub fn status_toggle_button<'a>(text: &'a str, active: bool, theme: &'a WavelinkTheme, button_type: StatusButtonType) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| {
        let desired_size = egui::vec2(60.0, 28.0);
        let response = ui.allocate_response(desired_size, egui::Sense::click());
        let rect = response.rect;
        
        if ui.is_rect_visible(rect) {
            let (bg_color, text_color, border_color) = match button_type {
                StatusButtonType::Mute => if active {
                    (theme.error, theme.deep_blue, theme.error)
                } else {
                    (theme.translucent_input_bg(), theme.text_secondary, theme.medium_blue)
                },
                StatusButtonType::Solo => if active {
                    (theme.warning, theme.deep_blue, theme.warning)
                } else {
                    (theme.translucent_input_bg(), theme.text_secondary, theme.medium_blue)
                },
                StatusButtonType::Record => if active {
                    (theme.error, theme.text_primary, theme.error)
                } else {
                    (theme.translucent_input_bg(), theme.text_secondary, theme.medium_blue)
                },
                StatusButtonType::Active => if active {
                    (theme.green_primary, theme.deep_blue, theme.green_primary)
                } else {
                    (theme.translucent_input_bg(), theme.text_secondary, theme.medium_blue)
                },
            };
            
            // Enhance colors on hover
            let final_bg = if response.hovered() && !active {
                egui::Color32::from_rgba_premultiplied(bg_color.r(), bg_color.g(), bg_color.b(), 150)
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
pub fn enhanced_glow_button<'a>(text: &'a str, theme: &'a WavelinkTheme, style: GlowButtonStyle) -> impl egui::Widget + 'a {
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
                        egui::Color32::from_rgba_premultiplied(glow_color.r(), glow_color.g(), glow_color.b(), alpha),
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
