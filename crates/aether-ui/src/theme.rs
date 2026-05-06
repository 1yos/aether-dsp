//! Aether Studio design tokens — ported from tokens.css to Rust.
//! All colors are gpui::Rgba values.

use gpui::*;

pub struct Theme;

impl Theme {
    // ── Base canvas ───────────────────────────────────────────────────────────
    pub const BG_VOID:     Rgba = rgba(0x02050aff);
    pub const BG_CANVAS:   Rgba = rgba(0x060c16ff);
    pub const BG_SURFACE:  Rgba = rgba(0x0a1422ff);
    pub const BG_ELEVATED: Rgba = rgba(0x0e1c2eff);

    // ── Borders ───────────────────────────────────────────────────────────────
    pub const BORDER:      Rgba = rgba(0x0f1e2eff);
    pub const BORDER_DIM:  Rgba = rgba(0x1a2a3aff);

    // ── Text ──────────────────────────────────────────────────────────────────
    pub const TEXT:        Rgba = rgba(0xe0e8f0ff);
    pub const TEXT_DIM:    Rgba = rgba(0x475569ff);
    pub const TEXT_MUTED:  Rgba = rgba(0x1e2d3aff);

    // ── Accent ────────────────────────────────────────────────────────────────
    pub const ACCENT:      Rgba = rgba(0x4db8ffff);
    pub const SUCCESS:     Rgba = rgba(0x00e5a0ff);
    pub const WARNING:     Rgba = rgba(0xffd54fff);
    pub const ERROR:       Rgba = rgba(0xef5350ff);

    // ── Regional colors ───────────────────────────────────────────────────────
    pub const EAST_AFRICA:  Rgba = rgba(0xd4a017ff);
    pub const WEST_AFRICA:  Rgba = rgba(0xc0392bff);
    pub const MIDDLE_EAST:  Rgba = rgba(0x1a7a8aff);
    pub const SOUTH_ASIA:   Rgba = rgba(0xe91e8cff);
    pub const EAST_ASIA:    Rgba = rgba(0xc0392bff);
    pub const EUROPE:       Rgba = rgba(0x4a90d9ff);
    pub const AMERICAS:     Rgba = rgba(0x00897bff);
    pub const ELECTRONIC:   Rgba = rgba(0x7c4dffff);

    // ── Typography ────────────────────────────────────────────────────────────
    pub const FONT_SIZE_XS:   f32 = 10.0;
    pub const FONT_SIZE_SM:   f32 = 12.0;
    pub const FONT_SIZE_BASE: f32 = 14.0;
    pub const FONT_SIZE_LG:   f32 = 16.0;

    // ── Spacing ───────────────────────────────────────────────────────────────
    pub const SPACE_1: f32 = 4.0;
    pub const SPACE_2: f32 = 8.0;
    pub const SPACE_3: f32 = 12.0;
    pub const SPACE_4: f32 = 16.0;

    // ── Radius ────────────────────────────────────────────────────────────────
    pub const RADIUS_SM: f32 = 4.0;
    pub const RADIUS_MD: f32 = 8.0;
    pub const RADIUS_LG: f32 = 12.0;

    // ── Track colors (cycling palette) ───────────────────────────────────────
    pub const TRACK_COLORS: &'static [Rgba] = &[
        rgba(0x4db8ffff),
        rgba(0xa78bfaff),
        rgba(0x34d399ff),
        rgba(0xf97316ff),
        rgba(0xf43f5eff),
        rgba(0xfbbf24ff),
        rgba(0x06b6d4ff),
        rgba(0x8b5cf6ff),
    ];
}

/// Convert a packed RGBA u32 to gpui::Rgba
pub fn color_from_u32(c: u32) -> Rgba {
    let r = ((c >> 24) & 0xff) as f32 / 255.0;
    let g = ((c >> 16) & 0xff) as f32 / 255.0;
    let b = ((c >>  8) & 0xff) as f32 / 255.0;
    let a = ( c        & 0xff) as f32 / 255.0;
    Rgba { r, g, b, a }
}
