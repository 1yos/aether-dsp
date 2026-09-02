//! Launch screen — shown when Aether Studio opens.

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Length};

use crate::project::TuningSystem;
use crate::theme::Theme;

#[derive(Debug, Clone)]
pub enum Message {
    NewProject,
    OpenProject,
    OpenRecent(String),
    ProjectNameChanged(String),
    BpmChanged(String),
    TuningSelected(TuningSystem),
    ConfirmCreate,
    CancelCreate,
}

pub struct LaunchScreen {
    pub recent_projects: Vec<String>,
    pub show_dialog: bool,
    pub project_name: String,
    pub bpm_str: String,
    pub selected_tuning: TuningSystem,
}

impl Default for LaunchScreen {
    fn default() -> Self {
        Self {
            recent_projects: Vec::new(),
            show_dialog: false,
            project_name: "Untitled Project".to_string(),
            bpm_str: "120".to_string(),
            selected_tuning: TuningSystem::EthiopianTizitaMajor,
        }
    }
}

impl LaunchScreen {
    pub fn view(&self) -> Element<'_, Message> {
        if self.show_dialog {
            self.view_dialog()
        } else {
            self.view_home()
        }
    }

    fn view_home(&self) -> Element<'_, Message> {
        let logo = text("◈  AETHER STUDIO")
            .size(36)
            .style(|_| text::Style { color: Some(Theme::ACCENT) });

        let subtitle = text("World Music Production")
            .size(16)
            .style(|_| text::Style { color: Some(Theme::TEXT_SECONDARY) });

        let btn_new = button(text("  ✦  New Project  ").size(18))
            .on_press(Message::NewProject)
            .padding([14, 32])
            .style(|_, status| button::Style {
                background: Some(iced::Background::Color(match status {
                    button::Status::Hovered | button::Status::Pressed => Theme::ACCENT_DIM,
                    _ => Theme::ACCENT,
                })),
                text_color: Color::BLACK,
                border: iced::Border { radius: 6.0.into(), ..Default::default() },
                ..Default::default()
            });

        let btn_open = button(text("  📂  Open Project  ").size(16))
            .on_press(Message::OpenProject)
            .padding([12, 32])
            .style(|_, _| button::Style {
                background: Some(iced::Background::Color(Color::TRANSPARENT)),
                text_color: Theme::TEXT_PRIMARY,
                border: iced::Border { color: Theme::BORDER, width: 1.0, radius: 6.0.into() },
                ..Default::default()
            });

        let recent_header = text("── Recent ──────────────────────────────────────")
            .size(12)
            .style(|_| text::Style { color: Some(Theme::TEXT_DISABLED) });

        let mut recent_col = column![recent_header].spacing(4);

        if self.recent_projects.is_empty() {
            recent_col = recent_col.push(
                text("No recent projects")
                    .size(13)
                    .style(|_| text::Style { color: Some(Theme::TEXT_DISABLED) }),
            );
        } else {
            for name in &self.recent_projects {
                let n = name.clone();
                recent_col = recent_col.push(
                    button(text(n.clone()).size(14))
                        .on_press(Message::OpenRecent(n))
                        .padding([6, 0])
                        .style(|_, _| button::Style {
                            background: None,
                            text_color: Theme::TEXT_PRIMARY,
                            ..Default::default()
                        }),
                );
            }
        }

        let version = text("v1.0.0  ·  MIT")
            .size(11)
            .style(|_| text::Style { color: Some(Theme::TEXT_DISABLED) });

        let content = column![
            logo,
            subtitle,
            iced::widget::Space::with_height(40),
            btn_new,
            iced::widget::Space::with_height(12),
            btn_open,
            iced::widget::Space::with_height(48),
            recent_col,
            iced::widget::Space::with_height(Length::Fill),
            version,
        ]
        .spacing(0)
        .width(400)
        .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Theme::APP_BG)),
                ..Default::default()
            })
            .into()
    }

    fn view_dialog(&self) -> Element<'_, Message> {
        let title = text("New Project")
            .size(20)
            .style(|_| text::Style { color: Some(Theme::TEXT_PRIMARY) });

        let btn_cancel = button(text("✕").size(16).style(|_| text::Style {
            color: Some(Theme::TEXT_SECONDARY),
        }))
        .on_press(Message::CancelCreate)
        .padding([6, 10])
        .style(|_, _| button::Style { background: None, ..Default::default() });

        let name_label = text("Project name")
            .size(13)
            .style(|_| text::Style { color: Some(Theme::TEXT_SECONDARY) });

        let name_input = text_input("Untitled Project", &self.project_name)
            .on_input(Message::ProjectNameChanged)
            .padding(10)
            .style(|_, _| text_input::Style {
                background: iced::Background::Color(Theme::SURFACE),
                border: iced::Border { color: Theme::BORDER, width: 1.0, radius: 4.0.into() },
                icon: Theme::TEXT_SECONDARY,
                placeholder: Theme::TEXT_DISABLED,
                value: Theme::TEXT_PRIMARY,
                selection: Theme::ACCENT,
            });

        let bpm_label = text("Tempo (BPM)")
            .size(13)
            .style(|_| text::Style { color: Some(Theme::TEXT_SECONDARY) });

        let bpm_input = text_input("120", &self.bpm_str)
            .on_input(Message::BpmChanged)
            .padding(10)
            .width(120)
            .style(|_, _| text_input::Style {
                background: iced::Background::Color(Theme::SURFACE),
                border: iced::Border { color: Theme::BORDER, width: 1.0, radius: 4.0.into() },
                icon: Theme::TEXT_SECONDARY,
                placeholder: Theme::TEXT_DISABLED,
                value: Theme::TEXT_PRIMARY,
                selection: Theme::ACCENT,
            });

        let tuning_label = text("Default tuning")
            .size(13)
            .style(|_| text::Style { color: Some(Theme::TEXT_SECONDARY) });

        // Tuning list
        let mut tuning_col = column![].spacing(4);
        let mut current_cat = "";
        for t in TuningSystem::all() {
            let cat = t.category();
            if cat != current_cat {
                current_cat = cat;
                tuning_col = tuning_col.push(
                    text(format!("── {} ──", cat))
                        .size(11)
                        .style(|_| text::Style { color: Some(Theme::TEXT_DISABLED) }),
                );
            }
            let selected = *t == self.selected_tuning;
            let tt = *t;
            tuning_col = tuning_col.push(
                button(
                    text(t.display_name())
                        .size(13)
                        .style(move |_| text::Style {
                            color: Some(if selected { Color::BLACK } else { Theme::TEXT_PRIMARY }),
                        }),
                )
                .on_press(Message::TuningSelected(tt))
                .padding([5, 10])
                .width(Length::Fill)
                .style(move |_, _| button::Style {
                    background: Some(iced::Background::Color(if selected {
                        Theme::ACCENT
                    } else {
                        Color::TRANSPARENT
                    })),
                    border: iced::Border {
                        color: if selected { Theme::ACCENT } else { Theme::BORDER },
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    text_color: if selected { Color::BLACK } else { Theme::TEXT_PRIMARY },
                    ..Default::default()
                }),
            );
        }

        let btn_create = button(text("  ✦  Create Project  ").size(16))
            .on_press(Message::ConfirmCreate)
            .padding([12, 24])
            .style(|_, _| button::Style {
                background: Some(iced::Background::Color(Theme::ACCENT)),
                text_color: Color::BLACK,
                border: iced::Border { radius: 6.0.into(), ..Default::default() },
                ..Default::default()
            });

        let dialog_inner = column![
            row![
                title,
                iced::widget::Space::with_width(Length::Fill),
                btn_cancel,
            ]
            .align_y(Alignment::Center),
            iced::widget::Space::with_height(20),
            name_label,
            name_input,
            iced::widget::Space::with_height(16),
            bpm_label,
            bpm_input,
            iced::widget::Space::with_height(16),
            tuning_label,
            scrollable(tuning_col).height(200),
            iced::widget::Space::with_height(20),
            btn_create,
        ]
        .spacing(6)
        .padding(28)
        .width(440);

        let dialog = container(dialog_inner).style(|_| container::Style {
            background: Some(iced::Background::Color(Theme::PANEL_BG)),
            border: iced::Border { color: Theme::BORDER, width: 1.0, radius: 8.0.into() },
            ..Default::default()
        });

        container(dialog)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 })),
                ..Default::default()
            })
            .into()
    }
}
