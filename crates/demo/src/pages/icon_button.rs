use iced::widget::{column, row, text};
use iced_ui::icon_button::{self, IconButton};

use crate::Element;
use crate::app::Demo;
use crate::message::Message;

pub(super) fn build<'a>(demo: &Demo) -> Element<'a, Message> {
    let standard = IconButton::new(text("X").size(18)).on_press(Message::Noop);

    let filled = IconButton::new(text("+").size(18))
        .variant(icon_button::Variant::Filled)
        .on_press(Message::Noop);

    let tonal = IconButton::new(text("?").size(18))
        .variant(icon_button::Variant::FilledTonal)
        .on_press(Message::Noop);

    let outlined = IconButton::new(text("!").size(18))
        .variant(icon_button::Variant::Outlined)
        .on_press(Message::Noop);

    let toggle = IconButton::new(text("*").size(18))
        .variant(icon_button::Variant::Filled)
        .toggled(demo.icon_btn_toggled)
        .on_press(Message::IconButtonToggled);

    column![
        text("IconButton").size(20),
        text("Four variants: Standard, Filled, Filled Tonal, Outlined. Supports toggle.").size(14),
        row![standard, filled, tonal, outlined, toggle].spacing(16),
        text(format!("Toggle state: {}", demo.icon_btn_toggled)).size(12),
    ]
    .spacing(16)
    .padding(20)
    .into()
}
