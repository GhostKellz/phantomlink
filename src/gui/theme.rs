use eframe::egui;

pub struct WavelinkTheme {
    // Primary colors - Deep blue gradient like Wavelink
    pub deep_blue: egui::Color32,
    pub medium_blue: egui::Color32,
    pub light_blue: egui::Color32,
    
    // Accent colors - Professional green
    pub green_primary: egui::Color32,
    pub green_secondary: egui::Color32,
    pub green_glow: egui::Color32,
    
    // UI colors
    pub background: egui::Color32,
    pub panel_bg: egui::Color32,
    pub card_bg: egui::Color32,
    pub input_bg: egui::Color32,
    
    // Text colors
    pub text_primary: egui::Color32,
    pub text_secondary: egui::Color32,
    pub text_muted: egui::Color32,
    
    // Status colors
    pub success: egui::Color32,
    pub warning: egui::Color32,
    pub error: egui::Color32,
    pub info: egui::Color32,
}

impl WavelinkTheme {
    pub fn new() -> Self {
        Self {
            // Deep blue palette inspired by Wavelink
            deep_blue: egui::Color32::from_rgb(11, 17, 35),        // Very deep blue background
            medium_blue: egui::Color32::from_rgb(18, 28, 52),      // Medium blue for panels
            light_blue: egui::Color32::from_rgb(28, 42, 78),       // Lighter blue for cards
            
            // Professional green accents
            green_primary: egui::Color32::from_rgb(34, 197, 94),    // Primary green
            green_secondary: egui::Color32::from_rgb(74, 222, 128), // Lighter green
            green_glow: egui::Color32::from_rgb(22, 163, 74),       // Deeper green glow
            
            // UI backgrounds
            background: egui::Color32::from_rgb(8, 12, 24),         // Deepest background
            panel_bg: egui::Color32::from_rgb(15, 22, 40),          // Panel background
            card_bg: egui::Color32::from_rgb(22, 32, 58),           // Card background
            input_bg: egui::Color32::from_rgb(25, 36, 64),          // Input field background
            
            // Text hierarchy
            text_primary: egui::Color32::from_rgb(240, 242, 247),   // Primary text
            text_secondary: egui::Color32::from_rgb(180, 188, 200), // Secondary text
            text_muted: egui::Color32::from_rgb(120, 128, 140),     // Muted text
            
            // Status colors
            success: egui::Color32::from_rgb(52, 211, 153),         // Green success
            warning: egui::Color32::from_rgb(251, 191, 36),         // Yellow warning
            error: egui::Color32::from_rgb(248, 113, 113),          // Red error
            info: egui::Color32::from_rgb(96, 165, 250),            // Blue info
        }
    }
    
    pub fn apply(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        // Overall dark theme base
        style.visuals = egui::Visuals::dark();
        
        // Main backgrounds
        style.visuals.window_fill = self.background;
        style.visuals.panel_fill = self.panel_bg;
        style.visuals.faint_bg_color = self.card_bg;
        
        // Window styling
        style.visuals.window_stroke = egui::Stroke::new(2.0, self.light_blue);
        style.visuals.window_rounding = egui::Rounding::same(12.0);
        
        // Widget styling - Inactive state
        style.visuals.widgets.noninteractive.bg_fill = self.input_bg;
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, self.text_secondary);
        style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
        
        // Widget styling - Inactive (buttons, inputs)
        style.visuals.widgets.inactive.bg_fill = self.card_bg;
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, self.text_primary);
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, self.medium_blue);
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
        
        // Widget styling - Hovered
        style.visuals.widgets.hovered.bg_fill = self.light_blue;
        style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, self.green_secondary);
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(2.0, self.green_primary);
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
        
        // Widget styling - Active/Pressed
        style.visuals.widgets.active.bg_fill = self.green_primary;
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, self.deep_blue);
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, self.green_glow);
        style.visuals.widgets.active.rounding = egui::Rounding::same(8.0);
        
        // Open widgets (dropdowns, etc.)
        style.visuals.widgets.open.bg_fill = self.medium_blue;
        style.visuals.widgets.open.fg_stroke = egui::Stroke::new(1.5, self.green_secondary);
        style.visuals.widgets.open.bg_stroke = egui::Stroke::new(2.0, self.green_primary);
        style.visuals.widgets.open.rounding = egui::Rounding::same(8.0);
        
        // Text colors
        style.visuals.override_text_color = Some(self.text_primary);
        style.visuals.hyperlink_color = self.green_secondary;
        
        // Selection styling
        style.visuals.selection.bg_fill = egui::Color32::from_rgba_premultiplied(34, 197, 94, 60);
        style.visuals.selection.stroke = egui::Stroke::new(1.5, self.green_primary);
        
        // Status colors
        style.visuals.error_fg_color = self.error;
        style.visuals.warn_fg_color = self.warning;
        
        // Menu and popup styling
        style.visuals.menu_rounding = egui::Rounding::same(10.0);
        style.visuals.popup_shadow = egui::epaint::Shadow {
            offset: egui::vec2(4.0, 8.0),
            blur: 16.0,
            spread: 2.0,
            color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 100),
        };
        
        // Spacing for touch-friendly modern look
        style.spacing.button_padding = egui::vec2(20.0, 14.0);  // Larger touch targets
        style.spacing.item_spacing = egui::vec2(16.0, 12.0);    // More spacing
        style.spacing.indent = 28.0;
        style.spacing.window_margin = egui::Margin::same(20.0);
        style.spacing.menu_margin = egui::Margin::same(12.0);
        style.spacing.slider_width = 24.0;  // Wider sliders for touch
        
        // Scrollbar styling
        style.visuals.widgets.noninteractive.bg_fill = self.medium_blue;
        
        ctx.set_style(style);
    }
    
    // Helper methods for custom colors with translucency
    pub fn channel_strip_bg(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(22, 32, 58, 200)  // Translucent card bg
    }
    
    pub fn channel_strip_border(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(28, 42, 78, 180)  // Translucent light blue
    }
    
    pub fn translucent_panel_bg(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(15, 22, 40, 220)  // More opaque panel bg
    }
    
    pub fn translucent_input_bg(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(25, 36, 64, 200)  // Translucent input bg
    }
    
    pub fn glass_button_bg(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(34, 197, 94, 40)  // Subtle green glass effect
    }
    
    pub fn glass_button_hover(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(34, 197, 94, 80)  // Stronger green glass on hover
    }
    
    pub fn glass_button_active(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(34, 197, 94, 120) // Full green glass when pressed
    }
    
    // Status indicator colors
    pub fn status_active(&self) -> egui::Color32 {
        self.green_primary
    }
    
    pub fn status_inactive(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(120, 128, 140, 180)
    }
    
    pub fn status_warning(&self) -> egui::Color32 {
        self.warning
    }
    
    pub fn status_error(&self) -> egui::Color32 {
        self.error
    }
    
    pub fn translucent_deep_bg(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(8, 12, 24, 220)   // Translucent deep background
    }
    
    pub fn vu_meter_bg(&self) -> egui::Color32 {
        self.deep_blue
    }
    
    pub fn vu_meter_green(&self) -> egui::Color32 {
        self.success
    }
    
    pub fn vu_meter_yellow(&self) -> egui::Color32 {
        self.warning
    }
    
    pub fn vu_meter_red(&self) -> egui::Color32 {
        self.error
    }
    
    pub fn glow_effect(&self, base_color: egui::Color32, intensity: f32) -> egui::Color32 {
        let [r, g, b, a] = base_color.to_array();
        let factor = 1.0 + intensity * 0.3;
        egui::Color32::from_rgba_premultiplied(
            ((r as f32 * factor).min(255.0)) as u8,
            ((g as f32 * factor).min(255.0)) as u8,
            ((b as f32 * factor).min(255.0)) as u8,
            a,
        )
    }
}

// Legacy alias for backwards compatibility
pub type SpaceTheme = WavelinkTheme;