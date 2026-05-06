//! VU meter widget — GPU-rendered segmented bar.

use gpui::*;
use crate::theme::Theme;

pub struct VuMeter {
    pub level: f32,   // 0.0 – 1.0
    pub height: f32,
    pub segments: usize,
}

impl VuMeter {
    pub fn new(level: f32, height: f32) -> Self {
        Self { level: level.clamp(0.0, 1.0), height, segments: 12 }
    }
}

impl Render for VuMeter {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let level = self.level;
        let segments = self.segments;
        let seg_h = self.height / segments as f32 - 1.0;

        div()
            .flex()
            .flex_col_reverse()
            .gap(px(1.0))
            .h(px(self.height))
            .w(px(8.0))
            .children((0..segments).map(|i| {
                let threshold = i as f32 / segments as f32;
                let active = level > threshold;
                let color = if i >= segments - 2 {
                    Theme::ERROR
                } else if i >= segments - 4 {
                    Theme::WARNING
                } else {
                    Theme::SUCCESS
                };
                div()
                    .h(px(seg_h))
                    .w_full()
                    .rounded(px(1.0))
                    .bg(if active { color } else { Theme::BG_ELEVATED })
            }))
    }
}
