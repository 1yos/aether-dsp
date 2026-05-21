// Code editor - for writing custom DSP code

use crate::theme::{AetherTheme, Spacing};
use iced::widget::{column, container, scrollable, text};
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum Message {
    CodeChanged(String),
    Compile,
    Run,
}

#[derive(Debug)]
pub struct CodeEditor {
    code: String,
    output: String,
}

impl CodeEditor {
    pub fn new() -> Self {
        Self {
            code: Self::default_code(),
            output: String::new(),
        }
    }

    fn default_code() -> String {
        r#"// Custom DSP Node
// This code will be compiled and integrated into your graph

use aetherdsp_core::node::DspNode;
use aetherdsp_core::BUFFER_SIZE;

pub struct CustomNode {
    // Your state here
    gain: f32,
}

impl CustomNode {
    pub fn new() -> Self {
        Self {
            gain: 1.0,
        }
    }
}

impl DspNode for CustomNode {
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        _sample_rate: f32,
    ) {
        if inputs.is_empty() || outputs.is_empty() {
            return;
        }

        let input = inputs[0];
        let output = outputs[0];

        for i in 0..BUFFER_SIZE {
            output[i] = input[i] * self.gain;
        }
    }

    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
}
"#
        .to_string()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let title = text("Code Editor").size(16).style(|_theme| text::Style {
            color: Some(AetherTheme::TEXT_PRIMARY),
        });

        let subtitle = text("Write custom DSP nodes in Rust")
            .size(12)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_SECONDARY),
            });

        // Code editor (simplified - in production would use a proper code editor widget)
        let code_display = scrollable(text(&self.code).size(12).style(|_theme| text::Style {
            color: Some(AetherTheme::TEXT_PRIMARY),
        }))
        .height(Length::Fill);

        let output_title = text("Output").size(12).style(|_theme| text::Style {
            color: Some(AetherTheme::TEXT_SECONDARY),
        });

        let output_display = container(text(&self.output).size(10).style(|_theme| text::Style {
            color: Some(AetherTheme::TEXT_DISABLED),
        }))
        .padding(Spacing::SM)
        .style(|_theme| container::Style {
            background: Some(AetherTheme::NODE_BACKGROUND.into()),
            ..Default::default()
        });

        let content = column![title, subtitle, code_display, output_title, output_display,]
            .spacing(Spacing::MD)
            .padding(Spacing::MD);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(AetherTheme::PANEL_BACKGROUND.into()),
                ..Default::default()
            })
            .into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::CodeChanged(new_code) => {
                self.code = new_code;
            }
            Message::Compile => {
                self.output = "Compiling...".to_string();
                // TODO: Implement compilation
            }
            Message::Run => {
                self.output = "Running...".to_string();
                // TODO: Implement execution
            }
        }
    }
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self::new()
    }
}
