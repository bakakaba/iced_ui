use iced::widget::{column, container, row, text};
use iced::{Border, Color};
use iced_ui::Tooltip;
use iced_ui::text::Text;
use iced_ui::tooltip::Position;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {}

#[derive(Default)]
pub(crate) struct TooltipPage;

impl super::PageView for TooltipPage {
    type Msg = Msg;
    const LABEL: &'static str = "Tooltip";

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let trigger = |label: &'static str| {
            container(text(label).size(14))
                .padding([8, 16])
                .style(|theme: &iced_ui::Theme| iced::widget::container::Style {
                    border: Border {
                        color: Color::from_rgb(0.3, 0.3, 0.3),
                        width: 1.0,
                        radius: theme.radius(iced_ui::Roundness::sx(1.0)).into(),
                    },
                    ..Default::default()
                })
        };

        let item = |label: &'static str, pos: Position| -> Element<'_, Msg> {
            Tooltip::new(trigger(label), text(label), pos).into()
        };

        let positions_row = row![
            item("Top", Position::Top),
            item("Bottom", Position::Bottom),
            item("Left", Position::Left),
            item("Right", Position::Right),
            item("Follow Cursor", Position::FollowCursor),
        ]
        .spacing(24);

        column![
            text("Reveals a floating bubble while its trigger is hovered.").size(14),
            Text::h2("Positions"),
            positions_row,
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
