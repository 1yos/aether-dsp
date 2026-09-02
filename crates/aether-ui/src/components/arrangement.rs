//! Arrangement view — canvas showing clips on a timeline.

use iced::widget::{canvas, container};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme as IcedTheme};

use crate::project::{Track, TrackId};
use crate::theme::Theme;

pub const BAR_WIDTH: f32 = 80.0;
pub const TRACK_HEIGHT: f32 = 72.0;
const HEADER_HEIGHT: f32 = 28.0;
const NUM_BARS: usize = 32;

#[derive(Debug, Clone)]
pub enum Message {
    ClickBar(TrackId, f32),
    OpenClip(TrackId, usize),
}

pub fn view<'a>(
    tracks: &'a [Track],
    selected_track: Option<TrackId>,
    playhead_bar: f32,
) -> Element<'a, Message> {
    container(
        canvas(ArrangementCanvas { tracks, selected_track, playhead_bar })
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(Theme::APP_BG)),
        ..Default::default()
    })
    .into()
}

struct ArrangementCanvas<'a> {
    tracks: &'a [Track],
    selected_track: Option<TrackId>,
    playhead_bar: f32,
}

impl<'a> canvas::Program<Message> for ArrangementCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &IcedTheme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Background
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), canvas::Fill::from(Theme::APP_BG));

        // Header background
        frame.fill_rectangle(
            Point::ORIGIN,
            Size::new(bounds.width, HEADER_HEIGHT),
            canvas::Fill::from(Theme::PANEL_BG),
        );

        // Bar grid lines + numbers
        for bar in 0..NUM_BARS {
            let x = bar as f32 * BAR_WIDTH;
            let mut path = canvas::path::Builder::new();
            path.move_to(Point::new(x, 0.0));
            path.line_to(Point::new(x, bounds.height));
            frame.stroke(
                &path.build(),
                canvas::Stroke::default().with_color(Theme::BORDER).with_width(1.0),
            );
            frame.fill_text(canvas::Text {
                content: format!("{}", bar + 1),
                position: Point::new(x + 4.0, 8.0),
                color: Theme::TEXT_DISABLED,
                size: iced::Pixels(11.0),
                ..Default::default()
            });
        }

        // Track rows
        for (row_idx, track) in self.tracks.iter().enumerate() {
            let y = HEADER_HEIGHT + row_idx as f32 * TRACK_HEIGHT;

            // Row background
            let row_bg = if self.selected_track == Some(track.id) {
                Color { r: 0.12, g: 0.12, b: 0.12, a: 1.0 }
            } else {
                Theme::APP_BG
            };
            frame.fill_rectangle(
                Point::new(0.0, y),
                Size::new(bounds.width, TRACK_HEIGHT),
                canvas::Fill::from(row_bg),
            );

            // Row divider
            let mut div = canvas::path::Builder::new();
            div.move_to(Point::new(0.0, y + TRACK_HEIGHT));
            div.line_to(Point::new(bounds.width, y + TRACK_HEIGHT));
            frame.stroke(
                &div.build(),
                canvas::Stroke::default().with_color(Theme::BORDER).with_width(1.0),
            );

            // Track color accent
            frame.fill_rectangle(
                Point::new(0.0, y),
                Size::new(3.0, TRACK_HEIGHT),
                canvas::Fill::from(track.color),
            );

            // Clips
            for clip in &track.clips {
                let cx = clip.start_bar * BAR_WIDTH;
                let cw = (clip.length_bars * BAR_WIDTH - 2.0).max(4.0);
                let clip_color = Color { a: 0.85, ..track.color };

                frame.fill_rectangle(
                    Point::new(cx + 3.0, y + 6.0),
                    Size::new(cw, TRACK_HEIGHT - 12.0),
                    canvas::Fill::from(clip_color),
                );
                frame.fill_text(canvas::Text {
                    content: clip.name.clone(),
                    position: Point::new(cx + 8.0, y + 12.0),
                    color: Color::BLACK,
                    size: iced::Pixels(11.0),
                    ..Default::default()
                });
            }
        }

        // Playhead
        if self.playhead_bar > 0.0 {
            let px = self.playhead_bar * BAR_WIDTH;
            let mut ph = canvas::path::Builder::new();
            ph.move_to(Point::new(px, 0.0));
            ph.line_to(Point::new(px, bounds.height));
            frame.stroke(
                &ph.build(),
                canvas::Stroke::default().with_color(Theme::ACCENT).with_width(2.0),
            );
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut (),
        event: canvas::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        if let canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(
            iced::mouse::Button::Left,
        )) = event
        {
            if let Some(pos) = cursor.position_in(bounds) {
                if pos.y > HEADER_HEIGHT {
                    let row = ((pos.y - HEADER_HEIGHT) / TRACK_HEIGHT) as usize;
                    if row < self.tracks.len() {
                        let track = &self.tracks[row];
                        let track_id = track.id;

                        // Check existing clips first
                        for (ci, clip) in track.clips.iter().enumerate() {
                            let cx = clip.start_bar * BAR_WIDTH;
                            let cw = clip.length_bars * BAR_WIDTH;
                            if pos.x >= cx + 3.0 && pos.x <= cx + cw {
                                return (
                                    canvas::event::Status::Captured,
                                    Some(Message::OpenClip(track_id, ci)),
                                );
                            }
                        }

                        // Empty area — create clip
                        let bar = (pos.x / BAR_WIDTH).floor();
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::ClickBar(track_id, bar)),
                        );
                    }
                }
            }
        }
        (canvas::event::Status::Ignored, None)
    }
}
