// Node editor - main view combining canvas, library, and inspector

use super::{GraphCanvas, NodeLibrary, Inspector, DspGraphState};
use crate::theme::{AetherTheme, Spacing};
use iced::widget::{button, container, row, text};
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum Message {
    Canvas(super::canvas::CanvasMessage),
    Library(super::node_library::Message),
    Inspector(super::inspector::Message),
    BackToWelcome,
}

#[derive(Debug)]
pub struct NodeEditor {
    canvas: GraphCanvas,
    library: NodeLibrary,
    inspector: Inspector,
}

impl NodeEditor {
    pub fn new() -> Self {
        Self {
            canvas: GraphCanvas::new(),
            library: NodeLibrary::new(),
            inspector: Inspector::new(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let toolbar = self.view_toolbar();
        
        let library_panel = self.library.view().map(Message::Library);
        
        let canvas_view = self.canvas.view().map(Message::Canvas);
        
        let selected_node = self.canvas.state().selected_node
            .and_then(|id| self.canvas.state().nodes.get(&id));
        let inspector_panel = self.inspector.view(selected_node).map(Message::Inspector);

        let main_area = row![
            library_panel,
            canvas_view,
            inspector_panel,
        ]
        .spacing(0);

        let content = iced::widget::column![toolbar, main_area]
            .spacing(0);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(AetherTheme::APP_BACKGROUND.into()),
                ..Default::default()
            })
            .into()
    }

    fn view_toolbar(&self) -> Element<'_, Message> {
        let back_button = button(text("← Back to Welcome").size(14))
            .on_press(Message::BackToWelcome)
            .padding(Spacing::SM)
            .style(button::secondary);

        let title = text("DSP Graph Editor")
            .size(16)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_PRIMARY),
            });

        let node_count = text(format!("{} nodes", self.canvas.state().nodes.len()))
            .size(12)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_SECONDARY),
            });

        let toolbar_content = row![back_button, title, node_count]
            .spacing(Spacing::MD)
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

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Canvas(canvas_msg) => {
                // Handle canvas interactions
                match canvas_msg {
                    super::canvas::CanvasMessage::NodeClicked(node_id) => {
                        self.canvas.state_mut().selected_node = Some(node_id);
                        self.inspector.set_selected_node(Some(node_id));
                        // Load node parameters into inspector
                        if let Some(node) = self.canvas.state().nodes.get(&node_id) {
                            for (name, value) in &node.parameters {
                                self.inspector.update_parameter(name.clone(), *value);
                            }
                        }
                    }
                    super::canvas::CanvasMessage::AddNode(_position) => {
                        // This would be triggered by dragging from library
                    }
                    _ => {}
                }
            }
            Message::Library(library_msg) => {
                // Update library state first
                self.library.update(library_msg.clone());
                
                // Handle library interactions
                if let super::node_library::Message::NodeSelected(node_type) = library_msg {
                    // Add node to canvas at center
                    let position = iced::Point::new(400.0, 300.0);
                    let node_id = self.canvas.state_mut().add_node(node_type, position);
                    // Select the newly created node
                    self.canvas.state_mut().selected_node = Some(node_id);
                    self.inspector.set_selected_node(Some(node_id));
                }
            }
            Message::Inspector(inspector_msg) => {
                // Handle inspector interactions
                match inspector_msg {
                    super::inspector::Message::ParameterChanged(param_name, value) => {
                        // Update inspector's internal state
                        self.inspector.update_parameter(param_name.clone(), value);
                        
                        // Update node's parameter
                        if let Some(node_id) = self.canvas.state().selected_node {
                            if let Some(node) = self.canvas.state_mut().nodes.get_mut(&node_id) {
                                node.parameters.insert(param_name, value);
                            }
                        }
                    }
                }
            }
            Message::BackToWelcome => {
                // Handled by parent
            }
        }
    }

    pub fn state(&self) -> &DspGraphState {
        self.canvas.state()
    }
}

impl Default for NodeEditor {
    fn default() -> Self {
        Self::new()
    }
}
