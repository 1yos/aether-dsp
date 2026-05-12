// Welcome screen for project selection

use crate::project::ProjectType;
use crate::theme::{AetherTheme, Spacing};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone)]
pub enum Message {
    CreateProject(ProjectType),
    OpenExisting,
    BrowseExamples,
    OpenDocumentation,
}

pub struct WelcomeScreen;

impl WelcomeScreen {
    pub fn view() -> Element<'static, Message> {
        let title = text("AETHER STUDIO")
            .size(32)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_PRIMARY),
            });

        let subtitle = text("What would you like to build today?")
            .size(18)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_SECONDARY),
            });

        let project_types = row![
            Self::project_card(ProjectType::Plugin),
            Self::project_card(ProjectType::Daw),
        ]
        .spacing(Spacing::LG);

        let project_types_2 = row![
            Self::project_card(ProjectType::NodeLibrary),
            Self::project_card(ProjectType::Utility),
        ]
        .spacing(Spacing::LG);

        let recent_projects = column![
            text("Recent Projects:")
                .size(14)
                .style(|_theme| text::Style {
                    color: Some(AetherTheme::TEXT_SECONDARY),
                }),
            text("• No recent projects")
                .size(12)
                .style(|_theme| text::Style {
                    color: Some(AetherTheme::TEXT_DISABLED),
                }),
        ]
        .spacing(Spacing::SM);

        let actions = row![
            button(text("Open Existing")).on_press(Message::OpenExisting),
            button(text("Browse Examples")).on_press(Message::BrowseExamples),
            button(text("Documentation")).on_press(Message::OpenDocumentation),
        ]
        .spacing(Spacing::MD);

        let content = column![
            title,
            subtitle,
            project_types,
            project_types_2,
            recent_projects,
            actions,
        ]
        .spacing(Spacing::XL)
        .padding(Spacing::XXL)
        .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(AetherTheme::APP_BACKGROUND.into()),
                ..Default::default()
            })
            .into()
    }

    fn project_card(project_type: ProjectType) -> Element<'static, Message> {
        let icon_str = project_type.icon().to_string();
        let name_str = project_type.name().to_string();
        let desc_str = project_type.description().to_string();
        let complexity_str = format!("Complexity: {}", project_type.complexity());
        let time_str = format!("Time to market: {}", project_type.time_to_market());

        let icon = text(icon_str).size(48);

        let name = text(name_str)
            .size(16)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_PRIMARY),
            });

        let description = text(desc_str)
            .size(12)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_SECONDARY),
            });

        let complexity = text(complexity_str)
            .size(10)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_DISABLED),
            });

        let time = text(time_str)
            .size(10)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_DISABLED),
            });

        let create_button = button(text("New Project"))
            .on_press(Message::CreateProject(project_type))
            .padding(Spacing::SM);

        let card_content = column![icon, name, description, complexity, time, create_button]
            .spacing(Spacing::SM)
            .padding(Spacing::LG)
            .align_x(Alignment::Center);

        container(card_content)
            .width(280)
            .height(240)
            .style(|_theme| container::Style {
                background: Some(AetherTheme::PANEL_BACKGROUND.into()),
                border: iced::Border {
                    color: AetherTheme::NODE_BACKGROUND,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .into()
    }
}
