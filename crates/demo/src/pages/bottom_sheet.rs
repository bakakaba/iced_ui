use iced::widget::{column, text};
use iced_ui::bottom_sheet::BottomSheet;
use iced_ui::icon_button::{self, IconButton};

use crate::Element;
use crate::app::Demo;
use crate::message::Message;

pub(super) fn build<'a>(demo: &Demo) -> Element<'a, Message> {
    let host_content: Element<'_, Message> = column![
        text("Bottom Sheet").size(20),
        text("A panel sliding from the bottom. Modal or standard.").size(14),
        IconButton::new(text("Toggle Sheet").size(14))
            .variant(icon_button::Variant::Filled)
            .size(140.0)
            .on_press(Message::ToggleBottomSheet),
    ]
    .spacing(16)
    .padding(20)
    .into();

    BottomSheet::new(
        host_content,
        "This is the bottom sheet content. It slides up from the bottom of the screen.",
    )
    .modal(true)
    .expanded(demo.bottom_sheet_expanded)
    .on_dismiss(Message::CloseBottomSheet)
    .drag_handle(true)
    .into()
}
