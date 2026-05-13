use iced::widget::{column, text};
use iced_ui::icon_button::{self, IconButton};
use iced_ui::snackbar::Snackbar;

use crate::Element;
use crate::app::Demo;
use crate::message::Message;

pub(super) fn build<'a>(demo: &Demo) -> Element<'a, Message> {
    let host_content: Element<'_, Message> = column![
        text("Snackbar").size(20),
        text("Temporary notification bar at the bottom of the host.").size(14),
        IconButton::new(text("Show Snackbar").size(14))
            .variant(icon_button::Variant::Filled)
            .size(140.0)
            .on_press(Message::ShowSnackbar),
    ]
    .spacing(16)
    .padding(20)
    .into();

    Snackbar::new(host_content)
        .message("Item has been archived.")
        .action("Undo", Message::HideSnackbar)
        .on_dismiss(Message::HideSnackbar)
        .visible(demo.snackbar_visible)
        .into()
}
