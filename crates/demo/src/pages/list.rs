use iced::Length;
use iced::widget::{column, text};
use iced_ui::list;
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {}

#[derive(Default)]
pub(crate) struct ListPage;

impl super::PageView for ListPage {
    type Msg = Msg;
    const LABEL: &'static str = "List";

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let example_list = list::List::new()
            .push(list::Item::new(text("Apple")))
            .push(list::Item::new(text("Banana")))
            .push(list::Item::new(text("Cherry")))
            .push(list::Item::new(text("Dragonfruit")))
            .push(list::Item::new(text("Elderberry")))
            .width(Length::Fixed(200.0));

        column![
            Text::h1("List"),
            text("A vertical list of interactive items with hover/press feedback.").size(14),
            Text::h2("Default"),
            example_list,
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
