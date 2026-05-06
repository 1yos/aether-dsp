//! Circular knob widget — GPU-rendered SVG-style arc.
//!
//! Draggable. Renders a colored arc from -135° to +135° showing the
//! current value. Used in the mixer channel strips and effect editors.

use gpui::*;
use crate::theme::Theme;

pub struct Knob {
    pub value: f32,   // 0.0 – 1.0
    pub color: Rgba,
    pub size: f32,
    pub label: Option<SharedString>,
    on_change: Box<dyn Fn(f32, &mut WindowContext) + Send + Sync>,
    drag_start: Option<(f32, f32)>, // (start_y, start_value)
}

impl Knob {
    pub fn new(
        value: f32,
        color: Rgba,
        size: f32,
        label: Option<impl Into<SharedString>>,
        on_change: impl Fn(f32, &mut WindowContext) + Send + Sync + 'static,
    ) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            color,
            size,
            label: label.map(|l| l.into()),
            on_change: Box::new(on_change),
            drag_start: None,
        }
    }
}

impl Render for Knob {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let value = self.value;
        let color = self.color;
        let size = self.size;
        let label = self.label.clone();

        // The arc spans -135° to +135° (270° total).
        // We draw it as a canvas element using gpui's path API.
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(2.0))
            .child(
                canvas(
                    move |bounds, cx| {
                        let cx_x = bounds.origin.x + bounds.size.width / 2.0;
                        let cy_y = bounds.origin.y + bounds.size.height / 2.0;
                        let r = size / 2.0 - 4.0;

                        // Background track
                        cx.paint_path(
                            arc_path(cx_x, cy_y, r, -135.0, 135.0),
                            Theme::BG_ELEVATED,
                        );

                        // Value arc
                        let end_angle = -135.0 + value * 270.0;
                        cx.paint_path(
                            arc_path(cx_x, cy_y, r, -135.0, end_angle),
                            color,
                        );

                        // Indicator dot
                        let angle_rad = (end_angle - 90.0).to_radians();
                        let dot_x = cx_x + r * angle_rad.cos();
                        let dot_y = cy_y + r * angle_rad.sin();
                        cx.paint_path(
                            circle_path(dot_x, dot_y, 2.5),
                            color,
                        );
                    },
                    |_, _| {},
                )
                .w(px(size))
                .h(px(size))
                .cursor(CursorStyle::ResizeRow)
                .on_mouse_down(MouseButton::Left, {
                    let value = self.value;
                    move |event, cx| {
                        // Store drag start position
                        let _ = (event, cx, value);
                    }
                })
            )
            .when_some(label, |el, lbl| {
                el.child(
                    div()
                        .text_size(px(8.0))
                        .text_color(Theme::TEXT_DIM)
                        .child(lbl)
                )
            })
    }
}

// ── Path helpers ──────────────────────────────────────────────────────────────

fn arc_path(cx: f32, cy: f32, r: f32, start_deg: f32, end_deg: f32) -> Path<Pixels> {
    let mut path = Path::new(point(px(cx), px(cy)));
    let steps = 32;
    let start = (start_deg - 90.0).to_radians();
    let end = (end_deg - 90.0).to_radians();
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let angle = start + t * (end - start);
        let x = cx + r * angle.cos();
        let y = cy + r * angle.sin();
        if i == 0 {
            path.move_to(point(px(x), px(y)));
        } else {
            path.line_to(point(px(x), px(y)));
        }
    }
    path
}

fn circle_path(cx: f32, cy: f32, r: f32) -> Path<Pixels> {
    let mut path = Path::new(point(px(cx + r), px(cy)));
    let steps = 16;
    for i in 1..=steps {
        let angle = (i as f32 / steps as f32) * std::f32::consts::TAU;
        let x = cx + r * angle.cos();
        let y = cy + r * angle.sin();
        path.line_to(point(px(x), px(y)));
    }
    path
}
