// Aether Studio - Main Entry Point

mod project;
mod workspace;
mod theme;
mod widgets;
mod dsp_graph;

use project::ProjectConfig;
use theme::AetherTheme;
use widgets::welcome_screen::{WelcomeScreen, Message as WelcomeMessage};
use workspace::{Workspace, WorkspaceMode};
use dsp_graph::node_editor::{NodeEditor, Message as NodeEditorMessage};

use iced::widget::{column, container, text};
use iced::{Element, Length, Task};

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application("Aether Studio", AetherStudio::update, AetherStudio::view)
        .window_size((1600.0, 1000.0))
        .theme(|_| iced::Theme::Dark)
        .run()
}

#[derive(Debug)]
struct AetherStudio {
    workspace: Workspace,
    node_editor: NodeEditor,
}

#[derive(Debug, Clone)]
enum Message {
    Welcome(WelcomeMessage),
    NodeEditor(NodeEditorMessage),
    CloseProject,
    SwitchMode(WorkspaceMode),
}

impl Default for AetherStudio {
    fn default() -> Self {
        Self {
            workspace: Workspace::new(),
            node_editor: NodeEditor::new(),
        }
    }
}

impl AetherStudio {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Welcome(welcome_msg) => match welcome_msg {
                WelcomeMessage::CreateProject(project_type) => {
                    let config = ProjectConfig {
                        name: format!("New {} Project", project_type.name()),
                        project_type,
                        version: "0.1.0".to_string(),
                        author: String::new(),
                        description: String::new(),
                    };
                    self.workspace.create_project(config);
                }
                WelcomeMessage::OpenExisting => {
                    // TODO: Implement file dialog
                    tracing::info!("Open existing project");
                }
                WelcomeMessage::BrowseExamples => {
                    // TODO: Implement examples browser
                    tracing::info!("Browse examples");
                }
                WelcomeMessage::OpenDocumentation => {
                    // TODO: Open documentation in browser
                    tracing::info!("Open documentation");
                }
            },
            Message::NodeEditor(node_editor_msg) => {
                match &node_editor_msg {
                    NodeEditorMessage::BackToWelcome => {
                        self.workspace.close_project();
                    }
                    _ => {
                        self.node_editor.update(node_editor_msg);
                    }
                }
            }
            Message::CloseProject => {
                self.workspace.close_project();
            }
            Message::SwitchMode(mode) => {
                self.workspace.set_mode(mode);
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        match self.workspace.active_mode {
            WorkspaceMode::Welcome => WelcomeScreen::view().map(Message::Welcome),
            WorkspaceMode::DspGraph => self.view_dsp_graph(),
            WorkspaceMode::GuiDesigner => self.view_gui_designer(),
            WorkspaceMode::ProjectSettings => self.view_project_settings(),
        }
    }

    fn view_dsp_graph(&self) -> Element<Message> {
        self.node_editor.view().map(Message::NodeEditor)
    }

    fn view_gui_designer(&self) -> Element<Message> {
        let content = column![
            text("GUI Designer Mode").size(24),
            text("Drag-and-drop UI builder").size(14),
            text("Coming soon...").size(12),
        ]
        .spacing(16)
        .padding(32);

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

    fn view_project_settings(&self) -> Element<Message> {
        let content = column![
            text("Project Settings Mode").size(24),
            text("Export, testing, and configuration").size(14),
            text("Coming soon...").size(12),
        ]
        .spacing(16)
        .padding(32);

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
}
