// Graph View - Visual representation of DSP graph (synchronized with code)

use crate::theme::{AetherTheme, Spacing};
use iced::widget::{column, container, text};
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum Message {
    // Graph interactions
}

#[derive(Debug)]
pub struct GraphView {
    // Graph state
}

impl GraphView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view(&self) -> Element<'_, Message> {
        let title = text("GRAPH VIEW")
            .size(12)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_SECONDARY),
            });

        let info = text("Visual representation of DSP graph\n(Synchronized with code)")
            .size(12)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_DISABLED),
            });

        let content = column![title, info]
            .spacing(Spacing::MD)
            .padding(Spacing::MD);

        container(content)
            .width(300)
            .height(Length::Fill)
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

impl Default for GraphView {
    fn default() -> Self {
        Self::new()
    }
}
