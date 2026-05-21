use iced::widget::column;
use iced_ui::text::Text as UiText;

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
}
