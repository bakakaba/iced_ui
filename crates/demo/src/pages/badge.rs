use iced::widget::{column, container, row, text};
use iced::{Border, Color};
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

        let bordered = |label: &'static str| {
            container(text(label).size(14))
                .padding([4, 12])
                .style(|theme: &iced_ui::Theme| iced::widget::container::Style {
                    border: Border {
                        color: Color::from_rgb(0.3, 0.3, 0.3),
                        width: 1.0,
                        radius: theme.radius(iced_ui::Roundness::sx(1.0)).into(),
                    },
                    ..Default::default()
                })
        };

        let position_item = |label: &'static str, pos: Position| -> Element<'_, Msg> {
            Badge::dot(bordered(label)).position(pos).into()
        };

        let count_position_item = |label: &'static str, pos: Position| -> Element<'_, Msg> {
            Badge::count(bordered(label), 42).position(pos).into()
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

        let count_positions_row = row![
            count_position_item("Top Left", Position::TopLeft),
            count_position_item("Top", Position::Top),
            count_position_item("Top Right", Position::TopRight),
            count_position_item("Right", Position::Right),
            count_position_item("Bottom Right", Position::BottomRight),
            count_position_item("Bottom", Position::Bottom),
            count_position_item("Bottom Left", Position::BottomLeft),
            count_position_item("Left", Position::Left),
        ]
        .spacing(24);

        column![
            Text::h1("Badge"),
            text("Small dot or count indicator overlaid on content.").size(14),
            Text::h2("Variants"),
            row![dot_badge, count_badge, large_count].spacing(32),
            Text::h2("Positions (dot)"),
            positions_row,
            Text::h2("Positions (count)"),
            count_positions_row,
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
