// Inspector panel - shows properties of selected node

use super::{GraphNode, NodeId};
use crate::theme::{AetherTheme, Spacing};
use iced::widget::{column, container, slider, text};
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum Message {
    ParameterChanged(String, f32),
}

#[derive(Debug)]
pub struct Inspector {
    selected_node: Option<NodeId>,
    parameter_values: std::collections::HashMap<String, f32>,
}

impl Inspector {
    pub fn new() -> Self {
        Self {
            selected_node: None,
            parameter_values: std::collections::HashMap::new(),
        }
    }

    pub fn set_selected_node(&mut self, node_id: Option<NodeId>) {
        self.selected_node = node_id;
        self.parameter_values.clear();
    }

    pub fn update_parameter(&mut self, name: String, value: f32) {
        self.parameter_values.insert(name, value);
    }

    pub fn view<'a>(&self, node: Option<&'a GraphNode>) -> Element<'a, Message> {
        let title = text("Inspector")
            .size(16)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_PRIMARY),
            });

        let content = if let Some(node) = node {
            self.view_node_properties(node)
        } else {
            column![
                text("No node selected")
                    .size(12)
                    .style(|_theme| text::Style {
                        color: Some(AetherTheme::TEXT_DISABLED),
                    })
            ]
            .spacing(Spacing::MD)
            .into()
        };

        let full_content = column![title, content]
            .spacing(Spacing::MD)
            .padding(Spacing::MD);

        container(full_content)
            .width(280)
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

    fn view_node_properties<'a>(&self, node: &'a GraphNode) -> Element<'a, Message> {
        let node_name = text(node.node_type.name())
            .size(14)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_PRIMARY),
            });

        let node_id = text(format!("ID: {:?}", node.id))
            .size(10)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_DISABLED),
            });

        let position = text(format!("Position: ({:.0}, {:.0})", node.position.x, node.position.y))
            .size(10)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_DISABLED),
            });

        // Parameters section
        let mut params: Vec<Element<'a, Message>> = vec![
            text("Parameters")
                .size(12)
                .style(|_theme| text::Style {
                    color: Some(AetherTheme::TEXT_SECONDARY),
                })
                .into(),
        ];

        // Add common parameters based on node type
        params.extend(self.get_node_parameters(&node.node_type));

        column![node_name, node_id, position]
            .spacing(Spacing::SM)
            .extend(params)
            .into()
    }

    fn get_node_parameters<'a>(&self, node_type: &super::NodeType) -> Vec<Element<'a, Message>> {
        use super::NodeType;
        
        match node_type {
            NodeType::Gain => vec![
                self.parameter_slider("Gain", 0.0, 2.0, self.get_param_value("Gain", 1.0)),
            ],
            NodeType::Oscillator { .. } => vec![
                self.parameter_slider("Frequency", 20.0, 20000.0, self.get_param_value("Frequency", 440.0)),
                self.parameter_slider("Amplitude", 0.0, 1.0, self.get_param_value("Amplitude", 0.5)),
            ],
            NodeType::LowPass | NodeType::HighPass | NodeType::BandPass => vec![
                self.parameter_slider("Cutoff", 20.0, 20000.0, self.get_param_value("Cutoff", 1000.0)),
                self.parameter_slider("Resonance", 0.0, 1.0, self.get_param_value("Resonance", 0.5)),
            ],
            NodeType::Delay => vec![
                self.parameter_slider("Time", 0.0, 2.0, self.get_param_value("Time", 0.5)),
                self.parameter_slider("Feedback", 0.0, 1.0, self.get_param_value("Feedback", 0.3)),
                self.parameter_slider("Mix", 0.0, 1.0, self.get_param_value("Mix", 0.5)),
            ],
            NodeType::Compressor => vec![
                self.parameter_slider("Threshold", -60.0, 0.0, self.get_param_value("Threshold", -20.0)),
                self.parameter_slider("Ratio", 1.0, 20.0, self.get_param_value("Ratio", 4.0)),
                self.parameter_slider("Attack", 0.0, 100.0, self.get_param_value("Attack", 10.0)),
                self.parameter_slider("Release", 0.0, 1000.0, self.get_param_value("Release", 100.0)),
            ],
            NodeType::Reverb => vec![
                self.parameter_slider("Room Size", 0.0, 1.0, self.get_param_value("Room Size", 0.5)),
                self.parameter_slider("Damping", 0.0, 1.0, self.get_param_value("Damping", 0.5)),
                self.parameter_slider("Mix", 0.0, 1.0, self.get_param_value("Mix", 0.3)),
            ],
            NodeType::LFO => vec![
                self.parameter_slider("Rate", 0.01, 20.0, self.get_param_value("Rate", 1.0)),
                self.parameter_slider("Depth", 0.0, 1.0, self.get_param_value("Depth", 0.5)),
            ],
            NodeType::Envelope => vec![
                self.parameter_slider("Attack", 0.0, 2.0, self.get_param_value("Attack", 0.01)),
                self.parameter_slider("Decay", 0.0, 2.0, self.get_param_value("Decay", 0.1)),
                self.parameter_slider("Sustain", 0.0, 1.0, self.get_param_value("Sustain", 0.7)),
                self.parameter_slider("Release", 0.0, 5.0, self.get_param_value("Release", 0.3)),
            ],
            _ => vec![
                text("No parameters")
                    .size(10)
                    .style(|_theme| text::Style {
                        color: Some(AetherTheme::TEXT_DISABLED),
                    })
                    .into(),
            ],
        }
    }

    fn get_param_value(&self, name: &str, default: f32) -> f32 {
        self.parameter_values.get(name).copied().unwrap_or(default)
    }

    fn parameter_slider<'a>(&self, name: &'a str, min: f32, max: f32, default: f32) -> Element<'a, Message> {
        let label = text(name)
            .size(10)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_SECONDARY),
            });

        let value_text = text(format!("{:.2}", default))
            .size(10)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_PRIMARY),
            });

        let name_owned = name.to_string();
        let slider_widget = slider(min..=max, default, move |value| {
            Message::ParameterChanged(name_owned.clone(), value)
        })
        .step(0.01);

        column![label, slider_widget, value_text]
            .spacing(Spacing::XS)
            .into()
    }
}

impl Default for Inspector {
    fn default() -> Self {
        Self::new()
    }
}
