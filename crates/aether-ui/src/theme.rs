// Aether Studio theme and styling

use iced::Color;

/// Professional Dark Studio theme colors
pub struct AetherTheme;

impl AetherTheme {
    // Background hierarchy
    pub const APP_BACKGROUND: Color = Color::from_rgb(0.102, 0.102, 0.102); // #1a1a1a
    pub const PANEL_BACKGROUND: Color = Color::from_rgb(0.141, 0.141, 0.141); // #242424
    pub const CANVAS_BACKGROUND: Color = Color::from_rgb(0.118, 0.118, 0.118); // #1e1e1e
    pub const NODE_BACKGROUND: Color = Color::from_rgb(0.176, 0.176, 0.176); // #2d2d2d
    pub const HOVER_STATE: Color = Color::from_rgb(0.208, 0.208, 0.208); // #353535
    pub const ACTIVE_STATE: Color = Color::from_rgb(0.239, 0.239, 0.239); // #3d3d3d

    // Accent colors
    pub const PRIMARY: Color = Color::from_rgb(0.290, 0.624, 1.0); // #4a9eff (blue)
    pub const SUCCESS: Color = Color::from_rgb(0.290, 0.867, 0.502); // #4ade80 (green)
    pub const WARNING: Color = Color::from_rgb(0.984, 0.573, 0.235); // #fb923c (orange)
    pub const ERROR: Color = Color::from_rgb(0.937, 0.267, 0.267); // #ef4444 (red)
    pub const INFO: Color = Color::from_rgb(0.655, 0.545, 0.980); // #a78bfa (purple)

    // Text colors
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.898, 0.898, 0.898); // #e5e5e5
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.639, 0.639, 0.639); // #a3a3a3
    pub const TEXT_DISABLED: Color = Color::from_rgb(0.322, 0.322, 0.322); // #525252
    pub const TEXT_LINK: Color = Color::from_rgb(0.376, 0.647, 0.980); // #60a5fa

    // Node type colors (subtle, desaturated)
    pub const NODE_AUDIO_IO: Color = Color::from_rgb(0.231, 0.510, 0.965); // #3b82f6
    pub const NODE_GENERATOR: Color = Color::from_rgb(0.063, 0.725, 0.506); // #10b981
    pub const NODE_EFFECT: Color = Color::from_rgb(0.545, 0.361, 0.965); // #8b5cf6
    pub const NODE_MODULATOR: Color = Color::from_rgb(0.961, 0.620, 0.043); // #f59e0b
    pub const NODE_UTILITY: Color = Color::from_rgb(0.420, 0.451, 0.502); // #6b7280
    pub const NODE_PARAMETER: Color = Color::from_rgb(0.925, 0.286, 0.600); // #ec4899

    // Cable colors (based on data type)
    pub const CABLE_AUDIO: Color = Color::from_rgb(0.290, 0.867, 0.502); // #4ade80 (green)
    pub const CABLE_CONTROL: Color = Color::from_rgb(0.984, 0.749, 0.141); // #fbbf24 (yellow)
    pub const CABLE_MIDI: Color = Color::from_rgb(0.655, 0.545, 0.980); // #a78bfa (purple)
    pub const CABLE_MODULATION: Color = Color::from_rgb(0.984, 0.573, 0.235); // #fb923c (orange)
}

/// Spacing system (base unit: 4px)
pub struct Spacing;

impl Spacing {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
}

/// Border radius
pub struct BorderRadius;

impl BorderRadius {
    pub const SMALL: f32 = 4.0;
    pub const MEDIUM: f32 = 6.0;
    pub const LARGE: f32 = 8.0;
}
