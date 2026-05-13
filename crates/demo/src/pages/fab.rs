use iced::widget::{column, row, text};
use iced_ui::fab::{Fab, FabSize};

use crate::Element;
use crate::message::Message;

pub(super) fn build<'a>() -> Element<'a, Message> {
    let small_fab = Fab::new(text("+").size(18))
        .size(FabSize::Small)
        .on_press(Message::FabPressed);

    let regular_fab = Fab::new(text("+").size(24)).on_press(Message::FabPressed);

    let large_fab = Fab::new(text("+").size(36))
        .size(FabSize::Large)
        .on_press(Message::FabPressed);

    let extended_fab = Fab::new(text("+").size(18))
        .label(text("Create").size(16))
        .on_press(Message::FabPressed);

    let lowered_fab = Fab::new(text("+").size(24))
        .lowered()
        .on_press(Message::FabPressed);

    column![
        text("FAB (Floating Action Button)").size(20),
        text("Small, Regular, Large, Extended, and Lowered variants.").size(14),
        row![small_fab, regular_fab, large_fab, extended_fab, lowered_fab].spacing(16),
    ]
    .spacing(16)
    .padding(20)
    .into()
}
