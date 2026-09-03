//! Transport bar — play, stop, BPM, elapsed time.

use iced::widget::{button, container, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::theme::Theme;

#[derive(Debug, Clone)]
pub enum Message {
    Play,
    Stop,
    BpmChanged(String),
}

pub struct Transport {
    pub is_playing: bool,
    pub bpm: f32,
    pub bpm_str: String,
    pub elapsed_str: String,
}

impl Default for Transport {
    fn default() -> Self {
        Self {
            is_playing: false,
            bpm: 120.0,
            bpm_str: "120".to_string(),
            elapsed_str: "00:00".to_string(),
        }
    }
}

impl Transport {
    pub fn view(&self) -> Element<'_, Message> {
        let (play_label, play_color) = if self.is_playing {
            ("■  Stop", Theme::RED)
        } else {
            ("▶  Play", Theme::GREEN)
        };

        let btn_play = button(
            text(play_label)
                .size(14)
                .style(move |_| text::Style { color: Some(play_color) }),
        )
        .on_press(if self.is_playing { Message::Stop } else { Message::Play })
        .padding([8, 18])
        .style(|_, _| button::Style {
            background: Some(iced::Background::Color(Theme::SURFACE)),
            border: iced::Border { color: Theme::BORDER, width: 1.0, radius: 4.0.into() },
            ..Default::default()
        });

        let bpm_label = text("BPM")
            .size(12)
            .style(|_| text::Style { color: Some(Theme::TEXT_SECONDARY) });

        let bpm_input = text_input("120", &self.bpm_str)
            .on_input(Message::BpmChanged)
            .padding([6, 10])
            .width(60)
            .style(|_, _| text_input::Style {
                background: iced::Background::Color(Theme::SURFACE),
                border: iced::Border { color: Theme::BORDER, width: 1.0, radius: 4.0.into() },
                icon: Theme::TEXT_SECONDARY,
                placeholder: Theme::TEXT_DISABLED,
                value: Theme::TEXT_PRIMARY,
                selection: Theme::ACCENT,
            });

        // Playing indicator dot
        let indicator = if self.is_playing {
            text("●").size(12).style(|_| text::Style { color: Some(Theme::RED) })
        } else {
            text("●").size(12).style(|_| text::Style { color: Some(Theme::TEXT_DISABLED) })
        };

        let time = text(&self.elapsed_str)
            .size(13)
            .font(iced::Font::MONOSPACE)
            .style(|_| text::Style { color: Some(Theme::TEXT_SECONDARY) });

        let space_hint = text("[ Space ]")
            .size(10)
            .style(|_| text::Style { color: Some(Theme::TEXT_DISABLED) });

        let bar = row![
            btn_play,
            iced::widget::Space::with_width(8),
            space_hint,
            iced::widget::Space::with_width(24),
            bpm_label,
            iced::widget::Space::with_width(6),
            bpm_input,
            iced::widget::Space::with_width(Length::Fill),
            indicator,
            iced::widget::Space::with_width(8),
            time,
            iced::widget::Space::with_width(16),
        ]
        .align_y(Alignment::Center)
        .padding([0, 16]);

        container(bar)
            .width(Length::Fill)
            .height(48)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Theme::PANEL_BG)),
                border: iced::Border { color: Theme::BORDER, width: 1.0, ..Default::default() },
                ..Default::default()
            })
            .into()
    }
}
