use iced::widget::{column, row, text};

use crate::Element;
use crate::state::ActionLog;

pub(super) fn build<'a>(log: &ActionLog) -> Element<'a, super::Message> {
    let status = match &log.last_action {
        Some(last) => format!("Last action: {last} (count: {})", log.counter),
        None => "Try opening a menu, activating an item, or pressing a shortcut.".to_string(),
    };

    column![
        text("MenuBar").size(20),
        text(status),
        row![
            text("Try keyboard shortcuts: "),
            text("Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+Z, Ctrl+Shift+Z, Ctrl+Q, F1").size(14),
        ]
        .spacing(8),
    ]
    .spacing(8)
    .padding(20)
    .into()
}
