use iced::Length;
use iced::widget::{column, row, text};
use iced_ui::divider::Divider;

use crate::Element;

pub(super) fn build<'a>() -> Element<'a, super::Message> {
    column![
        text("Divider").size(20),
        text("Horizontal and vertical separators with optional insets.").size(14),
        text("Full width:").size(14),
        Divider::horizontal(),
        text("With inset:").size(14),
        Divider::horizontal().inset(iced_ui::Space::sx(4.0)),
        row![
            text("Vertical:").size(14),
            Divider::vertical(),
            text("Between content").size(14),
        ]
        .spacing(8)
        .height(Length::Fixed(40.0)),
    ]
    .spacing(12)
    .padding(20)
    .into()
}
