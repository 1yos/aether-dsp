//! One track row in the track list.

use iced::widget::{button, column, container, row, slider, text};
use iced::{Alignment, Color, Element, Length};

use crate::project::{Track, TrackId};
use crate::theme::Theme;

#[derive(Debug, Clone)]
pub enum Message {
    Select(TrackId),
    VolumeChanged(TrackId, f32),
    ToggleMute(TrackId),
    ToggleSolo(TrackId),
    Remove(TrackId),
}

pub fn view_strip(track: &Track, is_selected: bool) -> Element<'_, Message> {
    let id = track.id;
    let color = track.color;

    // Left color swatch
    let swatch = container(iced::widget::Space::new(4, Length::Fill))
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(color)),
            ..Default::default()
        })
        .width(4)
        .height(Length::Fill);

    let name = text(&track.name)
        .size(13)
        .style(|_| text::Style { color: Some(Theme::TEXT_PRIMARY) });

    let tuning_name = text(track.tuning.display_name())
        .size(10)
        .style(|_| text::Style { color: Some(Theme::TEXT_SECONDARY) });

    let vol = track.volume;
    let vol_slider = slider(0.0_f32..=1.0_f32, vol, move |v| {
        Message::VolumeChanged(id, v)
    })
    .step(0.01_f32)
    .width(90);

    let vol_label = text(format!("{:.0}%", vol * 100.0))
        .size(10)
        .font(iced::Font::MONOSPACE)
        .style(|_| text::Style { color: Some(Theme::TEXT_SECONDARY) });

    let muted = track.muted;
    let btn_mute = button(text("M").size(11).style(move |_| text::Style {
        color: Some(if muted { Theme::RED } else { Theme::TEXT_DISABLED }),
    }))
    .on_press(Message::ToggleMute(id))
    .padding([3, 6])
    .style(|_, _| button::Style {
        background: Some(iced::Background::Color(Theme::SURFACE)),
        border: iced::Border { color: Theme::BORDER, width: 1.0, radius: 3.0.into() },
        ..Default::default()
    });

    let soloed = track.soloed;
    let btn_solo = button(text("S").size(11).style(move |_| text::Style {
        color: Some(if soloed { Theme::ACCENT } else { Theme::TEXT_DISABLED }),
    }))
    .on_press(Message::ToggleSolo(id))
    .padding([3, 6])
    .style(|_, _| button::Style {
        background: Some(iced::Background::Color(Theme::SURFACE)),
        border: iced::Border { color: Theme::BORDER, width: 1.0, radius: 3.0.into() },
        ..Default::default()
    });

    let controls = row![
        vol_slider,
        iced::widget::Space::with_width(4),
        vol_label,
        iced::widget::Space::with_width(8),
        btn_mute,
        iced::widget::Space::with_width(4),
        btn_solo,
    ]
    .align_y(Alignment::Center);

    let info = column![name, tuning_name, controls].spacing(3);

    let inner = row![swatch, iced::widget::Space::with_width(8), info]
        .align_y(Alignment::Center);

    let bg_color = if is_selected { Theme::SURFACE } else { Theme::PANEL_BG };

    button(
        container(inner)
            .width(Length::Fill)
            .height(72)
            .padding([8, 12]),
    )
    .on_press(Message::Select(id))
    .style(move |_, status| button::Style {
        background: Some(iced::Background::Color(match status {
            button::Status::Hovered => Theme::SURFACE,
            _ => bg_color,
        })),
        border: iced::Border {
            color: if is_selected { Theme::ACCENT } else { Color::TRANSPARENT },
            width: if is_selected { 1.0 } else { 0.0 },
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}
