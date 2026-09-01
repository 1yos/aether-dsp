//! Aether Studio design system — colors, spacing, typography.

use iced::Color;

pub struct Theme;

impl Theme {
    // ── Backgrounds ──────────────────────────────────────────────────────────
    pub const APP_BG: Color        = Color { r: 0.051, g: 0.051, b: 0.051, a: 1.0 }; // #0D0D0D
    pub const PANEL_BG: Color      = Color { r: 0.078, g: 0.078, b: 0.078, a: 1.0 }; // #141414
    pub const SURFACE: Color       = Color { r: 0.110, g: 0.110, b: 0.110, a: 1.0 }; // #1C1C1C
    pub const BORDER: Color        = Color { r: 0.165, g: 0.165, b: 0.165, a: 1.0 }; // #2A2A2A

    // ── Accent (Ethiopian gold/amber) ─────────────────────────────────────────
    pub const ACCENT: Color        = Color { r: 0.788, g: 0.588, b: 0.227, a: 1.0 }; // #C9963A
    pub const ACCENT_DIM: Color    = Color { r: 0.541, g: 0.392, b: 0.125, a: 1.0 }; // #8A6420

    // ── Text ──────────────────────────────────────────────────────────────────
    pub const TEXT_PRIMARY: Color  = Color { r: 0.941, g: 0.929, b: 0.910, a: 1.0 }; // #F0EDE8
    pub const TEXT_SECONDARY: Color= Color { r: 0.541, g: 0.518, b: 0.502, a: 1.0 }; // #8A8480
    pub const TEXT_DISABLED: Color = Color { r: 0.267, g: 0.267, b: 0.267, a: 1.0 }; // #444444

    // ── Status ────────────────────────────────────────────────────────────────
    pub const GREEN: Color         = Color { r: 0.290, g: 0.867, b: 0.502, a: 1.0 }; // #4ADE80
    pub const RED: Color           = Color { r: 0.937, g: 0.267, b: 0.267, a: 1.0 }; // #EF4444
    pub const BLUE: Color          = Color { r: 0.376, g: 0.647, b: 0.980, a: 1.0 }; // #60A5FA
}

// ── Spacing ───────────────────────────────────────────────────────────────────

pub struct Spacing;

impl Spacing {
    pub const XS: u16 = 4;
    pub const SM: u16 = 8;
    pub const MD: u16 = 16;
    pub const LG: u16 = 24;
    pub const XL: u16 = 32;
}

// ── Track colors (randomized palette) ────────────────────────────────────────

pub const TRACK_COLORS: &[Color] = &[
    Color { r: 0.788, g: 0.588, b: 0.227, a: 1.0 }, // amber
    Color { r: 0.290, g: 0.624, b: 1.000, a: 1.0 }, // blue
    Color { r: 0.290, g: 0.867, b: 0.502, a: 1.0 }, // green
    Color { r: 0.655, g: 0.545, b: 0.980, a: 1.0 }, // purple
    Color { r: 0.984, g: 0.573, b: 0.235, a: 1.0 }, // orange
    Color { r: 0.925, g: 0.286, b: 0.600, a: 1.0 }, // pink
    Color { r: 0.063, g: 0.725, b: 0.506, a: 1.0 }, // teal
    Color { r: 0.961, g: 0.620, b: 0.043, a: 1.0 }, // yellow
];

pub fn track_color(index: usize) -> Color {
    TRACK_COLORS[index % TRACK_COLORS.len()]
}
