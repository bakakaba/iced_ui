use iced::widget::{column, row, text};
use iced_ui::badge::{Badge, Position};
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {}

#[derive(Default)]
pub(crate) struct BadgePage;

impl super::PageView for BadgePage {
    type Msg = Msg;
    const LABEL: &'static str = "Badge";

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let dot_badge = Badge::dot(text("Mail").size(16));
        let count_badge = Badge::count(text("Inbox").size(16), 5);
        let large_count = Badge::count(text("Notifications").size(16), 1234).max(999);

        let position_item = |label: &'static str, pos: Position| -> Element<'_, Msg> {
            Badge::dot(text(label).size(14)).position(pos).into()
        };

        let positions_row = row![
            position_item("Top Left", Position::TopLeft),
            position_item("Top", Position::Top),
            position_item("Top Right", Position::TopRight),
            position_item("Right", Position::Right),
            position_item("Bottom Right", Position::BottomRight),
            position_item("Bottom", Position::Bottom),
            position_item("Bottom Left", Position::BottomLeft),
            position_item("Left", Position::Left),
        ]
        .spacing(24);

        column![
            Text::h1("Badge"),
            text("Small dot or count indicator overlaid on content.").size(14),
            Text::h2("Variants"),
            row![dot_badge, count_badge, large_count].spacing(32),
            Text::h2("Positions"),
            positions_row,
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
