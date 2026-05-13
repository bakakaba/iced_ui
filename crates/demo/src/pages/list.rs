use iced::Length;
use iced::widget::{column, text};
use iced_ui::list;

use crate::Element;
use crate::message::Message;

pub(super) fn build<'a>() -> Element<'a, Message> {
    let example_list = list::List::new()
        .push(list::Item::new(text("Apple")))
        .push(list::Item::new(text("Banana")))
        .push(list::Item::new(text("Cherry")))
        .push(list::Item::new(text("Dragonfruit")))
        .push(list::Item::new(text("Elderberry")))
        .width(Length::Fixed(200.0));

    column![
        text("List").size(20),
        text("A vertical list of interactive items with hover/press feedback.").size(14),
        example_list,
    ]
    .spacing(16)
    .padding(20)
    .into()
}
