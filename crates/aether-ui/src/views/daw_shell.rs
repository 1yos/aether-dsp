//! DawShell — the root view of Aether Studio.
//!
//! Layout:
//!   ┌─────────────────────────────────────────────┐
//!   │  TopBar (48px) — transport + view tabs      │
//!   ├──────────┬──────────────────────────────────┤
//!   │ Browser  │  Active View                     │
//!   │  220px   │  (Song / Piano Roll / Mixer / …) │
//!   ├──────────┴──────────────────────────────────┤
//!   │  Properties panel (180px)                   │
//!   └─────────────────────────────────────────────┘

use gpui::*;
use crate::app_state::{AppState, ActiveView};
use crate::theme::Theme;
use crate::views::{
    song_view::SongView,
    piano_roll::PianoRollView,
    mixer_view::MixerView,
    patcher_view::PatcherView,
    perform_view::PerformView,
};

pub struct DawShell {
    state: AppState,
    song_view: View<SongView>,
    piano_roll: View<PianoRollView>,
    mixer: View<MixerView>,
    patcher: View<PatcherView>,
    perform: View<PerformView>,
}

impl DawShell {
    pub fn new(state: AppState, cx: &mut ViewContext<Self>) -> Self {
        let song_view   = cx.new_view(|cx| SongView::new(state.clone(), cx));
        let piano_roll  = cx.new_view(|cx| PianoRollView::new(state.clone(), cx));
        let mixer       = cx.new_view(|cx| MixerView::new(state.clone(), cx));
        let patcher     = cx.new_view(|cx| PatcherView::new(state.clone(), cx));
        let perform     = cx.new_view(|cx| PerformView::new(state.clone(), cx));

        // Keyboard shortcuts
        cx.on_key_event::<KeyDownEvent>(|this, event, cx| {
            match event.keystroke.key.as_str() {
                "f1" => this.set_view(ActiveView::Song, cx),
                "f2" => this.set_view(ActiveView::PianoRoll, cx),
                "f3" => this.set_view(ActiveView::Mixer, cx),
                "f4" => this.set_view(ActiveView::Patcher, cx),
                "f5" => this.set_view(ActiveView::Perform, cx),
                " "  => this.toggle_play(cx),
                _    => {}
            }
        });

        Self { state, song_view, piano_roll, mixer, patcher, perform }
    }

    fn set_view(&mut self, view: ActiveView, cx: &mut ViewContext<Self>) {
        self.state.lock().unwrap().active_view = view;
        cx.notify();
    }

    fn toggle_play(&mut self, cx: &mut ViewContext<Self>) {
        let mut s = self.state.lock().unwrap();
        s.transport.is_playing = !s.transport.is_playing;
        drop(s);
        cx.notify();
    }

    fn render_top_bar(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let state = self.state.lock().unwrap();
        let active = state.active_view;
        let bpm = state.transport.bpm;
        let is_playing = state.transport.is_playing;
        let is_recording = state.transport.is_recording;
        let engine_ok = state.engine_status == crate::app_state::EngineStatus::Running;
        drop(state);

        let views = [
            (ActiveView::Song,      "≡", "Song",       "F1"),
            (ActiveView::PianoRoll, "♩", "Piano Roll",  "F2"),
            (ActiveView::Mixer,     "⊟", "Mixer",       "F3"),
            (ActiveView::Patcher,   "⬡", "Patcher",     "F4"),
            (ActiveView::Perform,   "🎚", "Perform",    "F5"),
        ];

        div()
            .h(px(48.0))
            .w_full()
            .flex()
            .items_center()
            .px(px(12.0))
            .gap(px(4.0))
            .bg(Theme::BG_SURFACE)
            .border_b_1()
            .border_color(Theme::BORDER)
            .child(
                // Logo
                div()
                    .flex()
                    .items_baseline()
                    .gap(px(2.0))
                    .mr(px(12.0))
                    .child(
                        div()
                            .text_size(px(15.0))
                            .font_weight(FontWeight::EXTRA_BOLD)
                            .text_color(Theme::ACCENT)
                            .child("Aether")
                    )
                    .child(
                        div()
                            .text_size(px(15.0))
                            .font_weight(FontWeight::LIGHT)
                            .text_color(Theme::TEXT_DIM)
                            .child("Studio")
                    )
            )
            .child(
                // View tabs
                div()
                    .flex()
                    .gap(px(2.0))
                    .mr(px(8.0))
                    .children(views.iter().map(|(view, icon, label, shortcut)| {
                        let is_active = active == *view;
                        let view = *view;
                        div()
                            .flex()
                            .items_center()
                            .gap(px(5.0))
                            .px(px(12.0))
                            .h(px(28.0))
                            .rounded(px(Theme::RADIUS_SM))
                            .bg(if is_active { rgba(0x4db8ff1a) } else { Rgba::transparent_black() })
                            .border_1()
                            .border_color(if is_active { rgba(0x4db8ff40) } else { Rgba::transparent_black() })
                            .text_color(if is_active { Theme::ACCENT } else { Theme::TEXT_DIM })
                            .text_size(px(12.0))
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, move |_, cx| {
                                // View switching handled via keyboard for now
                                // TODO: wire click → set_view
                                let _ = (view, cx);
                            })
                            .child(format!("{} {}", icon, label))
                            .id(SharedString::from(format!("tab-{:?}", view)))
                    }))
            )
            .child(
                // Divider
                div().w(px(1.0)).h(px(20.0)).bg(Theme::BORDER)
            )
            .child(
                // Transport
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child(
                        // Stop
                        div()
                            .px(px(10.0))
                            .h(px(28.0))
                            .flex()
                            .items_center()
                            .rounded(px(Theme::RADIUS_SM))
                            .text_color(Theme::TEXT_DIM)
                            .text_size(px(12.0))
                            .cursor_pointer()
                            .child("■ Stop")
                    )
                    .child(
                        // Play
                        div()
                            .px(px(10.0))
                            .h(px(28.0))
                            .flex()
                            .items_center()
                            .rounded(px(Theme::RADIUS_SM))
                            .bg(if is_playing { rgba(0x4db8ff1a) } else { Rgba::transparent_black() })
                            .border_1()
                            .border_color(if is_playing { rgba(0x4db8ff40) } else { Rgba::transparent_black() })
                            .text_color(if is_playing { Theme::SUCCESS } else { Theme::TEXT_DIM })
                            .text_size(px(12.0))
                            .cursor_pointer()
                            .child(if is_playing { "▶ Playing" } else { "▶ Play" })
                    )
                    .child(
                        // Record
                        div()
                            .px(px(10.0))
                            .h(px(28.0))
                            .flex()
                            .items_center()
                            .rounded(px(Theme::RADIUS_SM))
                            .bg(if is_recording { rgba(0xef53501a) } else { Rgba::transparent_black() })
                            .text_color(if is_recording { Theme::ERROR } else { Theme::TEXT_DIM })
                            .text_size(px(12.0))
                            .cursor_pointer()
                            .child("● Rec")
                    )
            )
            .child(
                // Divider
                div().w(px(1.0)).h(px(20.0)).bg(Theme::BORDER)
            )
            .child(
                // BPM
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(Theme::TEXT_MUTED)
                            .child("BPM")
                    )
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(Theme::TEXT)
                            .font(Font { family: "monospace".into(), ..Default::default() })
                            .child(format!("{:.0}", bpm))
                    )
            )
            .child(div().flex_1())
            .child(
                // Engine status
                div()
                    .flex()
                    .items_center()
                    .gap(px(5.0))
                    .child(
                        div()
                            .w(px(7.0))
                            .h(px(7.0))
                            .rounded_full()
                            .bg(if engine_ok { Theme::SUCCESS } else { Theme::ERROR })
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(if engine_ok { Theme::SUCCESS } else { Theme::TEXT_DIM })
                            .child(if engine_ok { "Live" } else { "Offline" })
                    )
            )
    }
}

impl Render for DawShell {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let active_view = self.state.lock().unwrap().active_view;

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(Theme::BG_VOID)
            .text_color(Theme::TEXT)
            .font_size(px(Theme::FONT_SIZE_BASE))
            .child(self.render_top_bar(cx))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .overflow_hidden()
                    .min_h_0()
                    .child(
                        match active_view {
                            ActiveView::Song      => div().flex_1().child(self.song_view.clone()),
                            ActiveView::PianoRoll => div().flex_1().child(self.piano_roll.clone()),
                            ActiveView::Mixer     => div().flex_1().child(self.mixer.clone()),
                            ActiveView::Patcher   => div().flex_1().child(self.patcher.clone()),
                            ActiveView::Perform   => div().flex_1().child(self.perform.clone()),
                        }
                    )
            )
    }
}
