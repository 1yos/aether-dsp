//! Aether Studio design tokens.
//! Color constants for the iced UI.

/// Pack RGBA bytes into a u32 for AppState color fields
pub const fn rgba_u32(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
}

pub struct Theme;
impl Theme {
    pub const BG_VOID:    u32 = rgba_u32(0x02, 0x05, 0x0a, 0xff);
    pub const BG_CANVAS:  u32 = rgba_u32(0x06, 0x0c, 0x16, 0xff);
    pub const BG_SURFACE: u32 = rgba_u32(0x0a, 0x14, 0x22, 0xff);
    pub const ACCENT:     u32 = rgba_u32(0x4d, 0xb8, 0xff, 0xff);
    pub const SUCCESS:    u32 = rgba_u32(0x00, 0xe5, 0xa0, 0xff);
    pub const ERROR:      u32 = rgba_u32(0xef, 0x53, 0x50, 0xff);
    pub const WARNING:    u32 = rgba_u32(0xff, 0xd5, 0x4f, 0xff);
}
