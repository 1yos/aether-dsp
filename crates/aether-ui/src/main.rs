//! Aether Studio — World Music Production
//! Entry point: starts the audio engine, then launches the Iced UI.

mod engine;
mod plugin_gui;
mod project;
mod theme;

use engine::Engine;
use iced::{Element, Task};

fn main() -> iced::Result {
    iced::application("Aether Studio", AetherStudio::update, AetherStudio::view)
        .window_size((1440.0, 900.0))
        .theme(|_| iced::Theme::Dark)
        .run_with(AetherStudio::new)
}

struct AetherStudio {
    engine: Option<Engine>,
    engine_status: String,
}

#[derive(Debug, Clone)]
enum Message {}

impl AetherStudio {
    fn new() -> (Self, Task<Message>) {
        // Start the audio engine before the first frame renders.
        let (engine, status) = match Engine::start() {
            Ok(e) => {
                let sr = e.sample_rate();
                (Some(e), format!("Audio engine running at {sr} Hz"))
            }
            Err(err) => (
                None,
                format!("Audio engine failed to start: {err}"),
            ),
        };

        (
            Self { engine, engine_status: status },
            Task::none(),
        )
    }

    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        use iced::widget::{center, column, text};

        let title = text("Aether Studio")
            .size(40)
            .style(|_theme| iced::widget::text::Style {
                color: Some(theme::Theme::ACCENT),
            });

        let status = text(&self.engine_status)
            .size(14)
            .style(|_theme| iced::widget::text::Style {
                color: Some(if self.engine.is_some() {
                    theme::Theme::GREEN
                } else {
                    theme::Theme::RED
                }),
            });

        center(
            column![title, status].spacing(12).align_x(iced::Alignment::Center),
        )
        .into()
    }
}
