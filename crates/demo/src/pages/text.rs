use iced::widget::column;
use iced_ui::text::Text as UiText;

use crate::Element;

pub(super) fn build<'a>() -> Element<'a, super::Message> {
    column![
        UiText::h1("Heading 1"),
        UiText::h2("Heading 2"),
        UiText::h3("Heading 3"),
        UiText::h4("Heading 4"),
        UiText::h5("Heading 5"),
    ]
    .spacing(16)
    .padding(20)
    .into()
}
