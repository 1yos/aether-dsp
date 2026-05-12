// Terminal - Build output and command execution

use crate::theme::{AetherTheme, Spacing};
use iced::widget::{column, container, scrollable, text};
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum Message {
    // Terminal interactions
}

#[derive(Debug)]
pub struct Terminal {
    output: String,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            output: String::from("Terminal ready.\n"),
        }
    }

    pub fn append_output(&mut self, text: &str) {
        self.output.push_str(text);
        self.output.push('\n');
    }

    pub fn clear(&mut self) {
        self.output.clear();
    }

    pub fn view(&self) -> Element<'_, Message> {
        let title = text("TERMINAL")
            .size(12)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_SECONDARY),
            });

        let output_display = scrollable(
            text(&self.output)
                .size(12)
                .font(iced::Font::MONOSPACE)
                .style(|_theme| text::Style {
                    color: Some(AetherTheme::TEXT_PRIMARY),
                })
        )
        .height(Length::Fill);

        let content = column![title, output_display]
            .spacing(Spacing::SM)
            .padding(Spacing::MD);

        container(content)
            .width(Length::Fill)
            .height(200)
            .style(|_theme| container::Style {
                background: Some(AetherTheme::NODE_BACKGROUND.into()),
                border: iced::Border {
                    color: AetherTheme::CANVAS_BACKGROUND,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into()
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}
