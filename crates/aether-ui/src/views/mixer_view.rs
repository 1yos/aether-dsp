//! MixerView — GPU-rendered channel strips.
//!
//! Each channel strip has:
//!   - Channel name + color bar
//!   - VU meter (GPU-rendered segments)
//!   - Volume fader (vertical)
//!   - Pan knob
//!   - Mute / Solo buttons
//!   - Volume readout

use gpui::*;
use crate::app_state::AppState;
use crate::theme::Theme;

const STRIP_W: f32 = 64.0;
const FADER_H: f32 = 100.0;
const VU_H: f32 = 60.0;
const VU_SEGMENTS: usize = 12;

pub struct MixerView {
    state: AppState,
}

impl MixerView {
    pub fn new(state: AppState, _cx: &mut ViewContext<Self>) -> Self {
        Self { state }
    }

    fn render_channel_strip(
        &self,
        ch_id: u64,
        name: &str,
        color: u32,
        volume: f32,
        pan: f32,
        muted: bool,
        solo: bool,
        audio_active: bool,
        cx: &mut ViewContext<Self>,
    ) -> impl IntoElement {
        let color_rgba = crate::theme::color_from_u32(color);
        let level = if audio_active { volume * (0.4 + (ch_id as f32 * 0.13).sin().abs() * 0.5) } else { 0.0 };
        let state = self.state.clone();

        div()
            .w(px(STRIP_W))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(4.0))
            .bg(Theme::BG_SURFACE)
            .border_1()
            .border_color(Theme::BORDER)
            .rounded(px(Theme::RADIUS_SM))
            .px(px(4.0))
            .py(px(8.0))
            .flex_shrink_0()
            .child(
                // Channel name
                div()
                    .text_size(px(9.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(color_rgba)
                    .overflow_hidden()
                    .text_ellipsis()
                    .w_full()
                    .text_align(TextAlign::Center)
                    .child(name.to_string())
            )
            .child(
                // VU meter (GPU canvas)
                canvas(
                    move |bounds, cx| {
                        let ox = bounds.origin.x.0;
                        let oy = bounds.origin.y.0;
                        let seg_h = VU_H / VU_SEGMENTS as f32 - 1.0;
                        let w = bounds.size.width.0;

                        for i in 0..VU_SEGMENTS {
                            let threshold = i as f32 / VU_SEGMENTS as f32;
                            let active = level > threshold;
                            let seg_color = if i >= VU_SEGMENTS - 2 {
                                Theme::ERROR
                            } else if i >= VU_SEGMENTS - 4 {
                                Theme::WARNING
                            } else {
                                Theme::SUCCESS
                            };
                            let y = oy + VU_H - (i + 1) as f32 * (seg_h + 1.0);
                            cx.paint_quad(PaintQuad {
                                bounds: Rect {
                                    origin: point(px(ox), px(y)),
                                    size: size(px(w), px(seg_h)),
                                },
                                corner_radii: Corners::all(px(1.0)),
                                background: if active { seg_color } else { rgba(0x0a1520ff) },
                                border_widths: Default::default(),
                                border_color: Rgba::transparent_black(),
                            });
                        }
                    },
                    |_, _| {},
                )
                .w(px(STRIP_W - 16.0))
                .h(px(VU_H))
            )
            .child(
                // Pan display
                div()
                    .text_size(px(8.0))
                    .text_color(Theme::TEXT_DIM)
                    .font(Font { family: "monospace".into(), ..Default::default() })
                    .child(if pan.abs() < 0.01 {
                        "C".to_string()
                    } else if pan > 0.0 {
                        format!("R{}", (pan * 100.0) as i32)
                    } else {
                        format!("L{}", (-pan * 100.0) as i32)
                    })
            )
            .child(
                // Fader (vertical slider rendered as canvas)
                canvas(
                    move |bounds, cx| {
                        let ox = bounds.origin.x.0;
                        let oy = bounds.origin.y.0;
                        let w = bounds.size.width.0;
                        let h = bounds.size.height.0;

                        // Track
                        cx.paint_quad(PaintQuad {
                            bounds: Rect {
                                origin: point(px(ox + w / 2.0 - 2.0), px(oy)),
                                size: size(px(4.0), px(h)),
                            },
                            corner_radii: Corners::all(px(2.0)),
                            background: rgba(0x0a1520ff),
                            border_widths: Default::default(),
                            border_color: Rgba::transparent_black(),
                        });

                        // Fill
                        let fill_h = h * volume;
                        cx.paint_quad(PaintQuad {
                            bounds: Rect {
                                origin: point(px(ox + w / 2.0 - 2.0), px(oy + h - fill_h)),
                                size: size(px(4.0), px(fill_h)),
                            },
                            corner_radii: Corners::all(px(2.0)),
                            background: color_rgba,
                            border_widths: Default::default(),
                            border_color: Rgba::transparent_black(),
                        });

                        // Thumb
                        let thumb_y = oy + h * (1.0 - volume) - 4.0;
                        cx.paint_quad(PaintQuad {
                            bounds: Rect {
                                origin: point(px(ox + 2.0), px(thumb_y)),
                                size: size(px(w - 4.0), px(8.0)),
                            },
                            corner_radii: Corners::all(px(2.0)),
                            background: rgba(0xe0e8f0ff),
                            border_widths: Default::default(),
                            border_color: Rgba::transparent_black(),
                        });
                    },
                    |_, _| {},
                )
                .w(px(STRIP_W - 16.0))
                .h(px(FADER_H))
                .cursor(CursorStyle::ResizeRow)
            )
            .child(
                // Volume readout
                div()
                    .text_size(px(8.0))
                    .text_color(Theme::TEXT_DIM)
                    .font(Font { family: "monospace".into(), ..Default::default() })
                    .child(format!("{}%", (volume * 100.0) as u32))
            )
            .child(
                // M / S buttons
                div()
                    .flex()
                    .gap(px(3.0))
                    .child(
                        div()
                            .px(px(4.0))
                            .h(px(16.0))
                            .flex()
                            .items_center()
                            .rounded(px(3.0))
                            .bg(if muted { rgba(0xef53501a) } else { Rgba::transparent_black() })
                            .border_1()
                            .border_color(if muted { rgba(0xef535050) } else { Theme::BORDER })
                            .text_color(if muted { Theme::ERROR } else { Theme::TEXT_DIM })
                            .text_size(px(8.0))
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, {
                                let state = state.clone();
                                move |_, cx| {
                                    let mut s = state.lock().unwrap();
                                    if let Some(ch) = s.channels.iter_mut().find(|c| c.id == ch_id) {
                                        ch.muted = !ch.muted;
                                    }
                                    drop(s);
                                    cx.notify();
                                }
                            })
                            .child("M")
                    )
                    .child(
                        div()
                            .px(px(4.0))
                            .h(px(16.0))
                            .flex()
                            .items_center()
                            .rounded(px(3.0))
                            .bg(if solo { rgba(0xffd54f1a) } else { Rgba::transparent_black() })
                            .border_1()
                            .border_color(if solo { rgba(0xffd54f50) } else { Theme::BORDER })
                            .text_color(if solo { Theme::WARNING } else { Theme::TEXT_DIM })
                            .text_size(px(8.0))
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, {
                                let state = state.clone();
                                move |_, cx| {
                                    let mut s = state.lock().unwrap();
                                    if let Some(ch) = s.channels.iter_mut().find(|c| c.id == ch_id) {
                                        ch.solo = !ch.solo;
                                    }
                                    drop(s);
                                    cx.notify();
                                }
                            })
                            .child("S")
                    )
            )
    }
}

impl Render for MixerView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let state = self.state.lock().unwrap();
        let channels = state.channels.clone();
        let audio_active = state.audio_active;
        drop(state);

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(Theme::BG_CANVAS)
            .child(
                // Header
                div()
                    .h(px(36.0))
                    .w_full()
                    .flex()
                    .items_center()
                    .px(px(16.0))
                    .gap(px(8.0))
                    .bg(Theme::BG_SURFACE)
                    .border_b_1()
                    .border_color(Theme::BORDER)
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(Theme::TEXT)
                            .child("Mixer")
                    )
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(Theme::TEXT_DIM)
                            .child(format!("{} channels", channels.len()))
                    )
            )
            .child(
                // Channel strips
                div()
                    .flex_1()
                    .flex()
                    .overflow_x_scroll()
                    .overflow_y_hidden()
                    .px(px(8.0))
                    .py(px(12.0))
                    .gap(px(4.0))
                    .items_end()
                    .children(
                        channels.iter().map(|ch| {
                            self.render_channel_strip(
                                ch.id,
                                &ch.name,
                                ch.color,
                                ch.volume,
                                ch.pan,
                                ch.muted,
                                ch.solo,
                                audio_active,
                                cx,
                            )
                        })
                    )
            )
    }
}
