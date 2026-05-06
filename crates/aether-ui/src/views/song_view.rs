//! SongView — GPU-rendered timeline / arrange view.
//!
//! This is the main DAW view. It renders:
//!   - Track headers (left panel, 200px)
//!   - Timeline ruler with bar numbers
//!   - Track lanes with clips
//!   - Playhead line
//!   - Loop region overlay
//!
//! All rendering is done via gpui's canvas API — direct GPU draw calls.
//! No DOM, no layout engine, no browser overhead.
//!
//! Performance targets:
//!   - 60fps with 100+ tracks and 1000+ clips
//!   - Playhead updates at audio callback rate (48kHz → UI at 60fps)
//!   - Clip drag/resize with zero jitter

use gpui::*;
use crate::app_state::{AppState, Track, Clip, TrackType};
use crate::theme::Theme;

// ── Layout constants ──────────────────────────────────────────────────────────

const HEADER_W: f32 = 200.0;
const RULER_H: f32 = 32.0;
const BEAT_W_BASE: f32 = 32.0;
const TOTAL_BEATS: usize = 256;
const MIN_TRACK_H: f32 = 48.0;
const MAX_TRACK_H: f32 = 200.0;

// ── Interaction state ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum DragState {
    None,
    DraggingClip { clip_id: u64, start_x: f32, start_beat: f64 },
    ResizingClip { clip_id: u64, start_x: f32, start_len: f64 },
    ResizingTrack { track_id: u64, start_y: f32, start_h: f32 },
    Panning { start_x: f32, start_scroll: f32 },
}

// ── SongView ──────────────────────────────────────────────────────────────────

pub struct SongView {
    state: AppState,
    zoom: f32,
    scroll_x: f32,
    drag: DragState,
    snap: SnapMode,
    context_menu: Option<ContextMenuState>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SnapMode {
    Bar,
    Beat,
    Half,
    Quarter,
    Eighth,
    Off,
}

#[derive(Debug, Clone)]
struct ContextMenuState {
    x: f32,
    y: f32,
    target: ContextMenuTarget,
}

#[derive(Debug, Clone)]
enum ContextMenuTarget {
    Track(u64),
    Clip(u64),
    Lane(u64), // track_id for empty lane click
}

impl SongView {
    pub fn new(state: AppState, _cx: &mut ViewContext<Self>) -> Self {
        Self {
            state,
            zoom: 1.0,
            scroll_x: 0.0,
            drag: DragState::None,
            snap: SnapMode::Beat,
            context_menu: None,
        }
    }

    fn beat_w(&self) -> f32 {
        BEAT_W_BASE * self.zoom
    }

    fn snap_beat(&self, beat: f64, time_sig: u8) -> f64 {
        let s = match self.snap {
            SnapMode::Bar     => time_sig as f64,
            SnapMode::Beat    => 1.0,
            SnapMode::Half    => 0.5,
            SnapMode::Quarter => 0.25,
            SnapMode::Eighth  => 0.125,
            SnapMode::Off     => return beat,
        };
        (beat / s).round() * s
    }

    fn beat_from_x(&self, x: f32) -> f64 {
        ((x + self.scroll_x) / self.beat_w()) as f64
    }

    fn x_from_beat(&self, beat: f64) -> f32 {
        beat as f32 * self.beat_w() - self.scroll_x
    }

    // ── Rendering helpers ─────────────────────────────────────────────────────

    fn render_toolbar(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let zoom = self.zoom;
        let snap = self.snap;

        div()
            .h(px(36.0))
            .w_full()
            .flex()
            .items_center()
            .px(px(10.0))
            .gap(px(6.0))
            .bg(Theme::BG_SURFACE)
            .border_b_1()
            .border_color(Theme::BORDER)
            .child(
                // + Instrument Track
                div()
                    .px(px(10.0))
                    .h(px(24.0))
                    .flex()
                    .items_center()
                    .rounded(px(Theme::RADIUS_SM))
                    .bg(rgba(0x4db8ff12))
                    .border_1()
                    .border_color(rgba(0x4db8ff30))
                    .text_color(Theme::ACCENT)
                    .text_size(px(11.0))
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, {
                        let state = self.state.clone();
                        move |_, cx| {
                            let mut s = state.lock().unwrap();
                            let id = s.next_id();
                            let color_idx = s.tracks.len() % 8;
                            let color = [
                                0x4db8ffff_u32, 0xa78bfaff, 0x34d399ff, 0xf97316ff,
                                0xf43f5eff, 0xfbbf24ff, 0x06b6d4ff, 0x8b5cf6ff,
                            ][color_idx];
                            s.tracks.push(crate::app_state::Track {
                                id,
                                name: format!("Track {}", s.tracks.len() + 1),
                                track_type: TrackType::Instrument,
                                color,
                                muted: false,
                                solo: false,
                                armed: false,
                                volume: 0.8,
                                pan: 0.0,
                                height: 72.0,
                                clips: Vec::new(),
                                effects: Vec::new(),
                            });
                            drop(s);
                            cx.notify();
                        }
                    })
                    .child("+ Instrument")
            )
            .child(
                div()
                    .px(px(10.0))
                    .h(px(24.0))
                    .flex()
                    .items_center()
                    .rounded(px(Theme::RADIUS_SM))
                    .border_1()
                    .border_color(Theme::BORDER)
                    .text_color(Theme::TEXT_DIM)
                    .text_size(px(11.0))
                    .cursor_pointer()
                    .child("+ Audio")
            )
            .child(div().w(px(1.0)).h(px(16.0)).bg(Theme::BORDER))
            .child(
                div()
                    .text_size(px(10.0))
                    .text_color(Theme::TEXT_DIM)
                    .child("Snap:")
            )
            .child(
                // Snap selector (simplified — shows current mode)
                div()
                    .px(px(8.0))
                    .h(px(24.0))
                    .flex()
                    .items_center()
                    .rounded(px(Theme::RADIUS_SM))
                    .bg(Theme::BG_ELEVATED)
                    .border_1()
                    .border_color(Theme::BORDER)
                    .text_color(Theme::TEXT)
                    .text_size(px(10.0))
                    .cursor_pointer()
                    .child(match snap {
                        SnapMode::Bar     => "Bar",
                        SnapMode::Beat    => "Beat",
                        SnapMode::Half    => "1/2",
                        SnapMode::Quarter => "1/4",
                        SnapMode::Eighth  => "1/8",
                        SnapMode::Off     => "Off",
                    })
            )
            .child(div().flex_1())
            .child(
                div()
                    .text_size(px(10.0))
                    .text_color(Theme::TEXT_DIM)
                    .child("Zoom:")
            )
            .child(
                div()
                    .px(px(8.0))
                    .h(px(24.0))
                    .flex()
                    .items_center()
                    .rounded(px(Theme::RADIUS_SM))
                    .bg(Theme::BG_ELEVATED)
                    .border_1()
                    .border_color(Theme::BORDER)
                    .text_color(Theme::ACCENT)
                    .text_size(px(10.0))
                    .font(Font { family: "monospace".into(), ..Default::default() })
                    .child(format!("{:.2}×", zoom))
            )
    }

    fn render_track_header(
        &self,
        track: &Track,
        selected: bool,
        cx: &mut ViewContext<Self>,
    ) -> impl IntoElement {
        let track_id = track.id;
        let color = crate::theme::color_from_u32(track.color);
        let name = track.name.clone();
        let muted = track.muted;
        let solo = track.solo;
        let armed = track.armed;
        let volume = track.volume;
        let height = track.height;
        let state = self.state.clone();

        div()
            .h(px(height))
            .w_full()
            .flex()
            .flex_col()
            .justify_between()
            .px(px(8.0))
            .py(px(5.0))
            .bg(if selected { rgba(0x4db8ff0a) } else { Theme::BG_SURFACE })
            .border_b_1()
            .border_color(Theme::BORDER)
            .border_l(px(3.0))
            .border_color(if selected { color } else { Rgba::transparent_black() })
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, {
                let state = state.clone();
                move |_, cx| {
                    state.lock().unwrap().selected_track_id = Some(track_id);
                    cx.notify();
                }
            })
            .child(
                // Row 1: icon + name + arm + close
                div()
                    .flex()
                    .items_center()
                    .gap(px(5.0))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .child(match track.track_type {
                                TrackType::Instrument => "🎹",
                                TrackType::Audio      => "🎵",
                                TrackType::Bus        => "🔀",
                                TrackType::Master     => "⊟",
                            })
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(if selected { Theme::TEXT } else { rgba(0x94a3b8ff) })
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(name)
                    )
                    .child(
                        // ARM button
                        div()
                            .w(px(14.0))
                            .h(px(14.0))
                            .rounded_full()
                            .bg(if armed { Theme::ERROR } else { rgba(0x1a2a3aff) })
                            .border_1()
                            .border_color(if armed { Theme::ERROR } else { Theme::BORDER })
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, {
                                let state = state.clone();
                                move |_, cx| {
                                    let mut s = state.lock().unwrap();
                                    if let Some(t) = s.tracks.iter_mut().find(|t| t.id == track_id) {
                                        t.armed = !t.armed;
                                    }
                                    drop(s);
                                    cx.notify();
                                }
                            })
                    )
                    .child(
                        // Remove button
                        div()
                            .text_size(px(10.0))
                            .text_color(Theme::TEXT_MUTED)
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, {
                                let state = state.clone();
                                move |_, cx| {
                                    let mut s = state.lock().unwrap();
                                    s.tracks.retain(|t| t.id != track_id);
                                    drop(s);
                                    cx.notify();
                                }
                            })
                            .child("✕")
                    )
            )
            .child(
                // Row 2: M S volume
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child(
                        // Mute
                        div()
                            .px(px(5.0))
                            .h(px(16.0))
                            .flex()
                            .items_center()
                            .rounded(px(3.0))
                            .bg(if muted { rgba(0xef53501a) } else { Rgba::transparent_black() })
                            .border_1()
                            .border_color(if muted { rgba(0xef535050) } else { Theme::BORDER })
                            .text_color(if muted { Theme::ERROR } else { Theme::TEXT_DIM })
                            .text_size(px(9.0))
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, {
                                let state = state.clone();
                                move |_, cx| {
                                    let mut s = state.lock().unwrap();
                                    if let Some(t) = s.tracks.iter_mut().find(|t| t.id == track_id) {
                                        t.muted = !t.muted;
                                    }
                                    drop(s);
                                    cx.notify();
                                }
                            })
                            .child("M")
                    )
                    .child(
                        // Solo
                        div()
                            .px(px(5.0))
                            .h(px(16.0))
                            .flex()
                            .items_center()
                            .rounded(px(3.0))
                            .bg(if solo { rgba(0xffd54f1a) } else { Rgba::transparent_black() })
                            .border_1()
                            .border_color(if solo { rgba(0xffd54f50) } else { Theme::BORDER })
                            .text_color(if solo { Theme::WARNING } else { Theme::TEXT_DIM })
                            .text_size(px(9.0))
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, {
                                let state = state.clone();
                                move |_, cx| {
                                    let mut s = state.lock().unwrap();
                                    if let Some(t) = s.tracks.iter_mut().find(|t| t.id == track_id) {
                                        t.solo = !t.solo;
                                    }
                                    drop(s);
                                    cx.notify();
                                }
                            })
                            .child("S")
                    )
                    .child(
                        // Volume display
                        div()
                            .flex_1()
                            .h(px(10.0))
                            .rounded(px(2.0))
                            .bg(Theme::BG_ELEVATED)
                            .relative()
                            .child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .top_0()
                                    .bottom_0()
                                    .w(relative(volume))
                                    .rounded(px(2.0))
                                    .bg(color)
                            )
                    )
            )
    }

    fn render_clip(
        &self,
        clip: &Clip,
        track_h: f32,
        selected: bool,
    ) -> impl IntoElement {
        let beat_w = self.beat_w();
        let x = self.x_from_beat(clip.start_beat);
        let w = (clip.length_beats as f32 * beat_w).max(8.0);
        let color = crate::theme::color_from_u32(clip.color);
        let name = clip.name.clone();
        let notes = clip.notes.clone();
        let length = clip.length_beats;
        let clip_id = clip.id;

        div()
            .absolute()
            .left(px(x + 1.0))
            .w(px(w - 2.0))
            .top(px(4.0))
            .h(px(track_h - 8.0))
            .rounded(px(3.0))
            .bg(Rgba { r: color.r, g: color.g, b: color.b, a: if selected { 0.18 } else { 0.1 } })
            .border_1()
            .border_color(Rgba { a: if selected { 1.0 } else { 0.4 }, ..color })
            .overflow_hidden()
            .cursor_pointer()
            .child(
                // Clip name
                div()
                    .px(px(5.0))
                    .pt(px(2.0))
                    .text_size(px(9.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(color)
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(format!("{}{}", name, if notes.is_empty() { "" } else { " " }))
            )
            .when(!notes.is_empty(), |el| {
                // Mini note preview
                el.child(
                    canvas(
                        move |bounds, cx| {
                            let h = bounds.size.height.0;
                            let w = bounds.size.width.0;
                            for note in &notes {
                                let nx = (note.beat / length) as f32 * w;
                                let nw = (note.duration / length) as f32 * w;
                                let ny = h - ((note.pitch as f32 - 36.0) / 60.0) * h;
                                let mut path = Path::new(point(
                                    bounds.origin.x + px(nx),
                                    bounds.origin.y + px(ny),
                                ));
                                path.line_to(point(
                                    bounds.origin.x + px(nx + nw.max(1.0)),
                                    bounds.origin.y + px(ny),
                                ));
                                path.line_to(point(
                                    bounds.origin.x + px(nx + nw.max(1.0)),
                                    bounds.origin.y + px(ny + 2.0),
                                ));
                                path.line_to(point(
                                    bounds.origin.x + px(nx),
                                    bounds.origin.y + px(ny + 2.0),
                                ));
                                cx.paint_path(path, Rgba { a: 0.85, ..color });
                            }
                        },
                        |_, _| {},
                    )
                    .absolute()
                    .inset(px(14.0))
                    .bottom(px(2.0))
                )
            })
            // Resize handle (right edge)
            .child(
                div()
                    .absolute()
                    .right_0()
                    .top_0()
                    .bottom_0()
                    .w(px(8.0))
                    .cursor(CursorStyle::ResizeColumn)
            )
    }
}

impl Render for SongView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let state = self.state.lock().unwrap();
        let tracks = state.tracks.clone();
        let transport = state.transport.clone();
        let selected_track_id = state.selected_track_id;
        let selected_clip_id = state.selected_clip_id;
        drop(state);

        let beat_w = self.beat_w();
        let playhead_x = self.x_from_beat(transport.playhead_beat);
        let time_sig = transport.time_sig_num;

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(Theme::BG_CANVAS)
            .child(self.render_toolbar(cx))
            .child(
                // Main area
                div()
                    .flex_1()
                    .flex()
                    .overflow_hidden()
                    .child(
                        // Track headers
                        div()
                            .w(px(HEADER_W))
                            .flex_shrink_0()
                            .flex()
                            .flex_col()
                            .bg(Theme::BG_SURFACE)
                            .border_r_1()
                            .border_color(Theme::BORDER)
                            .child(
                                // Ruler spacer
                                div()
                                    .h(px(RULER_H))
                                    .border_b_1()
                                    .border_color(Theme::BORDER)
                            )
                            .children(
                                tracks.iter().map(|track| {
                                    let selected = selected_track_id == Some(track.id);
                                    self.render_track_header(track, selected, cx)
                                })
                            )
                    )
                    .child(
                        // Timeline
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .relative()
                            .child(
                                canvas(
                                    {
                                        let tracks = tracks.clone();
                                        let beat_w = beat_w;
                                        let scroll_x = self.scroll_x;
                                        let time_sig = time_sig;
                                        let playhead_x = playhead_x;
                                        let loop_enabled = transport.loop_enabled;
                                        let loop_start_x = self.x_from_beat(transport.loop_start);
                                        let loop_end_x = self.x_from_beat(transport.loop_end);
                                        let selected_clip_id = selected_clip_id;

                                        move |bounds, cx| {
                                            let w = bounds.size.width.0;
                                            let h = bounds.size.height.0;
                                            let ox = bounds.origin.x.0;
                                            let oy = bounds.origin.y.0;

                                            // ── Ruler ─────────────────────────────────────────
                                            let ruler_bg = Rect {
                                                origin: point(px(ox), px(oy)),
                                                size: size(px(w), px(RULER_H)),
                                            };
                                            cx.paint_quad(PaintQuad {
                                                bounds: ruler_bg,
                                                corner_radii: Default::default(),
                                                background: rgba(0x080e18ff),
                                                border_widths: Edges { bottom: px(1.0), ..Default::default() },
                                                border_color: rgba(0x0f1e2eff),
                                            });

                                            // Beat lines + bar numbers
                                            let first_beat = (scroll_x / beat_w).floor() as usize;
                                            let last_beat = ((scroll_x + w) / beat_w).ceil() as usize + 1;
                                            let last_beat = last_beat.min(TOTAL_BEATS);

                                            for i in first_beat..last_beat {
                                                let is_bar = i % time_sig as usize == 0;
                                                let x = ox + i as f32 * beat_w - scroll_x;
                                                let line_h = if is_bar { RULER_H } else { RULER_H * 0.45 };
                                                let line_color = if is_bar { rgba(0x1e3a5fff) } else { rgba(0x0a1520ff) };

                                                let mut path = Path::new(point(px(x), px(oy + RULER_H - line_h)));
                                                path.line_to(point(px(x), px(oy + RULER_H)));
                                                cx.paint_path(path, line_color);
                                            }

                                            // ── Loop region ───────────────────────────────────
                                            if loop_enabled {
                                                let lx = ox + loop_start_x;
                                                let lw = loop_end_x - loop_start_x;
                                                cx.paint_quad(PaintQuad {
                                                    bounds: Rect {
                                                        origin: point(px(lx), px(oy)),
                                                        size: size(px(lw), px(h)),
                                                    },
                                                    corner_radii: Default::default(),
                                                    background: rgba(0xffd54f0d),
                                                    border_widths: Edges {
                                                        left: px(2.0),
                                                        right: px(2.0),
                                                        ..Default::default()
                                                    },
                                                    border_color: rgba(0xffd54f80),
                                                });
                                            }

                                            // ── Track lanes ───────────────────────────────────
                                            let mut track_y = oy + RULER_H;
                                            for track in &tracks {
                                                let th = track.height;

                                                // Lane background
                                                cx.paint_quad(PaintQuad {
                                                    bounds: Rect {
                                                        origin: point(px(ox), px(track_y)),
                                                        size: size(px(w), px(th)),
                                                    },
                                                    corner_radii: Default::default(),
                                                    background: rgba(0x0b1420ff),
                                                    border_widths: Edges { bottom: px(1.0), ..Default::default() },
                                                    border_color: rgba(0x0f1e2eff),
                                                });

                                                // Beat grid lines
                                                for i in first_beat..last_beat {
                                                    let is_bar = i % time_sig as usize == 0;
                                                    let x = ox + i as f32 * beat_w - scroll_x;
                                                    let mut path = Path::new(point(px(x), px(track_y)));
                                                    path.line_to(point(px(x), px(track_y + th)));
                                                    cx.paint_path(path, if is_bar { rgba(0x0f1e2eff) } else { rgba(0x080e18ff) });
                                                }

                                                // Playhead line through lane
                                                let phx = ox + playhead_x;
                                                let mut ph_path = Path::new(point(px(phx), px(track_y)));
                                                ph_path.line_to(point(px(phx), px(track_y + th)));
                                                cx.paint_path(ph_path, rgba(0x4db8ff66));

                                                track_y += th;
                                            }

                                            // ── Playhead ──────────────────────────────────────
                                            let phx = ox + playhead_x;
                                            let mut ph = Path::new(point(px(phx), px(oy)));
                                            ph.line_to(point(px(phx), px(oy + h)));
                                            cx.paint_path(ph, rgba(0x4db8ffff));

                                            // Playhead triangle
                                            let mut tri = Path::new(point(px(phx - 5.0), px(oy)));
                                            tri.line_to(point(px(phx + 5.0), px(oy)));
                                            tri.line_to(point(px(phx), px(oy + 10.0)));
                                            tri.line_to(point(px(phx - 5.0), px(oy)));
                                            cx.paint_path(tri, rgba(0x4db8ffff));
                                        }
                                    },
                                    |_, _| {},
                                )
                                .absolute()
                                .inset_0()
                            )
                            // Clip overlays (rendered as divs on top of canvas)
                            .children({
                                let mut track_y = RULER_H;
                                let mut clip_els: Vec<AnyElement> = Vec::new();
                                for track in &tracks {
                                    let th = track.height;
                                    for clip in &track.clips {
                                        let selected = selected_clip_id == Some(clip.id);
                                        let el = div()
                                            .absolute()
                                            .top(px(track_y))
                                            .left_0()
                                            .right_0()
                                            .h(px(th))
                                            .child(self.render_clip(clip, th, selected))
                                            .into_any_element();
                                        clip_els.push(el);
                                    }
                                    track_y += th;
                                }
                                clip_els
                            })
                    )
                )
            )
    }
}
