//! Vertical fader widget — GPU-rendered.

use gpui::*;
use crate::theme::Theme;

pub struct Fader {
    pub value: f32,
    pub color: Rgba,
    pub height: f32,
}

impl Fader {
    pub fn new(value: f32, color: Rgba, height: f32) -> Self {
        Self { value: value.clamp(0.0, 1.0), color, height }
    }
}

impl Render for Fader {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let value = self.value;
        let color = self.color;
        let height = self.height;

        div()
            .w(px(20.0))
            .h(px(height))
            .rounded(px(3.0))
            .bg(Theme::BG_ELEVATED)
            .relative()
            .child(
                // Fill bar
                div()
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .h(px(height * value))
                    .rounded(px(3.0))
                    .bg(color)
            )
            .child(
                // Thumb
                div()
                    .absolute()
                    .left(px(-2.0))
                    .right(px(-2.0))
                    .bottom(px(height * value - 4.0))
                    .h(px(8.0))
                    .rounded(px(2.0))
                    .bg(Theme::TEXT)
                    .cursor(CursorStyle::ResizeRow)
            )
    }
}
