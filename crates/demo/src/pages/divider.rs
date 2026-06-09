use iced::Length;
use iced::widget::{column, row, text};
use iced_ui::divider::Divider;
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {}

#[derive(Default)]
pub(crate) struct DividerPage;

impl super::PageView for DividerPage {
    type Msg = Msg;
    const LABEL: &'static str = "Divider";

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        column![
            Text::h1("Divider"),
            text("Horizontal and vertical separators with optional insets.").size(14),
            Text::h2("Type"),
            Divider::horizontal(),
            Divider::horizontal().inset(iced_ui::Space::sx(2.0)),
            Text::h2("Axis"),
            Divider::horizontal(),
            row![
                text("Left").size(14),
                Divider::vertical(),
                text("Right").size(14),
            ]
            .spacing(8)
            .height(Length::Fixed(40.0)),
        ]
        .spacing(12)
        .padding(20)
        .into()
    }
}
