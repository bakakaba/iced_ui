use iced::widget::{column, row, text};
use iced_ui::chip::Chip;

use crate::Element;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Toggled,
    Noop,
}

#[derive(Debug, Default)]
pub(super) struct ChipPage {
    selected: bool,
}

impl ChipPage {
    pub(super) fn update(&mut self, message: Message) {
        match message {
            Message::Toggled => self.selected = !self.selected,
            Message::Noop => {}
        }
    }

    pub(super) fn view(&self) -> Element<'_, Message> {
        let assist = Chip::assist(text("Add event").size(14)).on_press(Message::Noop);

        let filter = Chip::filter(text("Vegetarian").size(14))
            .selected(self.selected)
            .on_press(Message::Toggled);

        let input = Chip::input(text("John Doe").size(14))
            .on_press(Message::Noop)
            .on_close(Message::Noop);

        let suggestion = Chip::suggestion(text("Quick reply").size(14)).on_press(Message::Noop);

        column![
            text("Chip").size(20),
            text("Assist, Filter, Input, Suggestion variants.").size(14),
            row![assist, filter, input, suggestion].spacing(12),
            text(format!("Filter chip selected: {}", self.selected)).size(12),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
