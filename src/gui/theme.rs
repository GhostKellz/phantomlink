//! Tokyo Night theme system for PhantomLink.
//!
//! Three variants available:
//! - **Night** (Default) - Deep blues with cyan/purple accents
//! - **Storm** - Lighter blue-shifted variant
//! - **Moon** - Warmer with purple/rose tints

#![allow(dead_code)]

use eframe::egui;
use serde::{Deserialize, Serialize};

/// Tokyo Night theme variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[allow(clippy::enum_variant_names)]
pub enum ThemePreset {
    /// Tokyo Night - Deep dark blue base (default)
    #[default]
    TokyoNight,
    /// Tokyo Night Storm - Lighter blue-shifted
    TokyoNightStorm,
    /// Tokyo Night Moon - Warmer with purple undertones
    TokyoNightMoon,
}

impl ThemePreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::TokyoNight => "Night",
            Self::TokyoNightStorm => "Storm",
            Self::TokyoNightMoon => "Moon",
        }
    }

    pub fn all() -> &'static [ThemePreset] {
        &[
            Self::TokyoNight,
            Self::TokyoNightStorm,
            Self::TokyoNightMoon,
        ]
    }
}

/// Main theme structure for PhantomLink
pub struct WavelinkTheme {
    pub preset: ThemePreset,

    // Background colors
    pub bg: egui::Color32,
    pub bg_dark: egui::Color32,
    pub bg_highlight: egui::Color32,
    pub panel_bg: egui::Color32,
    pub card_bg: egui::Color32,
    pub input_bg: egui::Color32,

    // Primary accent colors
    pub accent_primary: egui::Color32,
    pub accent_secondary: egui::Color32,
    pub accent_glow: egui::Color32,

    // Secondary accents
    pub blue: egui::Color32,
    pub cyan: egui::Color32,
    pub purple: egui::Color32,
    pub magenta: egui::Color32,

    // Legacy color names (used throughout GUI code)
    pub deep_blue: egui::Color32,
    pub medium_blue: egui::Color32,
    pub light_blue: egui::Color32,
    pub green_primary: egui::Color32,
    pub green_secondary: egui::Color32,
    pub green_glow: egui::Color32,
    pub background: egui::Color32,

    // Text colors
    pub text_primary: egui::Color32,
    pub text_secondary: egui::Color32,
    pub text_muted: egui::Color32,
    pub fg: egui::Color32,
    pub fg_dark: egui::Color32,
    pub comment: egui::Color32,

    // Status colors
    pub success: egui::Color32,
    pub warning: egui::Color32,
    pub error: egui::Color32,
    pub info: egui::Color32,

    // VU Meter colors
    pub vu_green: egui::Color32,
    pub vu_yellow: egui::Color32,
    pub vu_red: egui::Color32,
}

impl WavelinkTheme {
    /// Create theme with specified preset
    pub fn with_preset(preset: ThemePreset) -> Self {
        match preset {
            ThemePreset::TokyoNight => Self::tokyo_night(),
            ThemePreset::TokyoNightStorm => Self::tokyo_night_storm(),
            ThemePreset::TokyoNightMoon => Self::tokyo_night_moon(),
        }
    }

    /// Tokyo Night (Night variant) - Default theme
    /// Colors from tokyonight.nvim night palette
    pub fn tokyo_night() -> Self {
        Self {
            preset: ThemePreset::TokyoNight,

            // Backgrounds - Deep dark blue
            bg: egui::Color32::from_rgb(0x1a, 0x1b, 0x26), // #1a1b26
            bg_dark: egui::Color32::from_rgb(0x16, 0x16, 0x1e), // #16161e
            bg_highlight: egui::Color32::from_rgb(0x29, 0x2e, 0x42), // #292e42
            panel_bg: egui::Color32::from_rgb(0x1f, 0x23, 0x35), // #1f2335
            card_bg: egui::Color32::from_rgb(0x24, 0x28, 0x3b), // #24283b
            input_bg: egui::Color32::from_rgb(0x29, 0x2e, 0x42), // #292e42

            // Primary accent - Blue
            accent_primary: egui::Color32::from_rgb(0x7a, 0xa2, 0xf7), // #7aa2f7
            accent_secondary: egui::Color32::from_rgb(0x7d, 0xcf, 0xff), // #7dcfff
            accent_glow: egui::Color32::from_rgb(0x2a, 0xc3, 0xde),    // #2ac3de

            // Secondary accents
            blue: egui::Color32::from_rgb(0x7a, 0xa2, 0xf7), // #7aa2f7
            cyan: egui::Color32::from_rgb(0x7d, 0xcf, 0xff), // #7dcfff
            purple: egui::Color32::from_rgb(0xbb, 0x9a, 0xf7), // #bb9af7
            magenta: egui::Color32::from_rgb(0xff, 0x00, 0x7c), // #ff007c

            // Legacy
            deep_blue: egui::Color32::from_rgb(0x16, 0x16, 0x1e),
            medium_blue: egui::Color32::from_rgb(0x1f, 0x23, 0x35),
            light_blue: egui::Color32::from_rgb(0x3b, 0x42, 0x61),
            green_primary: egui::Color32::from_rgb(0x9e, 0xce, 0x6a), // #9ece6a
            green_secondary: egui::Color32::from_rgb(0x73, 0xda, 0xca), // #73daca
            green_glow: egui::Color32::from_rgb(0x41, 0xa6, 0xb5),    // #41a6b5
            background: egui::Color32::from_rgb(0x1a, 0x1b, 0x26),

            // Text
            text_primary: egui::Color32::from_rgb(0xc0, 0xca, 0xf5), // #c0caf5
            text_secondary: egui::Color32::from_rgb(0xa9, 0xb1, 0xd6), // #a9b1d6
            text_muted: egui::Color32::from_rgb(0x56, 0x5f, 0x89),   // #565f89
            fg: egui::Color32::from_rgb(0xc0, 0xca, 0xf5),
            fg_dark: egui::Color32::from_rgb(0xa9, 0xb1, 0xd6),
            comment: egui::Color32::from_rgb(0x56, 0x5f, 0x89),

            // Status
            success: egui::Color32::from_rgb(0x9e, 0xce, 0x6a), // #9ece6a
            warning: egui::Color32::from_rgb(0xe0, 0xaf, 0x68), // #e0af68
            error: egui::Color32::from_rgb(0xf7, 0x76, 0x8e),   // #f7768e
            info: egui::Color32::from_rgb(0x7d, 0xcf, 0xff),    // #7dcfff

            // VU Meter
            vu_green: egui::Color32::from_rgb(0x9e, 0xce, 0x6a),
            vu_yellow: egui::Color32::from_rgb(0xe0, 0xaf, 0x68),
            vu_red: egui::Color32::from_rgb(0xf7, 0x76, 0x8e),
        }
    }

    /// Tokyo Night Storm - Lighter blue-shifted variant
    /// Uses #24283b as base instead of #1a1b26
    pub fn tokyo_night_storm() -> Self {
        let mut theme = Self::tokyo_night();
        theme.preset = ThemePreset::TokyoNightStorm;
        theme.bg = egui::Color32::from_rgb(0x24, 0x28, 0x3b); // #24283b
        theme.bg_dark = egui::Color32::from_rgb(0x1f, 0x23, 0x35); // #1f2335
        theme.bg_highlight = egui::Color32::from_rgb(0x2f, 0x33, 0x4d); // #2f334d
        theme.panel_bg = egui::Color32::from_rgb(0x24, 0x28, 0x3b);
        theme.card_bg = egui::Color32::from_rgb(0x29, 0x2e, 0x42);
        theme.input_bg = egui::Color32::from_rgb(0x2f, 0x33, 0x4d);
        theme.background = theme.bg;
        theme.deep_blue = theme.bg_dark;
        theme.medium_blue = theme.panel_bg;
        theme
    }

    /// Tokyo Night Moon - Warmer variant with purple/rose tints
    /// Colors from tokyonight.nvim moon palette
    pub fn tokyo_night_moon() -> Self {
        Self {
            preset: ThemePreset::TokyoNightMoon,

            // Backgrounds - Warmer with purple undertone
            bg: egui::Color32::from_rgb(0x22, 0x24, 0x36), // #222436
            bg_dark: egui::Color32::from_rgb(0x1e, 0x20, 0x30), // #1e2030
            bg_highlight: egui::Color32::from_rgb(0x2f, 0x33, 0x4d), // #2f334d
            panel_bg: egui::Color32::from_rgb(0x1e, 0x20, 0x30),
            card_bg: egui::Color32::from_rgb(0x2f, 0x33, 0x4d),
            input_bg: egui::Color32::from_rgb(0x2f, 0x33, 0x4d),

            // Primary accent - Blue with warmer tint
            accent_primary: egui::Color32::from_rgb(0x82, 0xaa, 0xff), // #82aaff
            accent_secondary: egui::Color32::from_rgb(0x86, 0xe1, 0xfc), // #86e1fc
            accent_glow: egui::Color32::from_rgb(0x65, 0xbc, 0xff),    // #65bcff

            // Secondary accents
            blue: egui::Color32::from_rgb(0x82, 0xaa, 0xff), // #82aaff
            cyan: egui::Color32::from_rgb(0x86, 0xe1, 0xfc), // #86e1fc
            purple: egui::Color32::from_rgb(0xc0, 0x99, 0xff), // #c099ff
            magenta: egui::Color32::from_rgb(0xff, 0x75, 0x7f), // #ff757f

            // Legacy
            deep_blue: egui::Color32::from_rgb(0x1e, 0x20, 0x30),
            medium_blue: egui::Color32::from_rgb(0x22, 0x24, 0x36),
            light_blue: egui::Color32::from_rgb(0x3b, 0x3f, 0x5c),
            green_primary: egui::Color32::from_rgb(0xc3, 0xe8, 0x8d), // #c3e88d
            green_secondary: egui::Color32::from_rgb(0x4f, 0xd6, 0xbe), // #4fd6be
            green_glow: egui::Color32::from_rgb(0x41, 0xa6, 0xb5),
            background: egui::Color32::from_rgb(0x22, 0x24, 0x36),

            // Text
            text_primary: egui::Color32::from_rgb(0xc8, 0xd3, 0xf5), // #c8d3f5
            text_secondary: egui::Color32::from_rgb(0xa9, 0xb8, 0xe8), // #a9b8e8
            text_muted: egui::Color32::from_rgb(0x63, 0x6d, 0xa6),   // #636da6
            fg: egui::Color32::from_rgb(0xc8, 0xd3, 0xf5),
            fg_dark: egui::Color32::from_rgb(0xa9, 0xb8, 0xe8),
            comment: egui::Color32::from_rgb(0x63, 0x6d, 0xa6),

            // Status
            success: egui::Color32::from_rgb(0xc3, 0xe8, 0x8d), // #c3e88d
            warning: egui::Color32::from_rgb(0xff, 0xc7, 0x77), // #ffc777
            error: egui::Color32::from_rgb(0xff, 0x75, 0x7f),   // #ff757f
            info: egui::Color32::from_rgb(0x86, 0xe1, 0xfc),    // #86e1fc

            // VU Meter
            vu_green: egui::Color32::from_rgb(0xc3, 0xe8, 0x8d),
            vu_yellow: egui::Color32::from_rgb(0xff, 0xc7, 0x77),
            vu_red: egui::Color32::from_rgb(0xff, 0x75, 0x7f),
        }
    }

    /// Create default theme (Tokyo Night)
    pub fn new() -> Self {
        Self::tokyo_night()
    }

    /// Switch to a different preset
    pub fn set_preset(&mut self, preset: ThemePreset) {
        *self = Self::with_preset(preset);
    }

    /// Apply theme to egui context
    pub fn apply(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        style.visuals = egui::Visuals::dark();

        // Main backgrounds
        style.visuals.window_fill = self.bg;
        style.visuals.panel_fill = self.panel_bg;
        style.visuals.faint_bg_color = self.card_bg;

        // Window styling
        style.visuals.window_stroke = egui::Stroke::new(2.0, self.bg_highlight);
        style.visuals.window_rounding = egui::Rounding::same(12.0);

        // Widget styling - Noninteractive
        style.visuals.widgets.noninteractive.bg_fill = self.input_bg;
        style.visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0, self.text_secondary);
        style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);

        // Widget styling - Inactive
        style.visuals.widgets.inactive.bg_fill = self.card_bg;
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, self.text_primary);
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, self.bg_highlight);
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);

        // Widget styling - Hovered
        style.visuals.widgets.hovered.bg_fill = self.bg_highlight;
        style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, self.accent_secondary);
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(2.0, self.accent_primary);
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);

        // Widget styling - Active/Pressed
        style.visuals.widgets.active.bg_fill = self.accent_primary;
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, self.bg_dark);
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, self.accent_glow);
        style.visuals.widgets.active.rounding = egui::Rounding::same(8.0);

        // Open widgets
        style.visuals.widgets.open.bg_fill = self.panel_bg;
        style.visuals.widgets.open.fg_stroke = egui::Stroke::new(1.5, self.accent_secondary);
        style.visuals.widgets.open.bg_stroke = egui::Stroke::new(2.0, self.accent_primary);
        style.visuals.widgets.open.rounding = egui::Rounding::same(8.0);

        // Text colors
        style.visuals.override_text_color = Some(self.text_primary);
        style.visuals.hyperlink_color = self.accent_secondary;

        // Selection
        let accent_with_alpha = egui::Color32::from_rgba_premultiplied(
            self.accent_primary.r(),
            self.accent_primary.g(),
            self.accent_primary.b(),
            60,
        );
        style.visuals.selection.bg_fill = accent_with_alpha;
        style.visuals.selection.stroke = egui::Stroke::new(1.5, self.accent_primary);

        // Status colors
        style.visuals.error_fg_color = self.error;
        style.visuals.warn_fg_color = self.warning;

        // Menu/popup styling
        style.visuals.menu_rounding = egui::Rounding::same(10.0);
        style.visuals.popup_shadow = egui::epaint::Shadow {
            offset: egui::vec2(4.0, 8.0),
            blur: 16.0,
            spread: 2.0,
            color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 100),
        };

        // Spacing
        style.spacing.button_padding = egui::vec2(20.0, 14.0);
        style.spacing.item_spacing = egui::vec2(16.0, 12.0);
        style.spacing.indent = 28.0;
        style.spacing.window_margin = egui::Margin::same(20.0);
        style.spacing.menu_margin = egui::Margin::same(12.0);
        style.spacing.slider_width = 24.0;

        ctx.set_style(style);
    }

    // Helper methods for translucent colors

    pub fn channel_strip_bg(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(
            self.card_bg.r(),
            self.card_bg.g(),
            self.card_bg.b(),
            200,
        )
    }

    pub fn channel_strip_border(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(
            self.bg_highlight.r(),
            self.bg_highlight.g(),
            self.bg_highlight.b(),
            180,
        )
    }

    pub fn translucent_panel_bg(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(
            self.panel_bg.r(),
            self.panel_bg.g(),
            self.panel_bg.b(),
            220,
        )
    }

    pub fn translucent_input_bg(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(
            self.input_bg.r(),
            self.input_bg.g(),
            self.input_bg.b(),
            200,
        )
    }

    pub fn glass_button_bg(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(
            self.accent_primary.r(),
            self.accent_primary.g(),
            self.accent_primary.b(),
            40,
        )
    }

    pub fn glass_button_hover(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(
            self.accent_primary.r(),
            self.accent_primary.g(),
            self.accent_primary.b(),
            80,
        )
    }

    pub fn glass_button_active(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(
            self.accent_primary.r(),
            self.accent_primary.g(),
            self.accent_primary.b(),
            120,
        )
    }

    pub fn status_active(&self) -> egui::Color32 {
        self.success
    }

    pub fn status_inactive(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(
            self.text_muted.r(),
            self.text_muted.g(),
            self.text_muted.b(),
            180,
        )
    }

    pub fn status_warning(&self) -> egui::Color32 {
        self.warning
    }

    pub fn status_error(&self) -> egui::Color32 {
        self.error
    }

    pub fn translucent_deep_bg(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(
            self.bg_dark.r(),
            self.bg_dark.g(),
            self.bg_dark.b(),
            220,
        )
    }

    pub fn vu_meter_bg(&self) -> egui::Color32 {
        self.bg_dark
    }

    pub fn vu_meter_green(&self) -> egui::Color32 {
        self.vu_green
    }

    pub fn vu_meter_yellow(&self) -> egui::Color32 {
        self.vu_yellow
    }

    pub fn vu_meter_red(&self) -> egui::Color32 {
        self.vu_red
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

impl Default for WavelinkTheme {
    fn default() -> Self {
        Self::new()
    }
}
