use iced::widget::{column, text};
use iced_ui::icon_button::{self, IconButton};
use iced_ui::snackbar::Snackbar;

use crate::Element;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Show,
    Hide,
}

#[derive(Debug, Default)]
pub(super) struct SnackbarPage {
    visible: bool,
}

impl SnackbarPage {
    pub(super) fn update(&mut self, message: Message) {
        match message {
            Message::Show => self.visible = true,
            Message::Hide => self.visible = false,
        }
    }

    pub(super) fn view(&self) -> Element<'_, Message> {
        let host_content: Element<'_, Message> = column![
            text("Snackbar").size(20),
            text("Temporary notification bar at the bottom of the host.").size(14),
            IconButton::new(text("Show Snackbar").size(14))
                .variant(icon_button::Variant::Filled)
                .size(140.0)
                .on_press(Message::Show),
        ]
        .spacing(16)
        .padding(20)
        .into();

        Snackbar::new(host_content)
            .message("Item has been archived.")
            .action("Undo", Message::Hide)
            .on_dismiss(Message::Hide)
            .visible(self.visible)
            .into()
    }
}
