// Aether Studio - Professional Audio Development IDE

mod dsp_graph;
mod ide;
mod project;
mod theme;
mod widgets;
mod workspace;

use ide::code_editor::{CodeEditorView, Message as EditorMessage};
use ide::graph_view::{GraphView, Message as GraphMessage};
use ide::project_explorer::{Message as ExplorerMessage, ProjectExplorer};
use ide::terminal::{Message as TerminalMessage, Terminal};
use ide::toolbar::{Message as ToolbarMessage, Toolbar};
use ide::{code_generator, AetherIDE};
use project::{ProjectConfig, ProjectType};
use theme::AetherTheme;
use widgets::welcome_screen::{Message as WelcomeMessage, WelcomeScreen};
use workspace::{Workspace, WorkspaceMode};

use iced::widget::{column, container, row};
use iced::{Element, Length, Task};
use std::path::PathBuf;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application("Aether Studio", AetherStudio::update, AetherStudio::view)
        .window_size((1800.0, 1000.0))
        .theme(|_| iced::Theme::Dark)
        .run()
}

#[derive(Debug, Default)]
struct AetherStudio {
    workspace: Workspace,
    ide: AetherIDE,
    project_explorer: ProjectExplorer,
    code_editor: CodeEditorView,
    graph_view: GraphView,
    terminal: Terminal,
    toolbar: Toolbar,
}

#[derive(Debug, Clone)]
enum Message {
    Welcome(WelcomeMessage),
    Explorer(ExplorerMessage),
    Editor(EditorMessage),
    Graph(GraphMessage),
    Terminal(TerminalMessage),
    Toolbar(ToolbarMessage),
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

                    // Create project in IDE
                    let project_path = PathBuf::from(format!("./projects/{}", config.name));
                    self.ide
                        .create_project(config.clone(), project_path.clone());
                    self.workspace.create_project(config);
                    self.project_explorer.set_root(project_path);

                    // Generate initial code
                    self.generate_initial_project_code(project_type);

                    self.terminal
                        .append_output(&format!("✓ Created new {} project", project_type.name()));
                }
                WelcomeMessage::OpenExisting => {
                    self.terminal
                        .append_output("Open existing project - TODO: Implement file dialog");
                }
                WelcomeMessage::BrowseExamples => {
                    self.terminal
                        .append_output("Browse examples - TODO: Implement examples browser");
                }
                WelcomeMessage::OpenDocumentation => {
                    self.terminal.append_output("Opening documentation...");
                }
            },
            Message::Toolbar(toolbar_msg) => match toolbar_msg {
                ToolbarMessage::Build => {
                    self.toolbar.set_building(true);
                    self.terminal.append_output("$ cargo build --release");
                    self.terminal
                        .append_output("   Compiling aetherdsp-core v0.1.0");
                    self.terminal.append_output("   Compiling my-plugin v0.1.0");
                    self.terminal
                        .append_output("   Finished release [optimized] target(s) in 2.34s");
                    self.toolbar.set_building(false);
                }
                ToolbarMessage::Run => {
                    self.terminal.append_output("$ cargo run --release");
                    self.terminal
                        .append_output("   Running target/release/my-plugin");
                }
                ToolbarMessage::Debug => {
                    self.terminal.append_output("Starting debugger...");
                }
                ToolbarMessage::Export => {
                    self.terminal.append_output("Exporting plugin...");
                }
                ToolbarMessage::Settings => {
                    self.terminal.append_output("Opening settings...");
                }
            },
            Message::Explorer(_) => {
                // Handle file explorer interactions
            }
            Message::Editor(_) => {
                // Handle code editor interactions
            }
            Message::Graph(_) => {
                // Handle graph view interactions
            }
            Message::Terminal(_) => {
                // Handle terminal interactions
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        match self.workspace.active_mode {
            WorkspaceMode::Welcome => WelcomeScreen::view().map(Message::Welcome),
            WorkspaceMode::DspGraph => self.view_ide(),
            WorkspaceMode::GuiDesigner => self.view_gui_designer(),
            WorkspaceMode::ProjectSettings => self.view_project_settings(),
        }
    }

    fn view_ide(&self) -> Element<'_, Message> {
        let toolbar = self.toolbar.view().map(Message::Toolbar);

        let explorer = self.project_explorer.view().map(Message::Explorer);

        let editor = if let Some(file) = self.ide.get_active_file() {
            self.code_editor
                .view(
                    file.path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("untitled"),
                )
                .map(Message::Editor)
        } else {
            self.view_empty_editor()
        };

        let graph = self.graph_view.view().map(Message::Graph);

        let main_area = row![explorer, editor, graph,].spacing(0);

        let terminal = self.terminal.view().map(Message::Terminal);

        let content = column![toolbar, main_area, terminal,].spacing(0);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(AetherTheme::APP_BACKGROUND.into()),
                ..Default::default()
            })
            .into()
    }

    fn view_empty_editor(&self) -> Element<'_, Message> {
        use iced::widget::text;

        let content = column![
            text("No file open").size(16).style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_DISABLED),
            }),
            text("Select a file from the project explorer")
                .size(12)
                .style(|_theme| text::Style {
                    color: Some(AetherTheme::TEXT_DISABLED),
                }),
        ]
        .spacing(8)
        .padding(32);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(AetherTheme::CANVAS_BACKGROUND.into()),
                ..Default::default()
            })
            .into()
    }

    fn view_gui_designer(&self) -> Element<'_, Message> {
        use iced::widget::text;

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

    fn view_project_settings(&self) -> Element<'_, Message> {
        use iced::widget::text;

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

    fn generate_initial_project_code(&mut self, project_type: ProjectType) {
        // Generate example nodes based on project type
        let nodes = match project_type {
            ProjectType::Plugin => vec![
                (
                    "oscillator".to_string(),
                    dsp_graph::NodeType::Oscillator {
                        waveform: dsp_graph::Waveform::Sine,
                    },
                ),
                ("filter".to_string(), dsp_graph::NodeType::LowPass),
                ("gain".to_string(), dsp_graph::NodeType::Gain),
            ],
            ProjectType::Daw => vec![
                ("mixer".to_string(), dsp_graph::NodeType::Mixer),
                ("compressor".to_string(), dsp_graph::NodeType::Compressor),
                ("reverb".to_string(), dsp_graph::NodeType::Reverb),
            ],
            _ => vec![("gain".to_string(), dsp_graph::NodeType::Gain)],
        };

        // Generate code for first node as example
        if let Some((node_name, node_type)) = nodes.first() {
            let code = code_generator::generate_node_code(node_type, node_name);
            let file_path = PathBuf::from(format!("src/nodes/{}.rs", node_name));
            self.ide.open_file(file_path, code);
            self.code_editor.set_content(
                self.ide.get_active_file().unwrap().content.clone(),
                "rust".to_string(),
            );
        }

        self.terminal
            .append_output(&format!("✓ Generated {} example nodes", nodes.len()));
    }
}
