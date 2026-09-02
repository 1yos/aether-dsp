//! Step sequencer — 16-step grid for drum tracks.

use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Color, Element, Length};

use crate::project::DrumPattern;
use crate::theme::Theme;

#[derive(Debug, Clone)]
pub enum Message {
    Close,
    ToggleStep(usize, usize),
}

pub fn view<'a>(pattern: &'a DrumPattern, title: &'a str) -> Element<'a, Message> {
    let title_bar = container(
        row![
            text(title).size(14).style(|_| text::Style {
                color: Some(Theme::TEXT_PRIMARY),
            }),
            iced::widget::Space::with_width(Length::Fill),
            button(
                text("✕ Close").size(13).style(|_| text::Style {
                    color: Some(Theme::TEXT_SECONDARY),
                })
            )
            .on_press(Message::Close)
            .padding([4, 12])
            .style(|_, _| button::Style {
                background: Some(iced::Background::Color(Theme::SURFACE)),
                border: iced::Border { color: Theme::BORDER, width: 1.0, radius: 4.0.into() },
                ..Default::default()
            }),
        ]
        .align_y(Alignment::Center)
        .padding([0, 16]),
    )
    .height(40)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(Theme::PANEL_BG)),
        border: iced::Border { color: Theme::BORDER, width: 1.0, ..Default::default() },
        ..Default::default()
    });

    let steps = pattern.steps as usize;

    // Beat numbers header
    let beat_labels = row(
        (0..steps)
            .map(|i| {
                let is_beat = i % 4 == 0;
                container(
                    text(if is_beat {
                        format!("{}", i / 4 + 1)
                    } else {
                        String::new()
                    })
                    .size(10)
                    .style(|_| text::Style { color: Some(Theme::TEXT_DISABLED) }),
                )
                .width(36)
                .center_x(36)
                .into()
            })
            .collect::<Vec<_>>(),
    )
    .spacing(2);

    let row_label_width = 72u16;
    let spacer: Element<'a, Message> = iced::widget::Space::with_width(row_label_width).into();
    let header_row: Element<'a, Message> = row![spacer, beat_labels].spacing(4).into();

    let mut rows_col = column![header_row].spacing(6).padding([0, 16]);

    for (row_idx, drum_row) in pattern.rows.iter().enumerate() {
        let label = container(
            text(&drum_row.name).size(12).style(|_| text::Style {
                color: Some(Theme::TEXT_SECONDARY),
            }),
        )
        .width(row_label_width)
        .align_x(iced::Alignment::End)
        .padding([0, 8]);

        let step_buttons: Vec<Element<'a, Message>> = drum_row
            .steps
            .iter()
            .enumerate()
            .map(|(step_idx, &active)| {
                let is_beat_start = step_idx % 4 == 0;
                let bg = if active {
                    Theme::ACCENT
                } else if is_beat_start {
                    Color { r: 0.15, g: 0.15, b: 0.15, a: 1.0 }
                } else {
                    Theme::SURFACE
                };

                button(iced::widget::Space::new(32, 28))
                    .on_press(Message::ToggleStep(row_idx, step_idx))
                    .style(move |_, status| button::Style {
                        background: Some(iced::Background::Color(match status {
                            button::Status::Hovered => Color {
                                a: 0.7,
                                ..bg
                            },
                            _ => bg,
                        })),
                        border: iced::Border {
                            color: Theme::BORDER,
                            width: 1.0,
                            radius: 3.0.into(),
                        },
                        ..Default::default()
                    })
                    .into()
            })
            .collect();

        let step_row = row![label, row(step_buttons).spacing(2)]
            .align_y(Alignment::Center)
            .spacing(4);

        rows_col = rows_col.push(step_row);
    }

    let help = text("Click steps to toggle on/off")
        .size(11)
        .style(|_| text::Style { color: Some(Theme::TEXT_DISABLED) });

    let footer = container(help).padding([6, 16]).style(|_| container::Style {
        background: Some(iced::Background::Color(Theme::PANEL_BG)),
        ..Default::default()
    });

    column![
        title_bar,
        iced::widget::Space::with_height(16),
        rows_col,
        iced::widget::Space::with_height(Length::Fill),
        footer,
    ]
    .into()
}
