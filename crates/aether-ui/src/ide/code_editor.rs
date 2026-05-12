// Code Editor - Syntax-highlighted code editing

use crate::theme::{AetherTheme, Spacing};
use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum Message {
    CodeChanged(String),
}

#[derive(Debug)]
pub struct CodeEditorView {
    content: String,
    language: String,
}

impl CodeEditorView {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            language: "rust".to_string(),
        }
    }

    pub fn set_content(&mut self, content: String, language: String) {
        self.content = content;
        self.language = language;
    }

    pub fn view<'a>(&'a self, file_name: &'a str) -> Element<'a, Message> {
        let header = row![
            text(file_name)
                .size(14)
                .style(|_theme| text::Style {
                    color: Some(AetherTheme::TEXT_PRIMARY),
                }),
            text(format!(" • {}", self.language))
                .size(12)
                .style(|_theme| text::Style {
                    color: Some(AetherTheme::TEXT_SECONDARY),
                }),
        ]
        .spacing(Spacing::SM)
        .padding(Spacing::SM);

        // Simple text display (TODO: Add syntax highlighting)
        let code_display = scrollable(
            text(&self.content)
                .size(13)
                .font(iced::Font::MONOSPACE)
                .style(|_theme| text::Style {
                    color: Some(AetherTheme::TEXT_PRIMARY),
                })
        )
        .height(Length::Fill);

        let content = column![header, code_display]
            .spacing(0);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(AetherTheme::CANVAS_BACKGROUND.into()),
                ..Default::default()
            })
            .into()
    }
}

impl Default for CodeEditorView {
    fn default() -> Self {
        Self::new()
    }
}
