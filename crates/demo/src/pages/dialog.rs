use iced::widget::{column, text};
use iced_ui::icon_button::{self, IconButton};

use crate::Element;
use crate::message::Message;

pub(super) fn build<'a>() -> Element<'a, Message> {
    column![
        text("Dialog").size(20),
        text("Modal overlay with scrim, title, body, and action buttons.").size(14),
        text("Press the button below to open a dialog:").size(14),
        IconButton::new(text("Open Dialog").size(14))
            .variant(icon_button::Variant::Filled)
            .size(120.0)
            .on_press(Message::OpenDialog),
    ]
    .spacing(16)
    .padding(20)
    .into()
}
