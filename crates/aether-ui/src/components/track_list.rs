//! Left panel — scrollable list of tracks + Add Track button.

use iced::widget::{button, column, container, scrollable, text};
use iced::{Element, Length};

use crate::project::{Track, TrackId};
use crate::theme::Theme;

use super::track_strip;

#[derive(Debug, Clone)]
pub enum Message {
    AddTrack,
    Strip(track_strip::Message),
}

pub fn view<'a>(tracks: &'a [Track], selected: Option<TrackId>) -> Element<'a, Message> {
    let header = container(
        text("TRACKS")
            .size(11)
            .style(|_| text::Style { color: Some(Theme::TEXT_DISABLED) }),
    )
    .padding([10, 16]);

    let mut list = column![].spacing(0);
    for track in tracks {
        let is_selected = selected == Some(track.id);
        list = list.push(track_strip::view_strip(track, is_selected).map(Message::Strip));
    }

    let btn_add = button(
        text("+ Add Track")
            .size(13)
            .style(|_| text::Style { color: Some(Theme::ACCENT) }),
    )
    .on_press(Message::AddTrack)
    .padding([10, 16])
    .width(Length::Fill)
    .style(|_, status| button::Style {
        background: Some(iced::Background::Color(match status {
            button::Status::Hovered => Theme::SURFACE,
            _ => iced::Color::TRANSPARENT,
        })),
        border: iced::Border { color: Theme::BORDER, width: 1.0, ..Default::default() },
        ..Default::default()
    });

    let content = column![header, scrollable(list).height(Length::Fill), btn_add];

    container(content)
        .width(260)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Theme::PANEL_BG)),
            border: iced::Border { color: Theme::BORDER, width: 1.0, ..Default::default() },
            ..Default::default()
        })
        .into()
}
