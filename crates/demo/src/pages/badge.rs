use iced::widget::{column, container, row, text};
use iced_ui::badge::{Badge, Position};

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
            Badge::dot(container(text(label).size(12)).center_x(32).center_y(32))
                .position(pos)
                .into()
        };

        let positions_row = row![
            position_item("TL", Position::TopLeft),
            position_item("T", Position::Top),
            position_item("TR", Position::TopRight),
            position_item("R", Position::Right),
            position_item("BR", Position::BottomRight),
            position_item("B", Position::Bottom),
            position_item("BL", Position::BottomLeft),
            position_item("L", Position::Left),
        ]
        .spacing(24);

        column![
            text("Badge").size(20),
            text("Small dot or count indicator overlaid on content.").size(14),
            row![dot_badge, count_badge, large_count].spacing(32),
            text("Positions").size(16),
            positions_row,
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
