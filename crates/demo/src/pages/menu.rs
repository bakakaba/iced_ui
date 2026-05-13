use iced::widget::{column, row, text};

use crate::Element;
use crate::app::Demo;
use crate::message::Message;

pub(super) fn build<'a>(demo: &Demo) -> Element<'a, Message> {
    let status = match &demo.last_action {
        Some(last) => format!("Last action: {last} (count: {})", demo.counter),
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
