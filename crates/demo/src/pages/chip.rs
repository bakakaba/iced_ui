use iced::widget::{column, row, text};
use iced_ui::chip::Chip;

use crate::Element;
use crate::app::Demo;
use crate::message::Message;

pub(super) fn build<'a>(demo: &Demo) -> Element<'a, Message> {
    let assist = Chip::assist(text("Add event").size(14)).on_press(Message::Noop);

    let filter = Chip::filter(text("Vegetarian").size(14))
        .selected(demo.chip_selected)
        .on_press(Message::ChipToggled);

    let input = Chip::input(text("John Doe").size(14))
        .on_press(Message::Noop)
        .on_close(Message::Noop);

    let suggestion = Chip::suggestion(text("Quick reply").size(14)).on_press(Message::Noop);

    column![
        text("Chip").size(20),
        text("Assist, Filter, Input, Suggestion variants.").size(14),
        row![assist, filter, input, suggestion].spacing(12),
        text(format!("Filter chip selected: {}", demo.chip_selected)).size(12),
    ]
    .spacing(16)
    .padding(20)
    .into()
}
