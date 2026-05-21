// Toolbar - Build, Run, Debug, Export buttons

use crate::theme::{AetherTheme, Spacing};
use iced::widget::{button, container, row, text};
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum Message {
    Build,
    Run,
    Debug,
    Export,
    Settings,
}

#[derive(Debug)]
pub struct Toolbar {
    is_building: bool,
}

impl Toolbar {
    pub fn new() -> Self {
        Self { is_building: false }
    }

    pub fn set_building(&mut self, building: bool) {
        self.is_building = building;
    }

    pub fn view(&self) -> Element<'_, Message> {
        let build_btn = button(
            row![text("▶").size(14), text("Build").size(14)]
                .spacing(Spacing::XS)
                .align_y(iced::Alignment::Center),
        )
        .on_press(Message::Build)
        .padding(Spacing::SM)
        .style(if self.is_building {
            button::secondary
        } else {
            button::primary
        });

        let run_btn = button(
            row![text("▶").size(14), text("Run").size(14)]
                .spacing(Spacing::XS)
                .align_y(iced::Alignment::Center),
        )
        .on_press(Message::Run)
        .padding(Spacing::SM)
        .style(button::success);

        let debug_btn = button(
            row![text("🐛").size(14), text("Debug").size(14)]
                .spacing(Spacing::XS)
                .align_y(iced::Alignment::Center),
        )
        .on_press(Message::Debug)
        .padding(Spacing::SM)
        .style(button::secondary);

        let export_btn = button(
            row![text("📦").size(14), text("Export").size(14)]
                .spacing(Spacing::XS)
                .align_y(iced::Alignment::Center),
        )
        .on_press(Message::Export)
        .padding(Spacing::SM)
        .style(button::secondary);

        let settings_btn = button(text("⚙️").size(14))
            .on_press(Message::Settings)
            .padding(Spacing::SM)
            .style(button::text);

        let toolbar_content = row![build_btn, run_btn, debug_btn, export_btn, settings_btn,]
            .spacing(Spacing::SM)
            .padding(Spacing::SM)
            .align_y(iced::Alignment::Center);

        container(toolbar_content)
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(AetherTheme::PANEL_BACKGROUND.into()),
                border: iced::Border {
                    color: AetherTheme::NODE_BACKGROUND,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into()
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}
