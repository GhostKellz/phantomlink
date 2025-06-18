use eframe::egui;

pub struct SpaceTheme {
    pub primary_color: egui::Color32,
    pub secondary_color: egui::Color32,
    pub accent_color: egui::Color32,
    pub background_color: egui::Color32,
    pub surface_color: egui::Color32,
    pub text_color: egui::Color32,
    pub muted_text_color: egui::Color32,
}

impl SpaceTheme {
    pub fn new() -> Self {
        Self {
            primary_color: egui::Color32::from_rgb(80, 217, 176),      // Mint green
            secondary_color: egui::Color32::from_rgb(19, 158, 209),    // Cyan blue
            accent_color: egui::Color32::from_rgb(255, 120, 180),      // Pink accent
            background_color: egui::Color32::from_rgba_premultiplied(5, 8, 20, 200), // Deep space
            surface_color: egui::Color32::from_rgba_premultiplied(15, 25, 45, 180),  // Panel bg
            text_color: egui::Color32::from_rgb(220, 220, 220),        // Light text
            muted_text_color: egui::Color32::from_rgb(140, 140, 140),  // Muted text
        }
    }
    
    pub fn apply(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        // Window styling
        style.visuals.window_fill = self.background_color;
        style.visuals.panel_fill = egui::Color32::TRANSPARENT;
        style.visuals.window_stroke = egui::Stroke::new(1.0, self.primary_color);
        // Window shadow removed due to API changes
        
        // Widget styling
        style.visuals.widgets.noninteractive.bg_fill = self.surface_color;
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, self.text_color);
        
        style.visuals.widgets.inactive.bg_fill = self.surface_color;
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, self.muted_text_color);
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(80, 217, 176, 60));
        
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgba_premultiplied(80, 217, 176, 40);
        style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, self.primary_color);
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(2.0, self.primary_color);
        
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgba_premultiplied(80, 217, 176, 80);
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, self.primary_color);
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(2.5, self.primary_color);
        
        // Button styling
        style.visuals.widgets.open.bg_fill = egui::Color32::from_rgba_premultiplied(19, 158, 209, 60);
        style.visuals.widgets.open.fg_stroke = egui::Stroke::new(1.5, self.secondary_color);
        style.visuals.widgets.open.bg_stroke = egui::Stroke::new(2.0, self.secondary_color);
        
        // Text colors
        style.visuals.override_text_color = Some(self.text_color);
        
        // Selection colors
        style.visuals.selection.bg_fill = egui::Color32::from_rgba_premultiplied(80, 217, 176, 50);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, self.primary_color);
        
        // Hyperlink colors
        style.visuals.hyperlink_color = self.secondary_color;
        
        // Error/warning colors
        style.visuals.error_fg_color = egui::Color32::from_rgb(255, 100, 100);
        style.visuals.warn_fg_color = egui::Color32::from_rgb(255, 200, 100);
        
        // Slider styling
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
        style.visuals.widgets.active.rounding = egui::Rounding::same(8.0);
        
        // Menu styling
        style.visuals.menu_rounding = egui::Rounding::same(8.0);
        
        // Spacing
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.indent = 20.0;
        
        ctx.set_style(style);
    }
    
    pub fn glow_color(&self, base_color: egui::Color32, intensity: f32) -> egui::Color32 {
        let [r, g, b, a] = base_color.to_array();
        egui::Color32::from_rgba_premultiplied(
            (r as f32 * (1.0 + intensity * 0.5)) as u8,
            (g as f32 * (1.0 + intensity * 0.5)) as u8,
            (b as f32 * (1.0 + intensity * 0.5)) as u8,
            a,
        )
    }
}