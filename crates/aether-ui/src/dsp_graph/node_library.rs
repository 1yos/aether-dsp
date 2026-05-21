// Node library panel - browsable list of available DSP nodes

use super::{NodeCategory, NodeType, Waveform};
use crate::theme::{AetherTheme, Spacing};
use iced::widget::{button, column, container, scrollable, text, text_input};
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum Message {
    SearchChanged(String),
    CategorySelected(Option<NodeCategory>),
    NodeSelected(NodeType),
}

#[derive(Debug)]
pub struct NodeLibrary {
    search_query: String,
    selected_category: Option<NodeCategory>,
}

impl NodeLibrary {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            selected_category: None,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::SearchChanged(query) => {
                self.search_query = query;
            }
            Message::CategorySelected(category) => {
                self.selected_category = category;
            }
            Message::NodeSelected(_) => {
                // Handled by parent
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let title = text("Node Library").size(16).style(|_theme| text::Style {
            color: Some(AetherTheme::TEXT_PRIMARY),
        });

        let search = text_input("Search nodes...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding(Spacing::SM);

        let categories = self.view_categories();
        let nodes = self.view_nodes();

        let content = column![title, search, categories, nodes]
            .spacing(Spacing::MD)
            .padding(Spacing::MD);

        container(content)
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

    fn view_categories(&self) -> Element<'_, Message> {
        let categories = vec![
            NodeCategory::AudioIO,
            NodeCategory::Generator,
            NodeCategory::Filter,
            NodeCategory::Dynamics,
            NodeCategory::TimeBased,
            NodeCategory::Distortion,
            NodeCategory::Utility,
            NodeCategory::Modulator,
        ];

        let all_button = button(text("All").size(12))
            .on_press(Message::CategorySelected(None))
            .padding(Spacing::XS)
            .style(if self.selected_category.is_none() {
                button::primary
            } else {
                button::secondary
            });

        let mut category_buttons = vec![all_button.into()];

        for category in categories {
            let is_selected = self.selected_category == Some(category);
            let category_name = category.name().to_string();
            let btn = button(text(category_name).size(12))
                .on_press(Message::CategorySelected(Some(category)))
                .padding(Spacing::XS)
                .style(if is_selected {
                    button::primary
                } else {
                    button::secondary
                });
            category_buttons.push(btn.into());
        }

        column(category_buttons).spacing(Spacing::XS).into()
    }

    fn view_nodes(&self) -> Element<'_, Message> {
        let nodes = self.get_filtered_nodes();

        let node_list: Vec<Element<Message>> = nodes
            .into_iter()
            .map(|node_type| {
                let name = node_type.name().to_string();
                let btn = button(text(name).size(12).style(|_theme| text::Style {
                    color: Some(AetherTheme::TEXT_PRIMARY),
                }))
                .on_press(Message::NodeSelected(node_type))
                .width(Length::Fill)
                .padding(Spacing::SM)
                .style(button::secondary);

                btn.into()
            })
            .collect();

        scrollable(column(node_list).spacing(Spacing::XS))
            .height(Length::Fill)
            .into()
    }

    fn get_filtered_nodes(&self) -> Vec<NodeType> {
        let all_nodes = self.get_all_nodes();

        all_nodes
            .into_iter()
            .filter(|node| {
                // Filter by category
                if let Some(category) = self.selected_category {
                    if node.category() != category {
                        return false;
                    }
                }

                // Filter by search query
                if !self.search_query.is_empty() {
                    let query = self.search_query.to_lowercase();
                    let name = node.name().to_lowercase();
                    if !name.contains(&query) {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    fn get_all_nodes(&self) -> Vec<NodeType> {
        vec![
            // Audio I/O
            NodeType::AudioInput,
            NodeType::AudioOutput,
            NodeType::MidiInput,
            NodeType::MidiOutput,
            // Generators
            NodeType::Oscillator {
                waveform: Waveform::Sine,
            },
            NodeType::NoiseGenerator,
            NodeType::SamplePlayer,
            NodeType::Wavetable,
            // Filters
            NodeType::LowPass,
            NodeType::HighPass,
            NodeType::BandPass,
            NodeType::Notch,
            NodeType::AllPass,
            NodeType::StateVariable,
            NodeType::MoogLadder,
            // Dynamics
            NodeType::Compressor,
            NodeType::Limiter,
            NodeType::Gate,
            NodeType::Expander,
            // Time-based
            NodeType::Delay,
            NodeType::Reverb,
            NodeType::Chorus,
            NodeType::Flanger,
            NodeType::Phaser,
            // Distortion
            NodeType::Waveshaper,
            NodeType::Saturation,
            NodeType::BitCrusher,
            // Utilities
            NodeType::Gain,
            NodeType::Mixer,
            NodeType::Pan,
            NodeType::Scope,
            NodeType::Analyzer,
            // Modulators
            NodeType::LFO,
            NodeType::Envelope,
        ]
    }
}

impl Default for NodeLibrary {
    fn default() -> Self {
        Self::new()
    }
}
