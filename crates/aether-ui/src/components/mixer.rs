//! Mixer bar — collapsible channel strips at the bottom of the screen.

use iced::widget::{button, column, container, row, slider, text};
use iced::{Alignment, Element, Length};

use crate::project::{Track, TrackId};
use crate::theme::Theme;

#[derive(Debug, Clone)]
pub enum Message {
    ToggleExpand,
    VolumeChanged(TrackId, f32),
    ToggleMute(TrackId),
}

#[derive(Debug, Default)]
pub struct Mixer {
    pub expanded: bool,
}

impl Mixer {
    pub fn view<'a>(&'a self, tracks: &'a [Track]) -> Element<'a, Message> {
        let toggle_label = if self.expanded { "▼  MIXER" } else { "▲  MIXER" };

        let toggle_btn = button(
            text(toggle_label).size(11).style(|_| text::Style {
                color: Some(Theme::TEXT_SECONDARY),
            }),
        )
        .on_press(Message::ToggleExpand)
        .padding([6, 16])
        .style(|_, _| button::Style {
            background: None,
            ..Default::default()
        });

        if !self.expanded {
            return container(
                row![toggle_btn, iced::widget::Space::with_width(Length::Fill)]
            )
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Theme::PANEL_BG)),
                border: iced::Border { color: Theme::BORDER, width: 1.0, ..Default::default() },
                ..Default::default()
            })
            .height(32)
            .width(Length::Fill)
            .into();
        }

        let header = container(
            row![toggle_btn, iced::widget::Space::with_width(Length::Fill)],
        )
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Theme::PANEL_BG)),
            border: iced::Border { color: Theme::BORDER, width: 1.0, ..Default::default() },
            ..Default::default()
        })
        .height(32)
        .width(Length::Fill);

        let mut strips: Vec<Element<'a, Message>> = tracks
            .iter()
            .map(|track| {
                let id = track.id;
                let vol = track.volume;
                let muted = track.muted;
                let color = track.color;

                let swatch = container(iced::widget::Space::new(Length::Fill, 4))
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(color)),
                        ..Default::default()
                    })
                    .width(Length::Fill)
                    .height(4);

                let name = text(&track.name).size(11).style(|_| text::Style {
                    color: Some(Theme::TEXT_SECONDARY),
                });

                let fader = slider(0.0_f32..=1.0_f32, vol, move |v| {
                    Message::VolumeChanged(id, v)
                })
                .step(0.01_f32)
                .width(Length::Fill);

                let vol_text = text(format!("{:.0}", vol * 100.0))
                    .size(10)
                    .font(iced::Font::MONOSPACE)
                    .style(|_| text::Style { color: Some(Theme::TEXT_DISABLED) });

                let mute_col = if muted { Theme::RED } else { Theme::TEXT_DISABLED };
                let btn_m =
                    button(text("M").size(10).style(move |_| text::Style {
                        color: Some(mute_col),
                    }))
                    .on_press(Message::ToggleMute(id))
                    .padding([2, 5])
                    .style(|_, _| button::Style {
                        background: Some(iced::Background::Color(Theme::SURFACE)),
                        border: iced::Border {
                            color: Theme::BORDER,
                            width: 1.0,
                            radius: 2.0.into(),
                        },
                        ..Default::default()
                    });

                container(
                    column![
                        swatch,
                        iced::widget::Space::with_height(6),
                        name,
                        iced::widget::Space::with_height(6),
                        fader,
                        row![
                            vol_text,
                            iced::widget::Space::with_width(Length::Fill),
                            btn_m,
                        ]
                        .align_y(Alignment::Center),
                    ]
                    .spacing(3)
                    .padding([8, 10]),
                )
                .width(80)
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(Theme::PANEL_BG)),
                    border: iced::Border {
                        color: Theme::BORDER,
                        width: 1.0,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
            })
            .collect();

        // Master strip
        strips.push(
            container(
                column![
                    text("MASTER").size(10).style(|_| text::Style {
                        color: Some(Theme::ACCENT),
                    }),
                    iced::widget::Space::with_height(8),
                    slider(0.0_f32..=1.0_f32, 0.85_f32, |_v| {
                        Message::ToggleMute(TrackId(u32::MAX))
                    })
                    .step(0.01_f32)
                    .width(Length::Fill),
                    text("85")
                        .size(10)
                        .font(iced::Font::MONOSPACE)
                        .style(|_| text::Style { color: Some(Theme::TEXT_DISABLED) }),
                ]
                .spacing(4)
                .padding([8, 10]),
            )
            .width(80)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Theme::PANEL_BG)),
                border: iced::Border { color: Theme::ACCENT, width: 1.0, ..Default::default() },
                ..Default::default()
            })
            .into(),
        );

        let strips_scroll = iced::widget::scrollable(row(strips).spacing(4).padding([8, 16]))
            .direction(iced::widget::scrollable::Direction::Horizontal(
                iced::widget::scrollable::Scrollbar::default(),
            ));

        container(
            column![
                header,
                container(strips_scroll)
                    .height(120)
                    .width(Length::Fill)
                    .style(|_| container::Style {
                        background: Some(iced::Background::Color(Theme::APP_BG)),
                        ..Default::default()
                    }),
            ],
        )
        .width(Length::Fill)
        .into()
    }
}
