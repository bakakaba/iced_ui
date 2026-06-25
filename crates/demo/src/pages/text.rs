use iced::widget::{column, text};
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {}

#[derive(Default)]
pub(crate) struct TextPage;

impl super::PageView for TextPage {
    type Msg = Msg;
    const LABEL: &'static str = "Text";

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        column![
            text("Semantic heading levels with bold weight and scaled sizes.").size(14),
            Text::h2("Heading Levels"),
            Text::h1("Heading 1"),
            Text::h2("Heading 2"),
            Text::h3("Heading 3"),
            Text::h4("Heading 4"),
            Text::h5("Heading 5"),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}
