use iced::widget::{column, row, text};
use iced_ui::fab::{Fab, FabSize};

use crate::Element;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Pressed,
}

/// Action returned from fab page for the parent to handle.
pub(super) enum Action {
    LogAction(String),
}

pub(super) fn update(message: Message) -> Action {
    match message {
        Message::Pressed => Action::LogAction("FAB pressed".to_string()),
    }
}

pub(super) fn view<'a>() -> Element<'a, Message> {
    let small_fab = Fab::new(text("+").size(18))
        .size(FabSize::Small)
        .on_press(Message::Pressed);

    let regular_fab = Fab::new(text("+").size(24)).on_press(Message::Pressed);

    let large_fab = Fab::new(text("+").size(36))
        .size(FabSize::Large)
        .on_press(Message::Pressed);

    let extended_fab = Fab::new(text("+").size(18))
        .label(text("Create").size(16))
        .on_press(Message::Pressed);

    let lowered_fab = Fab::new(text("+").size(24))
        .lowered()
        .on_press(Message::Pressed);

    column![
        text("FAB (Floating Action Button)").size(20),
        text("Small, Regular, Large, Extended, and Lowered variants.").size(14),
        row![small_fab, regular_fab, large_fab, extended_fab, lowered_fab].spacing(16),
    ]
    .spacing(16)
    .padding(20)
    .into()
}
