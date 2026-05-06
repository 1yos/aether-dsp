use gpui::*;
use crate::app_state::AppState;
use crate::theme::Theme;

pub struct MixerView { pub state: AppState }

impl MixerView {
    pub fn new(state: AppState, _cx: &mut ViewContext<Self>) -> Self {
        Self { state }
    }
}

impl Render for MixerView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(Theme::BG_CANVAS)
            .child(
                div()
                    .text_size(px(18.0))
                    .text_color(Theme::TEXT_DIM)
                    .child("Mixer — GPU channel strips coming")
            )
    }
}
