//! Aether Studio — World Music Production
//! Entry point: starts the Iced application.

mod plugin_gui;
mod project;
mod theme;

use iced::{Element, Task};

fn main() -> iced::Result {
    iced::application("Aether Studio", AetherStudio::update, AetherStudio::view)
        .window_size((1440.0, 900.0))
        .theme(|_| iced::Theme::Dark)
        .run()
}

#[derive(Debug, Default)]
struct AetherStudio;

#[derive(Debug, Clone)]
enum Message {}

impl AetherStudio {
    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        use iced::widget::{center, text};
        center(
            text("Aether Studio")
                .size(32)
                .style(|_theme| iced::widget::text::Style {
                    color: Some(theme::Theme::ACCENT),
                }),
        )
        .into()
    }
}
