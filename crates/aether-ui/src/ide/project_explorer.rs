// Project Explorer - File tree view

use crate::theme::{AetherTheme, Spacing};
use iced::widget::{button, column, container, scrollable, text};
use iced::{Element, Length};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    FileClicked(PathBuf),
    FolderToggled(PathBuf),
}

#[derive(Debug)]
pub struct ProjectExplorer {
    root_path: Option<PathBuf>,
    expanded_folders: Vec<PathBuf>,
}

impl ProjectExplorer {
    pub fn new() -> Self {
        Self {
            root_path: None,
            expanded_folders: Vec::new(),
        }
    }

    pub fn set_root(&mut self, path: PathBuf) {
        self.root_path = Some(path);
    }

    pub fn view(&self) -> Element<'_, Message> {
        let title = text("PROJECT EXPLORER")
            .size(12)
            .style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_SECONDARY),
            });

        let content = if let Some(root) = &self.root_path {
            column![title, self.view_directory(root, 0)].spacing(Spacing::XS)
        } else {
            column![
                title,
                text("No project open")
                    .size(12)
                    .style(|_theme| text::Style {
                        color: Some(AetherTheme::TEXT_DISABLED),
                    })
            ]
            .spacing(Spacing::SM)
        };

        container(scrollable(content.padding(Spacing::MD)))
            .width(250)
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

    fn view_directory(&self, _path: &PathBuf, _indent: usize) -> Element<'_, Message> {
        // TODO: Implement actual directory traversal
        let items = vec![
            ("📁 src", true),
            ("  📄 main.rs", false),
            ("  📄 graph.rs", false),
            ("  📁 nodes", true),
            ("    📄 oscillator.rs", false),
            ("    📄 filter.rs", false),
            ("📄 Cargo.toml", false),
            ("📄 README.md", false),
        ];

        let mut col = column![].spacing(Spacing::XS);

        for (name, _is_folder) in items {
            let btn = button(text(name).size(12).style(|_theme| text::Style {
                color: Some(AetherTheme::TEXT_PRIMARY),
            }))
            .style(button::text)
            .padding(Spacing::XS);

            col = col.push(btn);
        }

        col.into()
    }
}

impl Default for ProjectExplorer {
    fn default() -> Self {
        Self::new()
    }
}
