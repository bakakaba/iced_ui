use iced::widget::{column, text};
use iced_ui::bottom_sheet::BottomSheet;
use iced_ui::icon_button::{self, IconButton};

use crate::Element;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Toggle,
    Close,
}

#[derive(Debug, Default)]
pub(super) struct BottomSheetPage {
    expanded: bool,
}

impl BottomSheetPage {
    pub(super) fn update(&mut self, message: Message) {
        match message {
            Message::Toggle => self.expanded = !self.expanded,
            Message::Close => self.expanded = false,
        }
    }

    pub(super) fn view(&self) -> Element<'_, Message> {
        let host_content: Element<'_, Message> = column![
            text("Bottom Sheet").size(20),
            text("A panel sliding from the bottom. Modal or standard.").size(14),
            IconButton::new(text("Toggle Sheet").size(14))
                .variant(icon_button::Variant::Filled)
                .size(140.0)
                .on_press(Message::Toggle),
        ]
        .spacing(16)
        .padding(20)
        .into();

        BottomSheet::new(
            host_content,
            "This is the bottom sheet content. It slides up from the bottom of the screen.",
        )
        .modal(true)
        .expanded(self.expanded)
        .on_dismiss(Message::Close)
        .drag_handle(true)
        .into()
    }
}
