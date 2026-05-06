//! Piano keyboard widget — GPU-rendered.
//! Used in the piano roll view and the instrument browser.

use gpui::*;
use crate::theme::Theme;

const BLACK_PCS: &[u8] = &[1, 3, 6, 8, 10];

pub struct PianoKeys {
    pub start_midi: u8,
    pub end_midi: u8,
    pub key_height: f32,
    pub active_notes: Vec<u8>,
    pub scale_color: Rgba,
    pub in_scale: Vec<bool>, // 12 booleans, one per pitch class
}

impl PianoKeys {
    pub fn new(start_midi: u8, end_midi: u8, key_height: f32) -> Self {
        Self {
            start_midi,
            end_midi,
            key_height,
            active_notes: Vec::new(),
            scale_color: Theme::ACCENT,
            in_scale: vec![true; 12],
        }
    }
}

impl Render for PianoKeys {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let key_h = self.key_height;
        let scale_color = self.scale_color;
        let in_scale = self.in_scale.clone();
        let active = self.active_notes.clone();
        let start = self.start_midi;
        let end = self.end_midi;

        div()
            .flex()
            .flex_col()
            .w(px(48.0))
            .children((start..=end).rev().map(|midi| {
                let pc = (midi % 12) as usize;
                let is_black = BLACK_PCS.contains(&(pc as u8));
                let in_sc = in_scale.get(pc).copied().unwrap_or(false);
                let is_active = active.contains(&midi);
                let is_c = pc == 0;

                let bg = if is_active {
                    scale_color
                } else if is_black {
                    if in_sc { rgba(0xa78bfa26) } else { rgba(0x0a1218ff) }
                } else {
                    if in_sc {
                        Rgba { r: scale_color.r, g: scale_color.g, b: scale_color.b, a: 0.08 }
                    } else {
                        rgba(0x0d1520ff)
                    }
                };

                div()
                    .h(px(key_h))
                    .w_full()
                    .flex()
                    .items_center()
                    .px(px(5.0))
                    .bg(bg)
                    .border_b_1()
                    .border_color(rgba(0x0a1520ff))
                    .border_l(px(3.0))
                    .border_color(if in_sc { scale_color } else if is_black { rgba(0x1a2a3aff) } else { rgba(0x1e2d3dff) })
                    .when(is_c, |el| {
                        el.child(
                            div()
                                .text_size(px(8.0))
                                .text_color(if in_sc { scale_color } else { Theme::TEXT_MUTED })
                                .font_weight(FontWeight::BOLD)
                                .child(format!("C{}", midi / 12 - 1))
                        )
                    })
            }))
    }
}
