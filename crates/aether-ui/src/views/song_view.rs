// Song view — gpui implementation (placeholder, iced version in daw_app.rs)
use gpui::*;
use crate::app_state::AppState;
use crate::theme::Theme;
pub struct SongView { pub state: AppState }
impl SongView {
    pub fn new(state: AppState, _cx: &mut ViewContext<Self>) -> Self { Self { state } }
}
impl Render for SongView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().size_full().bg(Theme::BG_CANVAS)
            .child(div().text_color(Theme::TEXT_DIM).child("Song View"))
    }
}
