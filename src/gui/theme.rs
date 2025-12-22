//! # PhantomLink Theme System
//!
//! Comprehensive theming with multiple color schemes:
//! - **Tokyo Night** (Default) - Deep blues with cyan/purple accents
//! - **Catppuccin Mocha** - Warm dark theme with pastel accents
//! - **Dracula** - Classic dark purple theme
//! - **Wavelink** - Original green/blue professional theme

#![allow(dead_code)] // Complete theming API - colors/methods used as needed per theme preset

use eframe::egui;
use serde::{Deserialize, Serialize};

/// Available theme presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemePreset {
    /// Tokyo Night - Deep blue with cyan/purple accents (default)
    #[default]
    TokyoNight,
    /// Tokyo Night Storm - Lighter variant
    TokyoNightStorm,
    /// Tokyo Night Moon - Warmer variant with purple tints
    TokyoNightMoon,
    /// Catppuccin Mocha - Warm dark with pastel accents
    CatppuccinMocha,
    /// Catppuccin Macchiato - Medium dark variant
    CatppuccinMacchiato,
    /// Catppuccin Frappe - Lighter warm variant
    CatppuccinFrappe,
    /// Dracula - Classic purple dark theme
    Dracula,
    /// Nord - Arctic, north-bluish color palette
    Nord,
    /// Scarlett - Focusrite-inspired red/crimson theme
    Scarlett,
    /// Scarlett Solo - Focusrite Scarlett Solo 4th Gen hardware theme
    ScarlettSolo,
    /// Wavelink - Original professional green theme
    Wavelink,
}

impl ThemePreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::TokyoNight => "Tokyo Night",
            Self::TokyoNightStorm => "Tokyo Night Storm",
            Self::TokyoNightMoon => "Tokyo Night Moon",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::CatppuccinMacchiato => "Catppuccin Macchiato",
            Self::CatppuccinFrappe => "Catppuccin Frappe",
            Self::Dracula => "Dracula",
            Self::Nord => "Nord",
            Self::Scarlett => "Scarlett",
            Self::ScarlettSolo => "Scarlett Solo",
            Self::Wavelink => "Wavelink Pro",
        }
    }

    pub fn all() -> &'static [ThemePreset] {
        &[
            Self::TokyoNight,
            Self::TokyoNightStorm,
            Self::TokyoNightMoon,
            Self::CatppuccinMocha,
            Self::CatppuccinMacchiato,
            Self::CatppuccinFrappe,
            Self::Dracula,
            Self::Nord,
            Self::Scarlett,
            Self::ScarlettSolo,
            Self::Wavelink,
        ]
    }
}

/// Main theme structure for PhantomLink
pub struct WavelinkTheme {
    /// Current preset
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

    // Legacy color names (for compatibility)
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
            ThemePreset::CatppuccinMocha => Self::catppuccin_mocha(),
            ThemePreset::CatppuccinMacchiato => Self::catppuccin_macchiato(),
            ThemePreset::CatppuccinFrappe => Self::catppuccin_frappe(),
            ThemePreset::Dracula => Self::dracula(),
            ThemePreset::Nord => Self::nord(),
            ThemePreset::Scarlett => Self::scarlett(),
            ThemePreset::ScarlettSolo => Self::scarlett_solo(),
            ThemePreset::Wavelink => Self::wavelink(),
        }
    }

    /// Tokyo Night (Night variant) - Default theme
    pub fn tokyo_night() -> Self {
        // From tokyonight.nvim/lua/tokyonight/colors/night.lua
        Self {
            preset: ThemePreset::TokyoNight,

            // Backgrounds - Deep dark blue
            bg: egui::Color32::from_rgb(0x1a, 0x1b, 0x26),           // #1a1b26
            bg_dark: egui::Color32::from_rgb(0x16, 0x16, 0x1e),      // #16161e
            bg_highlight: egui::Color32::from_rgb(0x29, 0x2e, 0x42), // #292e42
            panel_bg: egui::Color32::from_rgb(0x1f, 0x23, 0x35),     // #1f2335
            card_bg: egui::Color32::from_rgb(0x24, 0x28, 0x3b),      // #24283b
            input_bg: egui::Color32::from_rgb(0x29, 0x2e, 0x42),     // #292e42

            // Primary accent - Cyan/Blue
            accent_primary: egui::Color32::from_rgb(0x7a, 0xa2, 0xf7),   // #7aa2f7 blue
            accent_secondary: egui::Color32::from_rgb(0x7d, 0xcf, 0xff), // #7dcfff cyan
            accent_glow: egui::Color32::from_rgb(0x2a, 0xc3, 0xde),      // #2ac3de blue1

            // Secondary accents
            blue: egui::Color32::from_rgb(0x7a, 0xa2, 0xf7),     // #7aa2f7
            cyan: egui::Color32::from_rgb(0x7d, 0xcf, 0xff),     // #7dcfff
            purple: egui::Color32::from_rgb(0xbb, 0x9a, 0xf7),   // #bb9af7 magenta
            magenta: egui::Color32::from_rgb(0xff, 0x00, 0x7c),  // #ff007c magenta2

            // Legacy compatibility
            deep_blue: egui::Color32::from_rgb(0x16, 0x16, 0x1e),
            medium_blue: egui::Color32::from_rgb(0x1f, 0x23, 0x35),
            light_blue: egui::Color32::from_rgb(0x3b, 0x42, 0x61),
            green_primary: egui::Color32::from_rgb(0x9e, 0xce, 0x6a),   // #9ece6a
            green_secondary: egui::Color32::from_rgb(0x73, 0xda, 0xca), // #73daca green1
            green_glow: egui::Color32::from_rgb(0x41, 0xa6, 0xb5),      // #41a6b5 green2
            background: egui::Color32::from_rgb(0x1a, 0x1b, 0x26),

            // Text
            text_primary: egui::Color32::from_rgb(0xc0, 0xca, 0xf5),   // #c0caf5 fg
            text_secondary: egui::Color32::from_rgb(0xa9, 0xb1, 0xd6), // #a9b1d6 fg_dark
            text_muted: egui::Color32::from_rgb(0x56, 0x5f, 0x89),     // #565f89 comment
            fg: egui::Color32::from_rgb(0xc0, 0xca, 0xf5),
            fg_dark: egui::Color32::from_rgb(0xa9, 0xb1, 0xd6),
            comment: egui::Color32::from_rgb(0x56, 0x5f, 0x89),

            // Status
            success: egui::Color32::from_rgb(0x9e, 0xce, 0x6a), // #9ece6a green
            warning: egui::Color32::from_rgb(0xe0, 0xaf, 0x68), // #e0af68 yellow
            error: egui::Color32::from_rgb(0xf7, 0x76, 0x8e),   // #f7768e red
            info: egui::Color32::from_rgb(0x7d, 0xcf, 0xff),    // #7dcfff cyan

            // VU Meter
            vu_green: egui::Color32::from_rgb(0x9e, 0xce, 0x6a),
            vu_yellow: egui::Color32::from_rgb(0xe0, 0xaf, 0x68),
            vu_red: egui::Color32::from_rgb(0xf7, 0x76, 0x8e),
        }
    }

    /// Tokyo Night Storm - Slightly lighter variant
    pub fn tokyo_night_storm() -> Self {
        let mut theme = Self::tokyo_night();
        theme.preset = ThemePreset::TokyoNightStorm;
        // Storm uses lighter backgrounds
        theme.bg = egui::Color32::from_rgb(0x24, 0x28, 0x3b);           // #24283b
        theme.bg_dark = egui::Color32::from_rgb(0x1f, 0x23, 0x35);      // #1f2335
        theme.panel_bg = egui::Color32::from_rgb(0x24, 0x28, 0x3b);
        theme.card_bg = egui::Color32::from_rgb(0x29, 0x2e, 0x42);
        theme.background = theme.bg;
        theme.deep_blue = theme.bg_dark;
        theme.medium_blue = theme.panel_bg;
        theme
    }

    /// Tokyo Night Moon - Warmer variant with purple/rose tints
    pub fn tokyo_night_moon() -> Self {
        Self {
            preset: ThemePreset::TokyoNightMoon,

            // Backgrounds - Warmer with slight purple undertone
            bg: egui::Color32::from_rgb(0x22, 0x24, 0x36),           // #222436
            bg_dark: egui::Color32::from_rgb(0x1e, 0x20, 0x30),      // #1e2030
            bg_highlight: egui::Color32::from_rgb(0x2f, 0x33, 0x4d), // #2f334d
            panel_bg: egui::Color32::from_rgb(0x1e, 0x20, 0x30),
            card_bg: egui::Color32::from_rgb(0x2f, 0x33, 0x4d),
            input_bg: egui::Color32::from_rgb(0x2f, 0x33, 0x4d),

            // Primary accent - Blue with warmer tint
            accent_primary: egui::Color32::from_rgb(0x82, 0xaa, 0xff),   // #82aaff blue
            accent_secondary: egui::Color32::from_rgb(0x86, 0xe1, 0xfc), // #86e1fc cyan
            accent_glow: egui::Color32::from_rgb(0x65, 0xbc, 0xff),      // #65bcff

            // Secondary accents
            blue: egui::Color32::from_rgb(0x82, 0xaa, 0xff),     // #82aaff
            cyan: egui::Color32::from_rgb(0x86, 0xe1, 0xfc),     // #86e1fc
            purple: egui::Color32::from_rgb(0xc0, 0x99, 0xff),   // #c099ff
            magenta: egui::Color32::from_rgb(0xff, 0x75, 0x7f),  // #ff757f

            // Legacy
            deep_blue: egui::Color32::from_rgb(0x1e, 0x20, 0x30),
            medium_blue: egui::Color32::from_rgb(0x22, 0x24, 0x36),
            light_blue: egui::Color32::from_rgb(0x3b, 0x3f, 0x5c),
            green_primary: egui::Color32::from_rgb(0xc3, 0xe8, 0x8d),   // #c3e88d
            green_secondary: egui::Color32::from_rgb(0x4f, 0xd6, 0xbe), // #4fd6be teal
            green_glow: egui::Color32::from_rgb(0x41, 0xa6, 0xb5),
            background: egui::Color32::from_rgb(0x22, 0x24, 0x36),

            // Text
            text_primary: egui::Color32::from_rgb(0xc8, 0xd3, 0xf5),   // #c8d3f5
            text_secondary: egui::Color32::from_rgb(0xa9, 0xb8, 0xe8), // #a9b8e8
            text_muted: egui::Color32::from_rgb(0x63, 0x6d, 0xa6),     // #636da6
            fg: egui::Color32::from_rgb(0xc8, 0xd3, 0xf5),
            fg_dark: egui::Color32::from_rgb(0xa9, 0xb8, 0xe8),
            comment: egui::Color32::from_rgb(0x63, 0x6d, 0xa6),

            // Status
            success: egui::Color32::from_rgb(0xc3, 0xe8, 0x8d), // #c3e88d green
            warning: egui::Color32::from_rgb(0xff, 0xc7, 0x77), // #ffc777 yellow
            error: egui::Color32::from_rgb(0xff, 0x75, 0x7f),   // #ff757f red
            info: egui::Color32::from_rgb(0x86, 0xe1, 0xfc),    // #86e1fc cyan

            // VU Meter
            vu_green: egui::Color32::from_rgb(0xc3, 0xe8, 0x8d),
            vu_yellow: egui::Color32::from_rgb(0xff, 0xc7, 0x77),
            vu_red: egui::Color32::from_rgb(0xff, 0x75, 0x7f),
        }
    }

    /// Catppuccin Mocha - Warm dark theme
    pub fn catppuccin_mocha() -> Self {
        // From catppuccin palette - Mocha variant
        Self {
            preset: ThemePreset::CatppuccinMocha,

            // Backgrounds
            bg: egui::Color32::from_rgb(0x1e, 0x1e, 0x2e),           // Base #1e1e2e
            bg_dark: egui::Color32::from_rgb(0x11, 0x11, 0x1b),      // Crust #11111b
            bg_highlight: egui::Color32::from_rgb(0x31, 0x32, 0x44), // Surface1 #313244
            panel_bg: egui::Color32::from_rgb(0x18, 0x18, 0x25),     // Mantle #181825
            card_bg: egui::Color32::from_rgb(0x31, 0x32, 0x44),      // Surface1
            input_bg: egui::Color32::from_rgb(0x45, 0x47, 0x5a),     // Surface2 #45475a

            // Primary accent - Mauve/Lavender
            accent_primary: egui::Color32::from_rgb(0xcb, 0xa6, 0xf7),   // Mauve #cba6f7
            accent_secondary: egui::Color32::from_rgb(0xb4, 0xbe, 0xfe), // Lavender #b4befe
            accent_glow: egui::Color32::from_rgb(0xf5, 0xc2, 0xe7),      // Pink #f5c2e7

            // Secondary accents
            blue: egui::Color32::from_rgb(0x89, 0xb4, 0xfa),     // Blue #89b4fa
            cyan: egui::Color32::from_rgb(0x94, 0xe2, 0xd5),     // Teal #94e2d5
            purple: egui::Color32::from_rgb(0xcb, 0xa6, 0xf7),   // Mauve
            magenta: egui::Color32::from_rgb(0xf5, 0xc2, 0xe7),  // Pink

            // Legacy
            deep_blue: egui::Color32::from_rgb(0x11, 0x11, 0x1b),
            medium_blue: egui::Color32::from_rgb(0x18, 0x18, 0x25),
            light_blue: egui::Color32::from_rgb(0x31, 0x32, 0x44),
            green_primary: egui::Color32::from_rgb(0xa6, 0xe3, 0xa1),   // Green #a6e3a1
            green_secondary: egui::Color32::from_rgb(0x94, 0xe2, 0xd5), // Teal
            green_glow: egui::Color32::from_rgb(0x74, 0xc7, 0xec),      // Sapphire #74c7ec
            background: egui::Color32::from_rgb(0x1e, 0x1e, 0x2e),

            // Text
            text_primary: egui::Color32::from_rgb(0xcd, 0xd6, 0xf4),   // Text #cdd6f4
            text_secondary: egui::Color32::from_rgb(0xba, 0xc2, 0xde), // Subtext1 #bac2de
            text_muted: egui::Color32::from_rgb(0x6c, 0x70, 0x86),     // Overlay0 #6c7086
            fg: egui::Color32::from_rgb(0xcd, 0xd6, 0xf4),
            fg_dark: egui::Color32::from_rgb(0xba, 0xc2, 0xde),
            comment: egui::Color32::from_rgb(0x6c, 0x70, 0x86),

            // Status
            success: egui::Color32::from_rgb(0xa6, 0xe3, 0xa1), // Green
            warning: egui::Color32::from_rgb(0xf9, 0xe2, 0xaf), // Yellow #f9e2af
            error: egui::Color32::from_rgb(0xf3, 0x8b, 0xa8),   // Red #f38ba8
            info: egui::Color32::from_rgb(0x89, 0xb4, 0xfa),    // Blue

            // VU Meter
            vu_green: egui::Color32::from_rgb(0xa6, 0xe3, 0xa1),
            vu_yellow: egui::Color32::from_rgb(0xf9, 0xe2, 0xaf),
            vu_red: egui::Color32::from_rgb(0xf3, 0x8b, 0xa8),
        }
    }

    /// Catppuccin Macchiato - Medium dark variant
    pub fn catppuccin_macchiato() -> Self {
        let mut theme = Self::catppuccin_mocha();
        theme.preset = ThemePreset::CatppuccinMacchiato;
        // Macchiato uses slightly lighter bases
        theme.bg = egui::Color32::from_rgb(0x24, 0x27, 0x3a);           // Base #24273a
        theme.bg_dark = egui::Color32::from_rgb(0x18, 0x19, 0x26);      // Crust #181926
        theme.panel_bg = egui::Color32::from_rgb(0x1e, 0x20, 0x30);     // Mantle #1e2030
        theme.card_bg = egui::Color32::from_rgb(0x36, 0x3a, 0x4f);      // Surface1 #363a4f
        theme.input_bg = egui::Color32::from_rgb(0x49, 0x4d, 0x64);     // Surface2 #494d64
        theme.background = theme.bg;
        theme.deep_blue = theme.bg_dark;
        theme.medium_blue = theme.panel_bg;
        theme.light_blue = theme.card_bg;
        theme
    }

    /// Catppuccin Frappe - Lighter warm variant
    pub fn catppuccin_frappe() -> Self {
        Self {
            preset: ThemePreset::CatppuccinFrappe,

            // Backgrounds - Lighter than Mocha
            bg: egui::Color32::from_rgb(0x30, 0x34, 0x46),           // Base #303446
            bg_dark: egui::Color32::from_rgb(0x23, 0x26, 0x34),      // Crust #232634
            bg_highlight: egui::Color32::from_rgb(0x41, 0x45, 0x59), // Surface1 #414559
            panel_bg: egui::Color32::from_rgb(0x29, 0x2c, 0x3c),     // Mantle #292c3c
            card_bg: egui::Color32::from_rgb(0x41, 0x45, 0x59),
            input_bg: egui::Color32::from_rgb(0x51, 0x57, 0x6d),     // Surface2 #51576d

            // Primary accent - Mauve/Lavender
            accent_primary: egui::Color32::from_rgb(0xca, 0x9e, 0xe6),   // Mauve #ca9ee6
            accent_secondary: egui::Color32::from_rgb(0xba, 0xbb, 0xf1), // Lavender #babbf1
            accent_glow: egui::Color32::from_rgb(0xf4, 0xb8, 0xe4),      // Pink #f4b8e4

            // Secondary accents
            blue: egui::Color32::from_rgb(0x8c, 0xaa, 0xee),     // Blue #8caaee
            cyan: egui::Color32::from_rgb(0x81, 0xc8, 0xbe),     // Teal #81c8be
            purple: egui::Color32::from_rgb(0xca, 0x9e, 0xe6),   // Mauve
            magenta: egui::Color32::from_rgb(0xf4, 0xb8, 0xe4),  // Pink

            // Legacy
            deep_blue: egui::Color32::from_rgb(0x23, 0x26, 0x34),
            medium_blue: egui::Color32::from_rgb(0x29, 0x2c, 0x3c),
            light_blue: egui::Color32::from_rgb(0x41, 0x45, 0x59),
            green_primary: egui::Color32::from_rgb(0xa6, 0xd1, 0x89),   // Green #a6d189
            green_secondary: egui::Color32::from_rgb(0x81, 0xc8, 0xbe), // Teal
            green_glow: egui::Color32::from_rgb(0x85, 0xc1, 0xdc),      // Sapphire #85c1dc
            background: egui::Color32::from_rgb(0x30, 0x34, 0x46),

            // Text
            text_primary: egui::Color32::from_rgb(0xc6, 0xd0, 0xf5),   // Text #c6d0f5
            text_secondary: egui::Color32::from_rgb(0xb5, 0xbf, 0xe2), // Subtext1 #b5bfe2
            text_muted: egui::Color32::from_rgb(0x73, 0x78, 0x94),     // Overlay0 #737894
            fg: egui::Color32::from_rgb(0xc6, 0xd0, 0xf5),
            fg_dark: egui::Color32::from_rgb(0xb5, 0xbf, 0xe2),
            comment: egui::Color32::from_rgb(0x73, 0x78, 0x94),

            // Status
            success: egui::Color32::from_rgb(0xa6, 0xd1, 0x89), // Green
            warning: egui::Color32::from_rgb(0xe5, 0xc8, 0x90), // Yellow #e5c890
            error: egui::Color32::from_rgb(0xe7, 0x82, 0x84),   // Red #e78284
            info: egui::Color32::from_rgb(0x8c, 0xaa, 0xee),    // Blue

            // VU Meter
            vu_green: egui::Color32::from_rgb(0xa6, 0xd1, 0x89),
            vu_yellow: egui::Color32::from_rgb(0xe5, 0xc8, 0x90),
            vu_red: egui::Color32::from_rgb(0xe7, 0x82, 0x84),
        }
    }

    /// Dracula theme
    pub fn dracula() -> Self {
        Self {
            preset: ThemePreset::Dracula,

            // Backgrounds
            bg: egui::Color32::from_rgb(0x28, 0x2a, 0x36),           // Background #282a36
            bg_dark: egui::Color32::from_rgb(0x21, 0x22, 0x2c),      // Darker
            bg_highlight: egui::Color32::from_rgb(0x44, 0x47, 0x5a), // Current Line #44475a
            panel_bg: egui::Color32::from_rgb(0x28, 0x2a, 0x36),
            card_bg: egui::Color32::from_rgb(0x44, 0x47, 0x5a),
            input_bg: egui::Color32::from_rgb(0x44, 0x47, 0x5a),

            // Primary accent - Purple
            accent_primary: egui::Color32::from_rgb(0xbd, 0x93, 0xf9),   // Purple #bd93f9
            accent_secondary: egui::Color32::from_rgb(0xff, 0x79, 0xc6), // Pink #ff79c6
            accent_glow: egui::Color32::from_rgb(0x8b, 0xe9, 0xfd),      // Cyan #8be9fd

            // Secondary accents
            blue: egui::Color32::from_rgb(0x8b, 0xe9, 0xfd),     // Cyan
            cyan: egui::Color32::from_rgb(0x8b, 0xe9, 0xfd),     // Cyan
            purple: egui::Color32::from_rgb(0xbd, 0x93, 0xf9),   // Purple
            magenta: egui::Color32::from_rgb(0xff, 0x79, 0xc6),  // Pink

            // Legacy
            deep_blue: egui::Color32::from_rgb(0x21, 0x22, 0x2c),
            medium_blue: egui::Color32::from_rgb(0x28, 0x2a, 0x36),
            light_blue: egui::Color32::from_rgb(0x44, 0x47, 0x5a),
            green_primary: egui::Color32::from_rgb(0x50, 0xfa, 0x7b),   // Green #50fa7b
            green_secondary: egui::Color32::from_rgb(0x8b, 0xe9, 0xfd), // Cyan
            green_glow: egui::Color32::from_rgb(0x50, 0xfa, 0x7b),
            background: egui::Color32::from_rgb(0x28, 0x2a, 0x36),

            // Text
            text_primary: egui::Color32::from_rgb(0xf8, 0xf8, 0xf2),   // Foreground #f8f8f2
            text_secondary: egui::Color32::from_rgb(0xbd, 0xbf, 0xbf), // Slightly muted
            text_muted: egui::Color32::from_rgb(0x62, 0x72, 0xa4),     // Comment #6272a4
            fg: egui::Color32::from_rgb(0xf8, 0xf8, 0xf2),
            fg_dark: egui::Color32::from_rgb(0xbd, 0xbf, 0xbf),
            comment: egui::Color32::from_rgb(0x62, 0x72, 0xa4),

            // Status
            success: egui::Color32::from_rgb(0x50, 0xfa, 0x7b), // Green
            warning: egui::Color32::from_rgb(0xf1, 0xfa, 0x8c), // Yellow #f1fa8c
            error: egui::Color32::from_rgb(0xff, 0x55, 0x55),   // Red #ff5555
            info: egui::Color32::from_rgb(0x8b, 0xe9, 0xfd),    // Cyan

            // VU Meter
            vu_green: egui::Color32::from_rgb(0x50, 0xfa, 0x7b),
            vu_yellow: egui::Color32::from_rgb(0xf1, 0xfa, 0x8c),
            vu_red: egui::Color32::from_rgb(0xff, 0x55, 0x55),
        }
    }

    /// Scarlett - Focusrite-inspired crimson/red theme
    /// Dark theme with red accents inspired by Focusrite Scarlett audio interfaces
    pub fn scarlett() -> Self {
        Self {
            preset: ThemePreset::Scarlett,

            // Backgrounds - Deep charcoal with warm undertone
            bg: egui::Color32::from_rgb(0x1a, 0x14, 0x16),           // Deep charcoal-red
            bg_dark: egui::Color32::from_rgb(0x12, 0x0e, 0x10),      // Darker
            bg_highlight: egui::Color32::from_rgb(0x2e, 0x22, 0x26), // Highlighted
            panel_bg: egui::Color32::from_rgb(0x1e, 0x18, 0x1a),     // Panel
            card_bg: egui::Color32::from_rgb(0x28, 0x1e, 0x22),      // Cards
            input_bg: egui::Color32::from_rgb(0x32, 0x26, 0x2a),     // Input fields

            // Primary accent - Focusrite Scarlett Red
            accent_primary: egui::Color32::from_rgb(0xdc, 0x14, 0x3c),   // Scarlett red #dc143c
            accent_secondary: egui::Color32::from_rgb(0xff, 0x45, 0x45), // Bright red
            accent_glow: egui::Color32::from_rgb(0xb0, 0x10, 0x30),      // Deep crimson glow

            // Secondary accents
            blue: egui::Color32::from_rgb(0x87, 0x9a, 0xb8),     // Cool steel blue
            cyan: egui::Color32::from_rgb(0x6b, 0x8e, 0x9e),     // Muted teal
            purple: egui::Color32::from_rgb(0x8b, 0x5c, 0x6b),   // Dusty rose
            magenta: egui::Color32::from_rgb(0xd6, 0x33, 0x6c),  // Hot pink-red

            // Legacy - Map to reds/charcoals
            deep_blue: egui::Color32::from_rgb(0x12, 0x0e, 0x10),
            medium_blue: egui::Color32::from_rgb(0x1e, 0x18, 0x1a),
            light_blue: egui::Color32::from_rgb(0x32, 0x26, 0x2a),
            green_primary: egui::Color32::from_rgb(0x4a, 0xb8, 0x6a),   // Muted green (for success)
            green_secondary: egui::Color32::from_rgb(0x6b, 0x8e, 0x9e), // Teal accent
            green_glow: egui::Color32::from_rgb(0x3d, 0x9a, 0x5a),
            background: egui::Color32::from_rgb(0x1a, 0x14, 0x16),

            // Text - Warm whites
            text_primary: egui::Color32::from_rgb(0xf5, 0xee, 0xef),   // Warm white
            text_secondary: egui::Color32::from_rgb(0xc8, 0xbe, 0xc0), // Warm gray
            text_muted: egui::Color32::from_rgb(0x7a, 0x6e, 0x72),     // Muted warm gray
            fg: egui::Color32::from_rgb(0xf5, 0xee, 0xef),
            fg_dark: egui::Color32::from_rgb(0xc8, 0xbe, 0xc0),
            comment: egui::Color32::from_rgb(0x7a, 0x6e, 0x72),

            // Status - Red theme-aware
            success: egui::Color32::from_rgb(0x4a, 0xb8, 0x6a), // Green (contrast with red)
            warning: egui::Color32::from_rgb(0xf0, 0xa0, 0x50), // Warm amber
            error: egui::Color32::from_rgb(0xff, 0x45, 0x45),   // Bright red
            info: egui::Color32::from_rgb(0x87, 0x9a, 0xb8),    // Steel blue

            // VU Meter - Classic pro audio colors
            vu_green: egui::Color32::from_rgb(0x4a, 0xb8, 0x6a),
            vu_yellow: egui::Color32::from_rgb(0xf0, 0xa0, 0x50),
            vu_red: egui::Color32::from_rgb(0xdc, 0x14, 0x3c),  // Scarlett red
        }
    }

    /// Nord - Arctic, north-bluish color palette
    /// Based on https://www.nordtheme.com/
    pub fn nord() -> Self {
        // Nord Polar Night (backgrounds)
        // nord0 #2e3440, nord1 #3b4252, nord2 #434c5e, nord3 #4c566a
        // Nord Snow Storm (text)
        // nord4 #d8dee9, nord5 #e5e9f0, nord6 #eceff4
        // Nord Frost (accents)
        // nord7 #8fbcbb, nord8 #88c0d0, nord9 #81a1c1, nord10 #5e81ac
        // Nord Aurora (status)
        // nord11 #bf616a (red), nord12 #d08770 (orange), nord13 #ebcb8b (yellow)
        // nord14 #a3be8c (green), nord15 #b48ead (purple)
        Self {
            preset: ThemePreset::Nord,

            // Backgrounds - Polar Night
            bg: egui::Color32::from_rgb(0x2e, 0x34, 0x40),           // nord0 #2e3440
            bg_dark: egui::Color32::from_rgb(0x24, 0x29, 0x33),      // Darker than nord0
            bg_highlight: egui::Color32::from_rgb(0x3b, 0x42, 0x52), // nord1 #3b4252
            panel_bg: egui::Color32::from_rgb(0x2e, 0x34, 0x40),     // nord0
            card_bg: egui::Color32::from_rgb(0x3b, 0x42, 0x52),      // nord1
            input_bg: egui::Color32::from_rgb(0x43, 0x4c, 0x5e),     // nord2 #434c5e

            // Primary accent - Frost (blue tones)
            accent_primary: egui::Color32::from_rgb(0x88, 0xc0, 0xd0),   // nord8 #88c0d0 (cyan-ish)
            accent_secondary: egui::Color32::from_rgb(0x81, 0xa1, 0xc1), // nord9 #81a1c1 (blue)
            accent_glow: egui::Color32::from_rgb(0x8f, 0xbc, 0xbb),      // nord7 #8fbcbb (teal)

            // Secondary accents
            blue: egui::Color32::from_rgb(0x5e, 0x81, 0xac),     // nord10 #5e81ac
            cyan: egui::Color32::from_rgb(0x88, 0xc0, 0xd0),     // nord8
            purple: egui::Color32::from_rgb(0xb4, 0x8e, 0xad),   // nord15 #b48ead
            magenta: egui::Color32::from_rgb(0xb4, 0x8e, 0xad),  // nord15

            // Legacy - Map to polar night
            deep_blue: egui::Color32::from_rgb(0x24, 0x29, 0x33),
            medium_blue: egui::Color32::from_rgb(0x2e, 0x34, 0x40),
            light_blue: egui::Color32::from_rgb(0x4c, 0x56, 0x6a),     // nord3
            green_primary: egui::Color32::from_rgb(0xa3, 0xbe, 0x8c),   // nord14 #a3be8c
            green_secondary: egui::Color32::from_rgb(0x8f, 0xbc, 0xbb), // nord7
            green_glow: egui::Color32::from_rgb(0x88, 0xc0, 0xd0),
            background: egui::Color32::from_rgb(0x2e, 0x34, 0x40),

            // Text - Snow Storm
            text_primary: egui::Color32::from_rgb(0xec, 0xef, 0xf4),   // nord6 #eceff4
            text_secondary: egui::Color32::from_rgb(0xd8, 0xde, 0xe9), // nord4 #d8dee9
            text_muted: egui::Color32::from_rgb(0x4c, 0x56, 0x6a),     // nord3 #4c566a
            fg: egui::Color32::from_rgb(0xec, 0xef, 0xf4),
            fg_dark: egui::Color32::from_rgb(0xd8, 0xde, 0xe9),
            comment: egui::Color32::from_rgb(0x4c, 0x56, 0x6a),

            // Status - Aurora
            success: egui::Color32::from_rgb(0xa3, 0xbe, 0x8c), // nord14 green
            warning: egui::Color32::from_rgb(0xeb, 0xcb, 0x8b), // nord13 yellow
            error: egui::Color32::from_rgb(0xbf, 0x61, 0x6a),   // nord11 red
            info: egui::Color32::from_rgb(0x88, 0xc0, 0xd0),    // nord8 cyan

            // VU Meter - Aurora colors
            vu_green: egui::Color32::from_rgb(0xa3, 0xbe, 0x8c),  // nord14
            vu_yellow: egui::Color32::from_rgb(0xeb, 0xcb, 0x8b), // nord13
            vu_red: egui::Color32::from_rgb(0xbf, 0x61, 0x6a),    // nord11
        }
    }

    /// Scarlett Solo - Focusrite Scarlett Solo 4th Gen hardware-accurate theme
    /// Inspired by the actual device: matte black enclosure, red ring gain knob,
    /// green/red LED indicators, silver/chrome accents
    pub fn scarlett_solo() -> Self {
        // Scarlett Solo 4th Gen color reference:
        // - Main body: Matte black anodized aluminum
        // - Gain ring: Bright red (Focusrite signature)
        // - LED indicators: Green (signal), Red (clip)
        // - Text/labels: White silk-screen
        // - Knob caps: Dark gray/gunmetal
        Self {
            preset: ThemePreset::ScarlettSolo,

            // Backgrounds - Matte black aluminum inspired
            bg: egui::Color32::from_rgb(0x18, 0x18, 0x18),           // Matte black
            bg_dark: egui::Color32::from_rgb(0x10, 0x10, 0x10),      // Deeper black
            bg_highlight: egui::Color32::from_rgb(0x2a, 0x2a, 0x2a), // Gunmetal gray
            panel_bg: egui::Color32::from_rgb(0x1c, 0x1c, 0x1c),     // Slightly lighter panel
            card_bg: egui::Color32::from_rgb(0x24, 0x24, 0x24),      // Card surface
            input_bg: egui::Color32::from_rgb(0x30, 0x30, 0x30),     // Input fields

            // Primary accent - Focusrite Red (the signature gain ring color)
            accent_primary: egui::Color32::from_rgb(0xcc, 0x00, 0x00),   // Focusrite red
            accent_secondary: egui::Color32::from_rgb(0xff, 0x33, 0x33), // Brighter red hover
            accent_glow: egui::Color32::from_rgb(0x99, 0x00, 0x00),      // Deep red glow

            // Secondary accents - Hardware inspired
            blue: egui::Color32::from_rgb(0x4a, 0x7c, 0xa7),     // Cool steel accent
            cyan: egui::Color32::from_rgb(0x5a, 0x8a, 0x9a),     // Muted teal
            purple: egui::Color32::from_rgb(0x6a, 0x5a, 0x7a),   // Subtle purple
            magenta: egui::Color32::from_rgb(0xb0, 0x30, 0x60),  // Deep magenta

            // Legacy
            deep_blue: egui::Color32::from_rgb(0x10, 0x10, 0x10),
            medium_blue: egui::Color32::from_rgb(0x1c, 0x1c, 0x1c),
            light_blue: egui::Color32::from_rgb(0x30, 0x30, 0x30),
            green_primary: egui::Color32::from_rgb(0x00, 0xcc, 0x44),   // Scarlett LED green
            green_secondary: egui::Color32::from_rgb(0x5a, 0x8a, 0x9a),
            green_glow: egui::Color32::from_rgb(0x00, 0x99, 0x33),
            background: egui::Color32::from_rgb(0x18, 0x18, 0x18),

            // Text - Silk-screen white on black
            text_primary: egui::Color32::from_rgb(0xf0, 0xf0, 0xf0),   // White labels
            text_secondary: egui::Color32::from_rgb(0xc0, 0xc0, 0xc0), // Light gray
            text_muted: egui::Color32::from_rgb(0x70, 0x70, 0x70),     // Muted gray
            fg: egui::Color32::from_rgb(0xf0, 0xf0, 0xf0),
            fg_dark: egui::Color32::from_rgb(0xc0, 0xc0, 0xc0),
            comment: egui::Color32::from_rgb(0x70, 0x70, 0x70),

            // Status - Hardware LED inspired
            success: egui::Color32::from_rgb(0x00, 0xcc, 0x44), // Signal present (green LED)
            warning: egui::Color32::from_rgb(0xff, 0xaa, 0x00), // Amber warning
            error: egui::Color32::from_rgb(0xff, 0x00, 0x00),   // Clip indicator (red LED)
            info: egui::Color32::from_rgb(0x4a, 0x7c, 0xa7),    // Info blue

            // VU Meter - Matches Scarlett LED colors exactly
            vu_green: egui::Color32::from_rgb(0x00, 0xcc, 0x44),  // Signal good
            vu_yellow: egui::Color32::from_rgb(0xff, 0xaa, 0x00), // -6dB to -3dB
            vu_red: egui::Color32::from_rgb(0xff, 0x00, 0x00),    // Clip/peak
        }
    }

    /// Original Wavelink professional theme
    pub fn wavelink() -> Self {
        Self {
            preset: ThemePreset::Wavelink,

            // Deep blue palette
            bg: egui::Color32::from_rgb(8, 12, 24),
            bg_dark: egui::Color32::from_rgb(11, 17, 35),
            bg_highlight: egui::Color32::from_rgb(28, 42, 78),
            panel_bg: egui::Color32::from_rgb(15, 22, 40),
            card_bg: egui::Color32::from_rgb(22, 32, 58),
            input_bg: egui::Color32::from_rgb(25, 36, 64),

            // Professional green accents
            accent_primary: egui::Color32::from_rgb(34, 197, 94),
            accent_secondary: egui::Color32::from_rgb(74, 222, 128),
            accent_glow: egui::Color32::from_rgb(22, 163, 74),

            // Secondary
            blue: egui::Color32::from_rgb(96, 165, 250),
            cyan: egui::Color32::from_rgb(34, 211, 238),
            purple: egui::Color32::from_rgb(167, 139, 250),
            magenta: egui::Color32::from_rgb(236, 72, 153),

            // Legacy
            deep_blue: egui::Color32::from_rgb(11, 17, 35),
            medium_blue: egui::Color32::from_rgb(18, 28, 52),
            light_blue: egui::Color32::from_rgb(28, 42, 78),
            green_primary: egui::Color32::from_rgb(34, 197, 94),
            green_secondary: egui::Color32::from_rgb(74, 222, 128),
            green_glow: egui::Color32::from_rgb(22, 163, 74),
            background: egui::Color32::from_rgb(8, 12, 24),

            // Text
            text_primary: egui::Color32::from_rgb(240, 242, 247),
            text_secondary: egui::Color32::from_rgb(180, 188, 200),
            text_muted: egui::Color32::from_rgb(120, 128, 140),
            fg: egui::Color32::from_rgb(240, 242, 247),
            fg_dark: egui::Color32::from_rgb(180, 188, 200),
            comment: egui::Color32::from_rgb(120, 128, 140),

            // Status
            success: egui::Color32::from_rgb(52, 211, 153),
            warning: egui::Color32::from_rgb(251, 191, 36),
            error: egui::Color32::from_rgb(248, 113, 113),
            info: egui::Color32::from_rgb(96, 165, 250),

            // VU Meter
            vu_green: egui::Color32::from_rgb(52, 211, 153),
            vu_yellow: egui::Color32::from_rgb(251, 191, 36),
            vu_red: egui::Color32::from_rgb(248, 113, 113),
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
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, self.text_secondary);
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

        // Touch-friendly spacing
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
            self.card_bg.r(), self.card_bg.g(), self.card_bg.b(), 200
        )
    }

    pub fn channel_strip_border(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(
            self.bg_highlight.r(), self.bg_highlight.g(), self.bg_highlight.b(), 180
        )
    }

    pub fn translucent_panel_bg(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(
            self.panel_bg.r(), self.panel_bg.g(), self.panel_bg.b(), 220
        )
    }

    pub fn translucent_input_bg(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(
            self.input_bg.r(), self.input_bg.g(), self.input_bg.b(), 200
        )
    }

    pub fn glass_button_bg(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(
            self.accent_primary.r(), self.accent_primary.g(), self.accent_primary.b(), 40
        )
    }

    pub fn glass_button_hover(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(
            self.accent_primary.r(), self.accent_primary.g(), self.accent_primary.b(), 80
        )
    }

    pub fn glass_button_active(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(
            self.accent_primary.r(), self.accent_primary.g(), self.accent_primary.b(), 120
        )
    }

    pub fn status_active(&self) -> egui::Color32 {
        self.success
    }

    pub fn status_inactive(&self) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(
            self.text_muted.r(), self.text_muted.g(), self.text_muted.b(), 180
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
            self.bg_dark.r(), self.bg_dark.g(), self.bg_dark.b(), 220
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

// Legacy alias for backwards compatibility
#[allow(dead_code)]
pub type SpaceTheme = WavelinkTheme;
