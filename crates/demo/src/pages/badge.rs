use iced::widget::{column, row, text};
use iced_ui::badge::Badge;

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

        column![
            text("Badge").size(20),
            text("Small dot or count indicator overlaid on content.").size(14),
            row![dot_badge, count_badge, large_count].spacing(32),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
