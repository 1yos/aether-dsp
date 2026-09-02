//! Slide-in panel for adding a new instrument track.

use iced::widget::{button, column, container, scrollable, text};
use iced::{Alignment, Color, Element, Length};

use crate::project::{TrackType, TuningSystem};
use crate::theme::Theme;

#[derive(Debug, Clone)]
pub enum Message {
    Close,
    SelectInstrument(InstrumentTemplate),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentTemplate {
    pub name: String,
    pub track_type: TrackType,
    pub tuning: TuningSystem,
}

impl InstrumentTemplate {
    fn melodic(name: &str, tuning: TuningSystem) -> Self {
        Self { name: name.to_string(), track_type: TrackType::Melodic, tuning }
    }
    fn drum(name: &str) -> Self {
        Self {
            name: name.to_string(),
            track_type: TrackType::Drum,
            tuning: TuningSystem::EqualTemperament,
        }
    }
}

type SectionEntry = (&'static str, &'static [(&'static str, TrackType, TuningSystem)]);

pub fn view() -> Element<'static, Message> {
    let sections: &[SectionEntry] = &[
        ("Ethiopian", &[
            ("Krar",    TrackType::Melodic, TuningSystem::EthiopianTizitaMajor),
            ("Masinko", TrackType::Melodic, TuningSystem::EthiopianAmbassel),
            ("Washint", TrackType::Melodic, TuningSystem::EthiopianAnchihoye),
            ("Begena",  TrackType::Melodic, TuningSystem::EthiopianBatiMinor),
        ]),
        ("Arabic", &[
            ("Oud (Rast)",     TrackType::Melodic, TuningSystem::ArabicRast),
            ("Qanun (Bayati)", TrackType::Melodic, TuningSystem::ArabicBayati),
            ("Nay (Hijaz)",    TrackType::Melodic, TuningSystem::ArabicHijaz),
        ]),
        ("Indian", &[
            ("Sitar (Yaman)", TrackType::Melodic, TuningSystem::IndianYaman),
        ]),
        ("Gamelan", &[
            ("Gamelan (Slendro)", TrackType::Melodic, TuningSystem::GamelanSlendro),
            ("Gamelan (Pelog)",   TrackType::Melodic, TuningSystem::GamelanPelog),
        ]),
        ("Synths", &[
            ("Warm Pad",   TrackType::Melodic, TuningSystem::EqualTemperament),
            ("Pluck",      TrackType::Melodic, TuningSystem::EqualTemperament),
            ("Bass Synth", TrackType::Melodic, TuningSystem::EqualTemperament),
        ]),
        ("Drums & Percussion", &[
            ("Kebero Kit", TrackType::Drum, TuningSystem::EqualTemperament),
            ("Studio Kit", TrackType::Drum, TuningSystem::EqualTemperament),
        ]),
        ("Empty", &[
            ("Melodic Track", TrackType::Melodic, TuningSystem::EqualTemperament),
            ("Drum Track",    TrackType::Drum,    TuningSystem::EqualTemperament),
        ]),
    ];

    let btn_close = button(
        text("✕").size(16).style(|_| text::Style { color: Some(Theme::TEXT_SECONDARY) }),
    )
    .on_press(Message::Close)
    .padding([6, 10])
    .style(|_, _| button::Style { background: None, ..Default::default() });

    let header = container(
        iced::widget::row![
            text("Add Track")
                .size(18)
                .style(|_| text::Style { color: Some(Theme::TEXT_PRIMARY) }),
            iced::widget::Space::with_width(Length::Fill),
            btn_close,
        ]
        .align_y(Alignment::Center)
        .padding([0, 8]),
    )
    .height(48);

    let mut list = column![].spacing(4).padding([0, 16]);
    for (cat, items) in sections {
        list = list.push(
            text(format!("── {} ──", cat))
                .size(11)
                .style(|_| text::Style { color: Some(Theme::TEXT_DISABLED) }),
        );
        for (name, ttype, tuning) in *items {
            let template = InstrumentTemplate {
                name: name.to_string(),
                track_type: *ttype,
                tuning: *tuning,
            };
            let icon = if *ttype == TrackType::Drum { "🥁" } else { "🎵" };
            list = list.push(
                button(
                    text(format!("{} {}", icon, name))
                        .size(14)
                        .style(|_| text::Style { color: Some(Theme::TEXT_PRIMARY) }),
                )
                .on_press(Message::SelectInstrument(template))
                .padding([8, 12])
                .width(Length::Fill)
                .style(|_, status| button::Style {
                    background: Some(iced::Background::Color(match status {
                        button::Status::Hovered => Theme::SURFACE,
                        _ => Color::TRANSPARENT,
                    })),
                    ..Default::default()
                }),
            );
        }
        list = list.push(iced::widget::Space::with_height(6));
    }

    container(column![header, scrollable(list).height(Length::Fill)])
        .width(280)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Theme::PANEL_BG)),
            border: iced::Border { color: Theme::BORDER, width: 1.0, ..Default::default() },
            ..Default::default()
        })
        .into()
}
